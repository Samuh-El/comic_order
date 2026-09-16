//! Manejadores de rutas REST para colecciones, cómics, streaming de páginas y progreso.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use crate::adapters::inbound::rest_api::auth::ServerState;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub max_width: Option<u32>,
    pub t: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProgressQuery {
    pub device_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProgressBody {
    pub device_id: String,
    pub current_page: usize,
}

#[derive(Debug, Serialize)]
pub struct CollectionDto {
    pub id: i64,
    pub name: String,
    pub has_icon: bool,
}

#[derive(Debug, Serialize)]
pub struct ComicDto {
    pub id: i64,
    pub title: String,
    pub file_type: String,
    pub year: Option<i32>,
    pub issue_number: Option<i32>,
    pub saga: Option<String>,
    pub page_count: usize,
    pub has_cover: bool,
}

#[derive(Debug, Serialize)]
pub struct ProgressDto {
    pub comic_id: i64,
    pub current_page: usize,
    pub completed: bool,
}

/// GET `/api/collections` — Listado de colecciones activas.
pub async fn list_collections(State(state): State<ServerState>) -> impl IntoResponse {
    info!("[HTTP] GET /api/collections");
    match state.facade.get_collections().await {
        Ok(collections) => {
            let data: Vec<CollectionDto> = collections
                .into_iter()
                .map(|c| CollectionDto {
                    id: c.id,
                    name: c.name,
                    has_icon: c.icon_data.is_some(),
                })
                .collect();
            Json(data).into_response()
        }
        Err(e) => {
            error!("[HTTP] Error obteniendo colecciones: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

/// GET `/api/collections/:id/icon` — Icono binario personalizado de la colección.
pub async fn get_collection_icon(
    State(state): State<ServerState>,
    Path(collection_id): Path<i64>,
) -> impl IntoResponse {
    info!("[HTTP] GET /api/collections/{}/icon", collection_id);
    match state.facade.get_collection_by_id(collection_id).await {
        Ok(Some(col)) => {
            if let Some(data) = col.icon_data {
                let content_type = if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                    "image/png"
                } else {
                    "image/jpeg"
                };

                Response::builder()
                    .header("Content-Type", content_type)
                    .header("Cache-Control", "public, max-age=3600")
                    .body(axum::body::Body::from(data))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!("[HTTP] Error obteniendo icono de colección {}: {}", collection_id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET `/api/collections/:id/comics` — Listado de volúmenes en la colección.
pub async fn list_comics(
    State(state): State<ServerState>,
    Path(collection_id): Path<i64>,
) -> impl IntoResponse {
    info!("[HTTP] GET /api/collections/{}/comics", collection_id);
    match state.facade.get_comics_by_collection(collection_id).await {
        Ok(comics) => {
            let data: Vec<ComicDto> = comics
                .into_iter()
                .map(|c| ComicDto {
                    id: c.id,
                    title: c.title,
                    file_type: c.format.as_str().to_string(),
                    year: c.year,
                    issue_number: c.issue_number,
                    saga: c.saga,
                    page_count: c.page_count,
                    has_cover: c.cover_data.is_some(),
                })
                .collect();
            Json(data).into_response()
        }
        Err(e) => {
            error!("[HTTP] Error listando cómics de colección {}: {}", collection_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

/// GET `/api/comics/:id/cover` — Portada en miniatura (300x450 JPEG).
pub async fn get_cover(
    State(state): State<ServerState>,
    Path(comic_id): Path<i64>,
) -> impl IntoResponse {
    info!("[HTTP] GET /api/comics/{}/cover", comic_id);
    match state.facade.get_cover_image(comic_id).await {
        Ok(cover) => {
            Response::builder()
                .header("Content-Type", "image/jpeg")
                .header("Cache-Control", "public, max-age=3600")
                .body(axum::body::Body::from(cover))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            error!("[HTTP] Error obteniendo portada del cómic {}: {}", comic_id, e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

/// GET `/api/comics/:id/page/:page?max_width=800` — Streaming selectivo de página bajo demanda.
pub async fn get_comic_page(
    State(state): State<ServerState>,
    Path((comic_id, page)): Path<(i64, usize)>,
    Query(query): Query<PageQuery>,
) -> impl IntoResponse {
    info!("[HTTP] GET /api/comics/{}/page/{} (max_width: {:?})", comic_id, page, query.max_width);
    match state.facade.get_page_image(comic_id, page, query.max_width).await {
        Ok(data) => {
            let content_type = if data.starts_with(&[0xFF, 0xD8]) {
                "image/jpeg"
            } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                "image/png"
            } else {
                "image/jpeg"
            };

            Response::builder()
                .header("Content-Type", content_type)
                .header("Cache-Control", "public, max-age=86400")
                .body(axum::body::Body::from(data))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            error!("[HTTP] Error obteniendo página {} del cómic {}: {}", page, comic_id, e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

/// GET `/api/comics/:id/progress?device_id=xyz` — Consulta del progreso de lectura.
pub async fn get_progress(
    State(state): State<ServerState>,
    Path(comic_id): Path<i64>,
    Query(query): Query<ProgressQuery>,
) -> impl IntoResponse {
    info!("[HTTP] GET /api/comics/{}/progress para dispositivo {}", comic_id, query.device_id);
    match state.facade.get_reading_progress(comic_id, &query.device_id).await {
        Ok(Some(progress)) => Json(ProgressDto {
            comic_id: progress.comic_id,
            current_page: progress.current_page,
            completed: progress.completed,
        })
        .into_response(),
        Ok(None) => Json(ProgressDto {
            comic_id,
            current_page: 0,
            completed: false,
        })
        .into_response(),
        Err(e) => {
            error!("[HTTP] Error obteniendo progreso de cómic {}: {}", comic_id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// POST `/api/comics/:id/progress` — Guardar progreso de lectura desde dispositivo cliente.
pub async fn save_progress(
    State(state): State<ServerState>,
    Path(comic_id): Path<i64>,
    Json(body): Json<UpdateProgressBody>,
) -> impl IntoResponse {
    info!("[HTTP] POST /api/comics/{}/progress pág {}", comic_id, body.current_page);
    match state
        .facade
        .save_reading_progress(comic_id, &body.device_id, body.current_page)
        .await
    {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))).into_response(),
        Err(e) => {
            error!("[HTTP] Error guardando progreso: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET `/api/icons/:name` — Entrega de iconos estáticos para la interfaz web.
pub async fn get_icon(Path(name): Path<String>) -> impl IntoResponse {
    let data = match name.as_str() {
        "layer-icon.png" => Some(include_bytes!("../../../../assets/layer-icon.png").as_slice()),
        "pause-round-icon.png" => Some(include_bytes!("../../../../assets/pause-round-icon.png").as_slice()),
        "book-icon.png" => Some(include_bytes!("../../../../assets/book-icon.png").as_slice()),
        "reading-icon.png" => Some(include_bytes!("../../../../assets/reading-icon.png").as_slice()),
        _ => None,
    };

    if let Some(content) = data {
        Response::builder()
            .header("Content-Type", "image/png")
            .header("Cache-Control", "public, max-age=86400")
            .body(axum::body::Body::from(content.to_vec()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
