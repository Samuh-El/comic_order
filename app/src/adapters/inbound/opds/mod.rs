//! Adaptador de entrada: Protocolo estándar OPDS 1.2 en XML Atom.

pub mod feed_builder;

use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tracing::{error, info};
use crate::adapters::inbound::rest_api::auth::{auth_middleware, ServerState};

const OPDS_NAV_MIME: &str = "application/atom+xml;profile=opds-catalog;kind=navigation;charset=utf-8";
const OPDS_ACQ_MIME: &str = "application/atom+xml;profile=opds-catalog;kind=acquisition;charset=utf-8";

/// Construye las rutas del feed OPDS 1.2 protegidas por autenticación de token.
pub fn create_opds_router(state: ServerState) -> Router {
    Router::new()
        .route("/", get(root_catalog))
        .route("/collections/:id", get(collection_catalog))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

/// GET `/opds` — Catálogo raíz de navegación OPDS 1.2.
pub async fn root_catalog(State(state): State<ServerState>) -> impl IntoResponse {
    info!("[OPDS] GET /opds - Solicitando catálogo raíz");
    match state.facade.get_collections().await {
        Ok(collections) => {
            let xml = feed_builder::build_root_catalog(&collections, Some(&state.session_token));
            Response::builder()
                .header(header::CONTENT_TYPE, HeaderValue::from_static(OPDS_NAV_MIME))
                .body(axum::body::Body::from(xml))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            error!("[OPDS] Error generando catálogo raíz: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET `/opds/collections/:id` — Catálogo de adquisición de colección OPDS 1.2.
pub async fn collection_catalog(
    State(state): State<ServerState>,
    Path(collection_id): Path<i64>,
) -> impl IntoResponse {
    info!("[OPDS] GET /opds/collections/{}", collection_id);
    match state.facade.get_collection_by_id(collection_id).await {
        Ok(Some(col)) => match state.facade.get_comics_by_collection(collection_id).await {
            Ok(comics) => {
                let xml = feed_builder::build_collection_catalog(&col, &comics, Some(&state.session_token));
                Response::builder()
                    .header(header::CONTENT_TYPE, HeaderValue::from_static(OPDS_ACQ_MIME))
                    .body(axum::body::Body::from(xml))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
            Err(e) => {
                error!("[OPDS] Error obteniendo cómics de colección {}: {}", collection_id, e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!("[OPDS] Error obteniendo colección {}: {}", collection_id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
