pub mod domain;
pub mod application;
pub mod adapters;
pub mod ingestion;
pub mod network;

use std::sync::Arc;
use std::path::PathBuf;
use tracing::{info, error};

use crate::application::facade::ComicFacade;
use crate::application::catalog_service::CatalogService;
use crate::application::reader_service::ReaderService;
use crate::adapters::outbound::persistence::SqliteComicRepository;
use crate::adapters::outbound::file_readers::{CbzAdapter, CbrAdapter};
use crate::adapters::outbound::cache::DiskImageCache;
use crate::domain::ports::ComicFileReader;
use crate::adapters::inbound::tauri_bridge::AppState;

const DB_URL: &str = "sqlite://data/comic.db?mode=rwc";
const SERVER_PORT: u16 = 8080;

#[tokio::main]
async fn main() {
    // 1. Inicializar directorios de datos y logs
    let data_dir = std::path::Path::new("data");
    let _ = std::fs::create_dir_all(data_dir);
    let log_dir = std::path::Path::new("log");
    let _ = std::fs::create_dir_all(log_dir);

    // 2. Inicializar logging a consola y a archivo rotatorio
    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .filename_prefix("comic")
        .filename_suffix("log")
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .build(log_dir)
        .expect("No se pudo inicializar el appender de logs");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let _ = tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .try_init();

    info!("==========================================");
    info!("   Comic App (Tauri Showcase) Iniciando   ");
    info!("==========================================");
    info!("DB URL: {}", DB_URL);
    info!("Server port: {}", SERVER_PORT);

    // 3. Inicializar persistencia y base de datos SQLite con modo WAL
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await
        .expect("No se pudo conectar a SQLite");

    crate::adapters::outbound::persistence::schema::initialize_schema(&pool)
        .await
        .expect("No se pudo inicializar el esquema de base de datos");

    // 4. Inicializar capas de dominio y adaptadores
    let repo = Arc::new(SqliteComicRepository::new(pool));
    let readers: Vec<Arc<dyn ComicFileReader>> = vec![
        Arc::new(CbzAdapter::new()),
        Arc::new(CbrAdapter::new()),
    ];
    let cache = Arc::new(DiskImageCache::new(PathBuf::from(".cache")));
    let catalog = CatalogService::new(repo.clone(), repo.clone(), readers.clone());
    let reader = ReaderService::new(repo.clone(), repo.clone(), cache, readers);
    let facade = Arc::new(ComicFacade::new(catalog, reader, repo));

    // 5. Iniciar servidor HTTP Axum en segundo plano para clientes remotos / OPDS
    let facade_for_server = facade.clone();
    tokio::spawn(async move {
        info!("Iniciando servidor HTTP Axum en puerto {}", SERVER_PORT);
        let server_state = crate::adapters::inbound::rest_api::ServerState {
            facade: (*facade_for_server).clone(),
            session_token: uuid::Uuid::new_v4().to_string(),
        };
        let app = crate::adapters::inbound::rest_api::create_router(server_state);
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
        if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
            let _ = axum::serve(listener, app).await;
        } else {
            error!("No se pudo enlazar el servidor HTTP en el puerto {}", SERVER_PORT);
        }
    });

    // 6. Iniciar descubrimiento mDNS
    crate::network::mdns::MdnsAnnouncer::start(SERVER_PORT);

    // 7. Configurar e iniciar el runtime de Tauri
    let app_state = AppState {
        facade: facade.clone(),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            crate::adapters::inbound::tauri_bridge::get_collections,
            crate::adapters::inbound::tauri_bridge::get_collection,
            crate::adapters::inbound::tauri_bridge::save_collection,
            crate::adapters::inbound::tauri_bridge::delete_collection,
            crate::adapters::inbound::tauri_bridge::get_comics_by_collection,
            crate::adapters::inbound::tauri_bridge::get_comic_page,
            crate::adapters::inbound::tauri_bridge::scan_monitored_paths,
            crate::adapters::inbound::tauri_bridge::get_server_status,
            crate::adapters::inbound::tauri_bridge::get_trusted_devices,
            crate::adapters::inbound::tauri_bridge::add_trusted_device,
            crate::adapters::inbound::tauri_bridge::remove_trusted_device,
            crate::adapters::inbound::tauri_bridge::pick_image_file,
            crate::adapters::inbound::tauri_bridge::pick_folder,
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación Tauri");
}
