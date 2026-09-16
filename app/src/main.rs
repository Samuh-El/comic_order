mod db;
mod comic_reader;
mod qr;
mod server;
mod ui;

use std::collections::HashMap;

use iced::widget::{button, column, container, image, row, stack, text, svg, Space};
use iced::keyboard;
use iced::{Alignment, Element, Font, Length, Subscription, Task, Theme};
use tracing::{info, warn, error, debug};

use db::{Collection, Comic, Database};
use ui::metadata_editor::MetadataForm;

const DB_URL: &str = "sqlite://comic.db?mode=rwc";
const SERVER_PORT: u16 = 8080;

fn main() -> iced::Result {
    // Initialize logging to ./log/ with daily rotation
    let log_dir = std::path::Path::new("log");
    std::fs::create_dir_all(log_dir).expect("No se pudo crear el directorio de logs");

    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .filename_prefix("comic")
        .filename_suffix("log")
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .build(log_dir)
        .expect("No se pudo inicializar el appender de archivos");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .init();

    info!("=== Comic App iniciada ===");
    info!("DB URL: {}", DB_URL);
    info!("Server port: {}", SERVER_PORT);

    // Load window icon
    let icon = load_icon();

    let mut app = iced::application("Comic", ComicApp::update, ComicApp::view)
        .subscription(ComicApp::subscription)
        .theme(|_| Theme::Dark)
        .default_font(Font::with_name("Segoe UI"))
        .window_size((1100.0, 700.0));

    if let Some(icon) = icon {
        app = app.window(iced::window::Settings {
            icon: Some(icon),
            ..Default::default()
        });
    }

    app.run_with(ComicApp::new)
}

fn load_icon() -> Option<iced::window::icon::Icon> {
    let icon_bytes = include_bytes!("../assets/wow-icon.png");
    let img = ::image::load_from_memory(icon_bytes).ok()?.to_rgba8();
    let (width, height) = ::image::GenericImageView::dimensions(&img);
    iced::window::icon::from_rgba(img.into_raw(), width, height).ok()
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
    Loading,
    Main,
    Reader,
}

struct ComicApp {
    // State
    view: AppView,
    db: Option<Database>,
    error_message: Option<String>,

    // Collections
    collections: Vec<Collection>,
    selected_collection_id: Option<i64>,
    selected_collection_name: String,
    show_new_collection: bool,
    new_collection_name: String,

    // Comics
    comics: Vec<Comic>,
    comic_handles: HashMap<i64, image::Handle>,
    is_scanning: bool,

    // Reader
    reading_comic: Option<Comic>,
    current_page: usize,
    total_pages: usize,
    page_handle: Option<image::Handle>,
    is_loading_page: bool,

    // Metadata editor
    editing_form: Option<MetadataForm>,
    collection_editor_form: Option<ui::collection_editor::CollectionForm>,

    // Server
    server_running: bool,
    server_url: Option<String>,
    server_token: Option<String>,
    qr_handle: Option<image::Handle>,
    show_qr: bool,
    shutdown_sender: Option<tokio::sync::oneshot::Sender<()>>,

    // Collection context menu
    context_menu_collection: Option<i64>,
    renaming_collection: Option<i64>,
    rename_input: String,

    // UI State
    is_sidebar_open: bool,
    current_time: String,

    // Reader state (Zoom/Pan)
    zoom: f32,
    pan: iced::Vector,
    is_dragging: bool,
    last_mouse_pos: iced::Point,
    drag_start_pos: iced::Point,
    show_reader_controls: bool,

    // Trusted Devices
    trusted_devices_form: Option<ui::trusted_devices::TrustedDevicesForm>,
    trusted_qr_handle: Option<iced::widget::image::Handle>,
    trusted_qr_url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Init
    DbConnected(Result<Database, String>),
    CollectionsLoaded(Vec<Collection>),

    // Collections
    SelectCollection(i64),
    ToggleNewCollection,
    NewCollectionNameChanged(String),
    CreateCollection,
    CollectionCreated(Result<i64, String>),
    DeleteCollection(i64),
    CollectionDeleted(Result<(), String>),
    ToggleCollectionMenu(i64),
    StartRenameCollection(i64),
    RenameInputChanged(String),
    ConfirmRename,
    CollectionRenamed(Result<(), String>),

    // Paths
    AddPath,
    PathSelected(Option<String>),
    PathAdded(Result<(), String>),

    // Comics
    ComicsLoaded(Vec<Comic>),
    CoverLoaded(i64, Option<Vec<u8>>),
    OpenComic(i64),
    PageLoaded(Option<Vec<u8>>),

    // Reader
    NextPage,
    PrevPage,
    CloseReader,

    // Editor
    EditComic(i64),
    EditorTitleChanged(String),

    // Reader interactions
    CanvasClicked,
    EditorYearChanged(String),
    EditorIssueChanged(String),
    EditorSagaChanged(String),
    SaveMetadata,
    MetadataSaved(Result<(), String>),
    CloseEditor,

    // Collection Editor
    EditCollection(i64),
    CollectionEditorNameChanged(String),
    SelectCollectionIcon,
    CollectionIconSelected(Option<String>),
    RemoveCollectionIcon,
    SaveCollectionEditor,
    CollectionEditorSaved(Result<(), String>),
    CloseCollectionEditor,

    // Server
    ToggleServer,
    ServerStarted,
    StopServer,
    CloseQR,

    // Trusted Devices
    ManageTrustedDevices,
    TrustedDevicesLoaded(Vec<db::TrustedDevice>),
    AddTrustedDevice,
    TrustedQRGenerated(String, iced::widget::image::Handle),
    DeleteTrustedDevice(i64),
    TrustedDeviceDeleted(Result<(), String>),
    CloseTrustedDevices,

    // Scanning
    ScanComplete(Result<Vec<Comic>, String>),

    // Keyboard
    KeyPressed(keyboard::Key),

    // UI Controls
    ToggleSidebar,
    ToggleReaderControls,
    Tick(chrono::DateTime<chrono::Local>),

    // Mouse Events for Reader
    MouseScrolled(f32),
    MousePressed(iced::mouse::Button),
    MouseReleased(iced::mouse::Button),
    MouseMoved(iced::Point),
}

impl ComicApp {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            view: AppView::Loading,
            db: None,
            error_message: None,
            collections: Vec::new(),
            selected_collection_id: None,
            selected_collection_name: String::new(),
            show_new_collection: false,
            new_collection_name: String::new(),
            comics: Vec::new(),
            comic_handles: HashMap::new(),
            reading_comic: None,
            current_page: 0,
            total_pages: 0,
            page_handle: None,
            editing_form: None,
            collection_editor_form: None,
            server_running: false,
            server_url: None,
            server_token: None,
            qr_handle: None,
            show_qr: false,
            context_menu_collection: None,
            renaming_collection: None,
            rename_input: String::new(),
            is_scanning: false,
            is_loading_page: false,
            shutdown_sender: None,
            is_sidebar_open: true,
            current_time: chrono::Local::now()
                .format("%I:%M %p  %a %b %d")
                .to_string(),
            zoom: 1.0,
            pan: iced::Vector::default(),
            is_dragging: false,
            last_mouse_pos: iced::Point::ORIGIN,
            drag_start_pos: iced::Point::ORIGIN,
            show_reader_controls: true,
            trusted_devices_form: None,
            trusted_qr_handle: None,
            trusted_qr_url: None,
        };

        let task = Task::perform(
            async {
                Database::connect(DB_URL)
                    .await
                    .map_err(|e| e.to_string())
            },
            Message::DbConnected,
        );

        (app, task)
    }

    fn load_collections(&self) -> Task<Message> {
        if let Some(db) = &self.db {
            let db = db.clone();
            Task::perform(
                async move { db.get_collections().await.unwrap_or_default() },
                Message::CollectionsLoaded,
            )
        } else {
            Task::none()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let clock = iced::time::every(std::time::Duration::from_secs(1)).map(|_| {
            Message::Tick(chrono::Local::now())
        });

        let keyboard = keyboard::on_key_press(|key, _modifiers| {
            Some(Message::KeyPressed(key))
        });

        let mouse = if self.view == AppView::Reader {
            iced::event::listen_with(|event, _status, _id| {
                match event {
                    iced::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                        match delta {
                            iced::mouse::ScrollDelta::Lines { y, .. } => Some(Message::MouseScrolled(y)),
                            iced::mouse::ScrollDelta::Pixels { y, .. } => Some(Message::MouseScrolled(y / 20.0)),
                        }
                    }
                    iced::Event::Mouse(iced::mouse::Event::ButtonPressed(button)) => Some(Message::MousePressed(button)),
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(button)) => Some(Message::MouseReleased(button)),
                    iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => Some(Message::MouseMoved(position)),
                    _ => None,
                }
            })
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![clock, keyboard, mouse])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DbConnected(result) => {
                match result {
                    Ok(db) => {
                        info!("Conexión a SQLite establecida correctamente");
                        self.db = Some(db.clone());
                        self.view = AppView::Main;
                        return Task::perform(
                            async move { db.get_collections().await.unwrap_or_default() },
                            Message::CollectionsLoaded,
                        );
                    }
                    Err(e) => {
                        error!("Error conectando a SQLite: {}", e);
                        self.error_message = Some(format!("Error de BD: {}. Verifique el archivo comic.db", e));
                        self.view = AppView::Main;
                    }
                }
            }

            Message::Tick(now) => {
                self.current_time = now.format("%I:%M %p  %a %b %d").to_string();
            }

            Message::ToggleSidebar => {
                self.is_sidebar_open = !self.is_sidebar_open;
            }

            Message::ToggleReaderControls => {
                self.show_reader_controls = !self.show_reader_controls;
            }

            Message::CollectionsLoaded(collections) => {
                info!("Colecciones cargadas: {} encontradas", collections.len());
                self.collections = collections;
            }

            Message::SelectCollection(id) => {
                info!("Colección seleccionada: id={}", id);
                self.selected_collection_id = Some(id);
                if let Some(c) = self.collections.iter().find(|c| c.id == id) {
                    self.selected_collection_name = c.name.clone();
                }
                self.comic_handles.clear();
                if let Some(db) = &self.db {
                    let db = db.clone();
                    return Task::perform(
                        async move { db.get_comics_by_collection(id).await.unwrap_or_default() },
                        Message::ComicsLoaded,
                    );
                }
            }

            Message::ToggleNewCollection => {
                self.show_new_collection = !self.show_new_collection;
                self.new_collection_name.clear();
            }

            Message::NewCollectionNameChanged(name) => {
                self.new_collection_name = name;
            }

            Message::CreateCollection => {
                let name = self.new_collection_name.trim().to_string();
                if name.is_empty() {
                    warn!("Intento de crear colección con nombre vacío");
                    return Task::none();
                }
                info!("Creando colección: '{}'", name);
                self.show_new_collection = false;
                if let Some(db) = &self.db {
                    let db = db.clone();
                    return Task::perform(
                        async move { db.create_collection(&name).await.map_err(|e| e.to_string()) },
                        Message::CollectionCreated,
                    );
                }
            }

            Message::CollectionCreated(result) => {
                self.new_collection_name.clear();
                match &result {
                    Ok(id) => {
                        info!("Colección creada con id={}", id);
                        self.selected_collection_id = Some(*id);
                    }
                    Err(e) => error!("Error creando colección: {}", e),
                }
                return self.reload_collections();
            }

            Message::DeleteCollection(id) => {
                if let Some(db) = &self.db {
                    let db = db.clone();
                    return Task::perform(
                        async move { db.delete_collection(id).await.map_err(|e| e.to_string()) },
                        Message::CollectionDeleted,
                    );
                }
            }

            Message::CollectionDeleted(_) => {
                self.selected_collection_id = None;
                self.comics.clear();
                self.comic_handles.clear();
                self.context_menu_collection = None;
                return self.reload_collections();
            }

            Message::ToggleCollectionMenu(id) => {
                if self.context_menu_collection == Some(id) {
                    self.context_menu_collection = None;
                } else {
                    self.context_menu_collection = Some(id);
                    self.renaming_collection = None;
                }
            }

            Message::StartRenameCollection(id) => {
                if let Some(c) = self.collections.iter().find(|c| c.id == id) {
                    info!("Iniciando renombrado de colección: '{}' id={}", c.name, id);
                    self.rename_input = c.name.clone();
                    self.renaming_collection = Some(id);
                    self.context_menu_collection = None;
                }
            }

            Message::RenameInputChanged(v) => {
                self.rename_input = v;
            }

            Message::ConfirmRename => {
                if let (Some(id), Some(db)) = (self.renaming_collection, &self.db) {
                    let new_name = self.rename_input.trim().to_string();
                    if new_name.is_empty() {
                        warn!("Intento de renombrar con nombre vacío");
                        return Task::none();
                    }
                    info!("Renombrando colección id={} a '{}'", id, new_name);
                    self.renaming_collection = None;
                    let db = db.clone();
                    return Task::perform(
                        async move { db.rename_collection(id, &new_name).await.map_err(|e| e.to_string()) },
                        Message::CollectionRenamed,
                    );
                }
            }

            Message::CollectionRenamed(result) => {
                match &result {
                    Ok(()) => info!("Colección renombrada correctamente"),
                    Err(e) => error!("Error renombrando colección: {}", e),
                }
                self.renaming_collection = None;
                return self.reload_collections();
            }

            Message::AddPath => {
                info!("Abriendo diálogo de selección de carpeta");
                return Task::perform(
                    async {
                        rfd_pick_folder().await
                    },
                    Message::PathSelected,
                );
            }

            Message::PathSelected(Some(path)) => {
                info!("Carpeta seleccionada: {}", path);
                if let (Some(db), Some(collection_id)) = (&self.db, self.selected_collection_id) {
                    let db = db.clone();
                    let path_clone = path.clone();
                    return Task::perform(
                        async move {
                            db.add_collection_path(collection_id, &path_clone)
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        },
                        Message::PathAdded,
                    );
                }
            }

            Message::PathSelected(None) => {
                info!("Selección de carpeta cancelada");
            }

            Message::PathAdded(result) => {
                match &result {
                    Ok(()) => {
                        info!("Ruta añadida correctamente, iniciando escaneo de comics...");
                        self.is_scanning = true;
                        return self.scan_collection_paths();
                    }
                    Err(e) => {
                        error!("Error añadiendo ruta: {}", e);
                        self.error_message = Some(format!("Error añadiendo carpeta: {}", e));
                    }
                }
            }

            Message::ScanComplete(result) => {
                match &result {
                    Ok(new_comics) => {
                        info!("Escaneo completo: {} comics nuevos encontrados", new_comics.len());
                        for c in new_comics {
                            debug!("  Comic: '{}' ({}) - {} páginas", c.title, c.file_type, c.page_count);
                        }
                    }
                    Err(e) => {
                        error!("Error durante escaneo: {}", e);
                    }
                }
                self.is_scanning = false;
                if let (Ok(new_comics), Some(db)) = (result, &self.db) {
                    if new_comics.is_empty() {
                        info!("No se encontraron comics nuevos");
                        // Still reload to show current comics
                        if let Some(collection_id) = self.selected_collection_id {
                            let db = db.clone();
                            return Task::perform(
                                async move { db.get_comics_by_collection(collection_id).await.unwrap_or_default() },
                                Message::ComicsLoaded,
                            );
                        }
                        return Task::none();
                    }
                    let db = db.clone();
                    let collection_id = self.selected_collection_id;
                    return Task::perform(
                        async move {
                            for comic in &new_comics {
                                match db.upsert_comic(comic).await {
                                    Ok(id) => info!("Comic guardado: '{}' id={}", comic.title, id),
                                    Err(e) => error!("Error guardando comic '{}': {}", comic.title, e),
                                }
                            }
                            if let Some(cid) = collection_id {
                                db.get_comics_by_collection(cid).await.unwrap_or_default()
                            } else {
                                Vec::new()
                            }
                        },
                        Message::ComicsLoaded,
                    );
                }
            }

            Message::ComicsLoaded(comics) => {
                info!("Comics cargados para mostrar: {}", comics.len());
                self.comics = comics;
                // Load covers
                let tasks: Vec<Task<Message>> = self.comics.iter().map(|comic| {
                    let id = comic.id;
                    if comic.cover_data.is_some() {
                        let data = comic.cover_data.clone();
                        Task::perform(async move { (id, data) }, |(id, data)| Message::CoverLoaded(id, data))
                    } else {
                        let file_path = comic.file_path.clone();
                        let title = comic.title.clone();
                        Task::perform(
                            async move {
                                debug!("Extrayendo portada de: '{}'", title);
                                let cover = comic_reader::extract_cover(&file_path);
                                (id, cover)
                            },
                            |(id, data)| Message::CoverLoaded(id, data),
                        )
                    }
                }).collect();
                return Task::batch(tasks);
            }

            Message::CoverLoaded(id, Some(data)) => {
                let handle = image::Handle::from_bytes(data.clone());
                self.comic_handles.insert(id, handle);

                // Also save cover to DB if the comic didn't have one
                if let Some(comic) = self.comics.iter().find(|c| c.id == id) {
                    if comic.cover_data.is_none() {
                        if let Some(db) = &self.db {
                            let db = db.clone();
                            let mut updated = comic.clone();
                            updated.cover_data = Some(data);
                            return Task::perform(
                                async move { let _ = db.upsert_comic(&updated).await; },
                                |_| Message::MetadataSaved(Ok(())),
                            );
                        }
                    }
                }
            }

            Message::CoverLoaded(_, None) => {}

            Message::OpenComic(id) => {
                if let Some(comic) = self.comics.iter().find(|c| c.id == id) {
                    info!("Abriendo comic: '{}' ({} páginas)", comic.title, comic.page_count);
                    let file_path = comic.file_path.clone();
                    self.reading_comic = Some(comic.clone());
                    self.current_page = 0;
                    self.total_pages = comic.page_count as usize;
                    self.view = AppView::Reader;
                    self.is_loading_page = true;
                    self.page_handle = None;
                    self.zoom = 1.0;
                    self.pan = iced::Vector::default();
                    self.show_reader_controls = true;

                    return Task::perform(
                        async move { comic_reader::get_full_page(&file_path, 0) },
                        Message::PageLoaded,
                    );
                }
            }

            Message::PageLoaded(Some(data)) => {
                self.is_loading_page = false;
                self.page_handle = Some(image::Handle::from_bytes(data));
            }

            Message::PageLoaded(None) => {
                self.is_loading_page = false;
                self.page_handle = None;
            }

            Message::NextPage => {
                if self.current_page < self.total_pages.saturating_sub(1) {
                    self.current_page += 1;
                    self.is_loading_page = true;
                    self.page_handle = None;
                    if let Some(comic) = &self.reading_comic {
                        let file_path = comic.file_path.clone();
                        let page = self.current_page;
                        return Task::perform(
                            async move { comic_reader::get_full_page(&file_path, page) },
                            Message::PageLoaded,
                        );
                    }
                }
            }

            Message::PrevPage => {
                if self.current_page > 0 {
                    self.current_page -= 1;
                    self.is_loading_page = true;
                    self.page_handle = None;
                    if let Some(comic) = &self.reading_comic {
                        let file_path = comic.file_path.clone();
                        let page = self.current_page;
                        return Task::perform(
                            async move { comic_reader::get_full_page(&file_path, page) },
                            Message::PageLoaded,
                        );
                    }
                }
            }

            Message::CloseReader => {
                self.view = AppView::Main;
                self.reading_comic = None;
                self.page_handle = None;
                self.zoom = 1.0;
                self.pan = iced::Vector::default();
                self.show_reader_controls = true;
            }

            Message::EditComic(id) => {
                if let Some(comic) = self.comics.iter().find(|c| c.id == id) {
                    self.editing_form = Some(MetadataForm::new(
                        comic.id,
                        &comic.title,
                        comic.year,
                        comic.issue_number,
                        comic.saga.as_deref(),
                    ));
                }
            }

            Message::EditorTitleChanged(v) => {
                if let Some(form) = &mut self.editing_form {
                    form.title = v;
                }
            }

            Message::EditorYearChanged(v) => {
                if let Some(form) = &mut self.editing_form {
                    form.year = v;
                }
            }

            Message::EditorIssueChanged(v) => {
                if let Some(form) = &mut self.editing_form {
                    form.issue_number = v;
                }
            }

            Message::EditorSagaChanged(v) => {
                if let Some(form) = &mut self.editing_form {
                    form.saga = v;
                }
            }

            Message::SaveMetadata => {
                if let (Some(form), Some(db)) = (self.editing_form.take(), &self.db) {
                    if let Some(comic) = self.comics.iter().find(|c| c.id == form.comic_id) {
                        info!("Guardando metadatos: '{}' año={} num={} saga={}", form.title, form.year, form.issue_number, form.saga);
                        let db = db.clone();
                        let mut updated = comic.clone();
                        updated.title = form.title;
                        updated.year = form.year.parse().ok();
                        updated.issue_number = form.issue_number.parse().ok();
                        updated.saga = if form.saga.is_empty() {
                            None
                        } else {
                            Some(form.saga)
                        };

                        return Task::perform(
                            async move {
                                db.upsert_comic(&updated).await.map(|_| ()).map_err(|e| e.to_string())
                            },
                            Message::MetadataSaved,
                        );
                    }
                }
            }

            Message::MetadataSaved(Ok(())) => {
                if let Some(collection_id) = self.selected_collection_id {
                    if let Some(db) = &self.db {
                        let db = db.clone();
                        return Task::perform(
                            async move { db.get_comics_by_collection(collection_id).await.unwrap_or_default() },
                            Message::ComicsLoaded,
                        );
                    }
                }
            }

            Message::MetadataSaved(Err(e)) => {
                error!("Error guardando metadatos: {}", e);
                self.error_message = Some(e);
            }

            Message::CloseEditor => {
                self.editing_form = None;
            }

            // Collection Editor
            Message::EditCollection(id) => {
                self.context_menu_collection = None;
                if let Some(col) = self.collections.iter().find(|c| c.id == id) {
                    self.collection_editor_form = Some(ui::collection_editor::CollectionForm::new(
                        col.id,
                        &col.name,
                        col.icon_data.clone(),
                    ));
                }
            }
            Message::CollectionEditorNameChanged(name) => {
                if let Some(form) = &mut self.collection_editor_form {
                    form.name = name;
                }
            }
            Message::SelectCollectionIcon => {
                return Task::perform(pick_image(), Message::CollectionIconSelected);
            }
            Message::CollectionIconSelected(Some(path)) => {
                if let Some(form) = &mut self.collection_editor_form {
                    if let Ok(data) = std::fs::read(&path) {
                        // Optional: Resize to 1:1 if needed, for now just load
                        form.icon_data = Some(data);
                    }
                }
            }
            Message::CollectionIconSelected(None) => {}
            Message::RemoveCollectionIcon => {
                if let Some(form) = &mut self.collection_editor_form {
                    form.icon_data = None;
                }
            }
            Message::SaveCollectionEditor => {
                if let (Some(form), Some(db)) = (&self.collection_editor_form, &self.db) {
                    let col = Collection {
                        id: form.id,
                        name: form.name.clone(),
                        icon_data: form.icon_data.clone(),
                    };
                    let db = db.clone();
                    return Task::perform(
                        async move {
                            db.update_collection(&col).await.map_err(|e| e.to_string())
                        },
                        Message::CollectionEditorSaved,
                    );
                }
            }
            Message::CollectionEditorSaved(Ok(_)) => {
                self.collection_editor_form = None;
                return self.load_collections();
            }
            Message::CollectionEditorSaved(Err(e)) => {
                self.error_message = Some(format!("Error al guardar colección: {}", e));
            }
            Message::CloseCollectionEditor => {
                self.collection_editor_form = None;
            }
            Message::ToggleServer => {
                if self.server_running {
                    // Just show QR if already running
                    self.show_qr = true;
                } else {
                    if let Some(db) = &self.db {
                        let db = db.clone();
                        let port = SERVER_PORT;
                        
                        // Generate a secure session token
                        let token = uuid::Uuid::new_v4().to_string().replace("-", "")[..16].to_string();
                        info!("Token de seguridad generado: ******");
                        
                        self.server_running = true;
                        let url = qr::get_server_url(port, Some(&token));
                        info!("Iniciando servidor HTTP en {}", url);
                        self.server_url = Some(url.clone());
                        self.server_token = Some(token.clone());
                        self.show_qr = true;

                        // Generate QR
                        if let Some((data, w, h)) = qr::generate_qr_image(&url, 256) {
                            info!("Código QR generado ({}x{})", w, h);
                            self.qr_handle = Some(image::Handle::from_rgba(w, h, data));
                        }

                        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
                        self.shutdown_sender = Some(tx);

                        return Task::perform(
                            async move {
                                let state = server::ServerState { db, token };
                                let router = server::create_router(state);
                                let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
                                    .await
                                    .expect("Cannot bind to port");
                                info!("Servidor HTTP escuchando en 0.0.0.0:{}", port);
                                
                                axum::serve(listener, router)
                                    .with_graceful_shutdown(async move {
                                        rx.await.ok();
                                        info!("Apagando servidor HTTP Axum...");
                                    })
                                    .await.ok();
                            },
                            |_| Message::ServerStarted,
                        );
                    }
                }
            }

            Message::ServerStarted => {
                info!("Servidor HTTP iniciado correctamente");
            }

            Message::StopServer => {
                info!("Deteniendo servidor HTTP por petición del usuario");
                if let Some(tx) = self.shutdown_sender.take() {
                    let _ = tx.send(());
                }
                self.server_running = false;
                self.show_qr = false;
                self.server_url = None;
                self.server_token = None;
                self.qr_handle = None;
            }

            Message::CloseQR => {
                self.show_qr = false;
            }

            Message::MouseScrolled(delta) => {
                if self.view == AppView::Reader {
                    if delta > 0.0 {
                        self.zoom *= 1.1;
                    } else {
                        self.zoom /= 1.1;
                    }
                    self.zoom = self.zoom.clamp(0.1, 5.0);
                    
                    // Adjust pan to zoom towards center? Simple for now
                    if self.zoom == 1.0 {
                        self.pan = iced::Vector::new(0.0, 0.0);
                    }
                }
            }

            Message::MousePressed(button) => {
                if self.view == AppView::Reader && button == iced::mouse::Button::Left {
                    self.is_dragging = true;
                    self.drag_start_pos = self.last_mouse_pos;
                }
            }

            Message::MouseReleased(button) => {
                if button == iced::mouse::Button::Left {
                    self.is_dragging = false;
                }
            }

            Message::CanvasClicked => {
                let dx = self.last_mouse_pos.x - self.drag_start_pos.x;
                let dy = self.last_mouse_pos.y - self.drag_start_pos.y;
                if dx.abs() < 5.0 && dy.abs() < 5.0 {
                    // Only toggle if it was a tiny movement (a click, not a drag)
                    self.show_reader_controls = !self.show_reader_controls;
                }
            }

            Message::MouseMoved(position) => {
                if self.is_dragging && self.view == AppView::Reader {
                    let delta = position - self.last_mouse_pos;
                    self.pan.x += delta.x;
                    self.pan.y += delta.y;
                }
                self.last_mouse_pos = position;
            }

            Message::KeyPressed(key) => {
                if self.view == AppView::Reader {
                    match key {
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                            return self.update(Message::NextPage);
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                            return self.update(Message::PrevPage);
                        }
                        keyboard::Key::Named(keyboard::key::Named::Escape) => {
                            self.zoom = 1.0;
                            self.pan = iced::Vector::new(0.0, 0.0);
                            return self.update(Message::CloseReader);
                        }
                        _ => {}
                    }
                }
            }

            Message::ManageTrustedDevices => {
                if let Some(db) = &self.db {
                    let db = db.clone();
                    return Task::perform(
                        async move { db.get_trusted_devices().await.unwrap_or_default() },
                        Message::TrustedDevicesLoaded,
                    );
                }
            }
            Message::TrustedDevicesLoaded(devices) => {
                self.trusted_devices_form = Some(ui::trusted_devices::TrustedDevicesForm::new(devices));
            }
            Message::AddTrustedDevice => {
                if let Some(db) = &self.db {
                    let db = db.clone();
                    let server_url = self.server_url.clone();
                    info!("[UI] Iniciando registro de dispositivo recurrente");
                    return Task::perform(
                        async move {
                            if let Some(name) = prompt_name().await {
                                info!("[UI] Nombre recibido: {}", name);
                                let token = uuid::Uuid::new_v4().to_string().replace("-", "")[..16].to_string();
                                if db.add_trusted_device(&token, &name).await.is_ok() {
                                    let url = if let Some(existing_url) = server_url {
                                        let base = existing_url.split('?').next().unwrap_or(&existing_url);
                                        format!("{}?token={}", base, token)
                                    } else {
                                        crate::qr::get_server_url(8080, Some(&token))
                                    };

                                    info!("[UI] Generando QR para: {}", url);
                                    if let Some(handle) = generate_qr(&url) {
                                        return Some((url, handle));
                                    } else {
                                        error!("[UI] Error generando handle de imagen QR");
                                    }
                                } else {
                                    error!("[UI] Error guardando dispositivo en DB");
                                }
                            } else {
                                info!("[UI] Dialogo de nombre cancelado o vacio");
                            }
                            None
                        },
                        |res| {
                            if let Some((url, handle)) = res {
                                Message::TrustedQRGenerated(url, handle)
                            } else {
                                Message::ManageTrustedDevices
                            }
                        }
                    );
                }
            }
            Message::TrustedQRGenerated(url, handle) => {
                info!("[UI] QR de dispositivo recurrente generado y listo para mostrar");
                self.trusted_qr_url = Some(url);
                self.trusted_qr_handle = Some(handle);
                return self.update(Message::ManageTrustedDevices);
            }
            Message::DeleteTrustedDevice(id) => {
                if let Some(db) = &self.db {
                    let db = db.clone();
                    return Task::perform(
                        async move { db.delete_trusted_device(id).await.map_err(|e| e.to_string()) },
                        Message::TrustedDeviceDeleted,
                    );
                }
            }
            Message::TrustedDeviceDeleted(Ok(_)) => {
                return self.update(Message::ManageTrustedDevices);
            }
            Message::TrustedDeviceDeleted(Err(e)) => {
                self.error_message = Some(format!("Error eliminando dispositivo: {}", e));
            }
            Message::CloseTrustedDevices => {
                self.trusted_devices_form = None;
                self.trusted_qr_handle = None;
                self.trusted_qr_url = None;
            }
        }

        Task::none()
    }


    fn view(&self) -> Element<'_, Message> {
        match &self.view {
            AppView::Loading => {
                let content = if let Some(err) = &self.error_message {
                    container(
                        column![
                            text("⚠️ Error").size(24),
                            text(err).size(14),
                        ]
                        .spacing(10)
                        .align_x(iced::Alignment::Center),
                    )
                } else {
                    container(
                        column![
                            text("📚 Comic").size(32),
                            text("Conectando a la base de datos...").size(14),
                        ]
                        .spacing(10)
                        .align_x(iced::Alignment::Center),
                    )
                };

                content
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|_theme: &Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.20, 0.20, 0.22,
                        ))),
                        ..Default::default()
                    })
                    .into()
            }

            AppView::Main => {
                let sidebar = ui::sidebar::view(
                    &self.collections,
                    self.selected_collection_id,
                    &self.new_collection_name,
                    self.show_new_collection,
                    self.server_running,
                    self.context_menu_collection,
                );

                let main_content = if let Some(err) = &self.error_message {
                    container(
                        column![
                            text("⚠️ Error de Conexión").size(24),
                            text(err).size(14),
                            text("Asegúrate de que el archivo comic.db no esté bloqueado").size(12),
                        ]
                        .spacing(10)
                        .align_x(iced::Alignment::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                } else if self.selected_collection_id.is_some() {
                    ui::comic_grid::view(&self.comics, &self.selected_collection_name, &self.comic_handles, self.is_scanning)
                } else {
                    let logo_icon = include_bytes!("../assets/wow-icon.png");
                    let logo_handle = image::Handle::from_bytes(logo_icon.as_slice());

                    container(
                        column![
                            image(logo_handle).width(100).height(100),
                            text("COMIC").size(32).color(iced::Color::WHITE),
                            text("Selecciona o crea una coleccion").size(18).color(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                            text("Usa el panel izquierdo para gestionar tus colecciones").size(13).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                        ]
                        .spacing(15)
                        .align_x(iced::Alignment::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                };

                let main_area = container(main_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_theme: &Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.20, 0.20, 0.22,
                        ))),
                        ..Default::default()
                    });

                let layout: Element<Message> = if self.is_sidebar_open {
                    row![sidebar, main_area].height(Length::Fill).into()
                } else {
                    main_area.into()
                };

                let root = column![
                    self.top_bar(),
                    layout,
                ];

                // Overlay editor or QR if active
                if let Some(form) = &self.editing_form {
                    return ui::metadata_editor::view(form);
                }

                if let Some(form) = &self.collection_editor_form {
                    return ui::collection_editor::view(form);
                }
                
                if self.show_qr {
                    let qr_overlay = self.qr_overlay();
                    stack![root, qr_overlay].into()
                } else if let Some(form) = &self.trusted_devices_form {
                    let td_overlay = ui::trusted_devices::view(form, self.trusted_qr_handle.as_ref(), self.trusted_qr_url.as_deref());
                    stack![root, td_overlay].into()
                } else {
                    root.into()
                }
            }

            AppView::Reader => {
                let title = self
                    .reading_comic
                    .as_ref()
                    .map(|c| c.title.as_str())
                    .unwrap_or("Comic");

                ui::reader::view(
                    self.page_handle.as_ref(),
                    self.current_page,
                    self.total_pages,
                    title,
                    self.is_loading_page,
                    self.zoom,
                    self.pan,
                    self.is_dragging,
                    self.show_reader_controls,
                )
            }
        }
    }

    // === Helpers ===

    fn reload_collections(&self) -> Task<Message> {
        if let Some(db) = &self.db {
            let db = db.clone();
            Task::perform(
                async move { db.get_collections().await.unwrap_or_default() },
                Message::CollectionsLoaded,
            )
        } else {
            Task::none()
        }
    }

    fn scan_collection_paths(&self) -> Task<Message> {
        if let (Some(db), Some(collection_id)) = (&self.db, self.selected_collection_id) {
            let db = db.clone();
            Task::perform(
                async move {
                    let paths = db
                        .get_collection_paths(collection_id)
                        .await
                        .map_err(|e| e.to_string())?;

                    let mut new_comics = Vec::new();
                    for cp in &paths {
                        let scanned = comic_reader::scan_directory(&cp.path);
                        for sc in scanned {
                            let exists = db
                                .comic_exists_by_path(&sc.file_path)
                                .await
                                .unwrap_or(true);
                            if !exists {
                                let page_count = comic_reader::get_page_count(&sc.file_path);
                                let cover = comic_reader::extract_cover(&sc.file_path);
                                new_comics.push(Comic {
                                    id: 0,
                                    collection_id,
                                    title: sc.file_name,
                                    file_path: sc.file_path,
                                    file_type: sc.format.as_str().to_string(),
                                    year: None,
                                    issue_number: None,
                                    saga: None,
                                    cover_data: cover,
                                    page_count: page_count as i32,
                                });
                            }
                        }
                    }
                    Ok(new_comics)
                },
                Message::ScanComplete,
            )
        } else {
            Task::none()
        }
    }

    fn top_bar(&self) -> Element<'_, Message> {
        let menu_icon = button(
            text("≡")
                .size(28)
                .font(iced::Font::with_name("Segoe UI"))
        )
        .padding(10)
        .on_press(Message::ToggleSidebar)
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
            text_color: iced::Color::WHITE,
            ..Default::default()
        });

        let clock = text(&self.current_time)
            .size(16)
            .color(iced::Color::from_rgb(0.7, 0.7, 0.7));

        let title = text("My Comics")
            .size(20)
            .font(iced::Font::with_name("Segoe UI Semibold"))
            .color(iced::Color::WHITE);

        container(
            row![
                clock,
                iced::widget::Space::with_width(Length::Fill),
                title,
                iced::widget::Space::with_width(Length::Fill),
                menu_icon,
            ]
            .padding([10, 20])
            .align_y(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .height(60)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.14))),
            ..Default::default()
        })
        .into()
    }

    fn qr_overlay(&self) -> Element<'_, Message> {
        let url_text = self.server_url.as_deref().unwrap_or("...");

        let mut content = column![
            row![
                text("Compartir Coleccion").size(22),
                Space::with_width(Length::Fill),
                button(
                    svg(svg::Handle::from_memory(include_bytes!("../assets/close-circle-svgrepo-com.svg").as_slice()))
                        .width(20)
                        .height(20)
                        .style(|_theme: &Theme, _status| svg::Style {
                            color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                        })
                )
                .on_press(Message::CloseQR)
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                    ..Default::default()
                }),
            ]
            .align_y(Alignment::Center),
            text("Escanea el codigo QR desde tu dispositivo").size(14),
        ]
        .spacing(10)
        .align_x(iced::Alignment::Center);

        if let Some(handle) = &self.qr_handle {
            content = content.push(
                container(image(handle.clone()).width(256).height(256))
                    .padding(10)
                    .style(|_theme: &Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::WHITE)),
                        border: iced::Border {
                            radius: 12.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            );
        }

        content = content.push(text(url_text).size(13));

        content = content.push(
            iced::widget::button(text("Cerrar").size(14))
                .padding([8, 20])
                .on_press(Message::CloseQR)
                .style(|_theme: &Theme, _status| iced::widget::button::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.91, 0.27, 0.37,
                    ))),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        );

        container(
            container(content.spacing(15).padding(30))
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.1, 0.1, 0.18,
                    ))),
                    border: iced::Border {
                        radius: 16.0.into(),
                        color: iced::Color::from_rgb(0.2, 0.2, 0.3),
                        width: 1.0,
                    },
                    ..Default::default()
                }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.0, 0.0, 0.0, 0.7,
            ))),
            ..Default::default()
        })
        .into()
    }
}

/// Use native Windows folder picker dialog
async fn rfd_pick_folder() -> Option<String> {
    // Use a simple stdin-based approach since rfd requires main thread
    // For now, use a hardcoded dialog approach via tokio::task::spawn_blocking
    tokio::task::spawn_blocking(|| {
        // Use Windows COM dialog via PowerShell
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                r#"
                Add-Type -AssemblyName System.Windows.Forms
                $dialog = New-Object System.Windows.Forms.FolderBrowserDialog
                $dialog.Description = 'Selecciona la carpeta con comics'
                $dialog.ShowNewFolderButton = $false
                $result = $dialog.ShowDialog()
                if ($result -eq [System.Windows.Forms.DialogResult]::OK) {
                    Write-Output $dialog.SelectedPath
                }
                "#,
            ])
            .output()
            .ok()?;

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            None
        } else {
            Some(path)
        }
    })
    .await
    .ok()
    .flatten()
}

/// Use native Windows file picker dialog for images
async fn pick_image() -> Option<String> {
    tokio::task::spawn_blocking(|| {
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                r#"
                Add-Type -AssemblyName System.Windows.Forms
                $dialog = New-Object System.Windows.Forms.OpenFileDialog
                $dialog.Filter = 'Imágenes|*.jpg;*.jpeg;*.png;*.webp;*.bmp'
                $dialog.Title = 'Selecciona una imagen para la colección'
                $result = $dialog.ShowDialog()
                if ($result -eq [System.Windows.Forms.DialogResult]::OK) {
                    Write-Output $dialog.FileName
                }
                "#,
            ])
            .output()
            .ok()?;

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            None
        } else {
            Some(path)
        }
    })
    .await
    .ok()
    .flatten()
}
/// Generate a QR code as an iced image Handle
fn generate_qr(url: &str) -> Option<iced::widget::image::Handle> {
    if let Some((rgba, w, h)) = crate::qr::generate_qr_image(url, 256) {
        Some(iced::widget::image::Handle::from_rgba(w, h, rgba))
    } else {
        None
    }
}

/// Use native Windows dialog to prompt for a string
async fn prompt_name() -> Option<String> {
    tokio::task::spawn_blocking(|| {
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                r#"
                Add-Type -AssemblyName Microsoft.VisualBasic
                $name = [Microsoft.VisualBasic.Interaction]::InputBox('Ingresa un nombre para el nuevo dispositivo:', 'Añadir Dispositivo Recurrente', '')
                if ($name) {
                    Write-Output $name
                }
                "#,
            ])
            .output()
            .ok()?;

        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    })
    .await
    .ok()
    .flatten()
}
