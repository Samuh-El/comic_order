//! Adaptador de entrada: Servidor Web HTTP y API REST (Axum).

pub mod auth;
pub mod routes;
pub mod spa;

pub use auth::ServerState;

use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::get,
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

/// Construye el enrutador HTTP Axum con todos los endpoints de la API, middlewares de seguridad y CORS.
pub fn create_router(state: ServerState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        .route("/collections", get(routes::list_collections))
        .route("/collections/:id/icon", get(routes::get_collection_icon))
        .route("/collections/:id/comics", get(routes::list_comics))
        .route("/comics/:id/cover", get(routes::get_cover))
        .route("/comics/:id/page/:page", get(routes::get_comic_page))
        .route("/comics/:id/progress", get(routes::get_progress).post(routes::save_progress))
        .route("/icons/:name", get(routes::get_icon))
        .route_layer(from_fn_with_state(state.clone(), auth::auth_middleware))
        .with_state(state.clone());

    let opds_routes = crate::adapters::inbound::opds::create_opds_router(state);

    Router::new()
        .route("/", get(spa::index_page))
        .route("/ping", get(|| async { "pong" }))
        .nest("/api", api_routes)
        .nest("/opds", opds_routes)
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(from_fn(auth::security_headers))
        .layer(cors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;
    use crate::application::catalog_service::CatalogService;
    use crate::application::facade::ComicFacade;
    use crate::application::reader_service::ReaderService;
    use crate::adapters::outbound::persistence::SqliteComicRepository;
    use crate::adapters::outbound::file_readers::{CbzAdapter, CbrAdapter};
    use crate::adapters::outbound::cache::DiskImageCache;
    use crate::domain::ports::ComicFileReader;

    async fn setup_test_facade() -> ComicFacade {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::adapters::outbound::persistence::schema::initialize_schema(&pool).await.unwrap();
        let repo = Arc::new(SqliteComicRepository::new(pool));
        let readers: Vec<Arc<dyn ComicFileReader>> = vec![Arc::new(CbzAdapter), Arc::new(CbrAdapter)];
        let cache = Arc::new(DiskImageCache::new("target/test_cache_api".into()));

        let catalog = CatalogService::new(repo.clone(), repo.clone(), readers.clone());
        let reader = ReaderService::new(repo.clone(), repo.clone(), cache, readers);

        ComicFacade::new(catalog, reader, repo)
    }

    #[tokio::test]
    async fn test_ping_endpoint() {
        let facade = setup_test_facade().await;
        let state = ServerState {
            facade,
            session_token: "test_token_123".to_string(),
        };
        let app = create_router(state);

        let req = Request::builder()
            .uri("/ping")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_rejects_missing_token() {
        let facade = setup_test_facade().await;
        let state = ServerState {
            facade,
            session_token: "secret123".to_string(),
        };
        let app = create_router(state);

        let req = Request::builder()
            .uri("/api/collections")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_accepts_valid_bearer_token() {
        let facade = setup_test_facade().await;
        let state = ServerState {
            facade,
            session_token: "secret123".to_string(),
        };
        let app = create_router(state);

        let req = Request::builder()
            .uri("/api/collections")
            .header("Authorization", "Bearer secret123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_accepts_valid_query_token() {
        let facade = setup_test_facade().await;
        let state = ServerState {
            facade,
            session_token: "secret123".to_string(),
        };
        let app = create_router(state);

        let req = Request::builder()
            .uri("/api/collections?t=secret123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
