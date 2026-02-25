use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tracing::{info, error};


use crate::db::Database;
use crate::comic_reader;

#[derive(Clone)]
pub struct ServerState {
    pub db: Database,
}

/// Create the Axum router with all API endpoints
pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/api/collections", get(list_collections))
        .route("/api/collections/:id/comics", get(list_comics))
        .route("/api/comics/:id/cover", get(get_cover))
        .route("/api/comics/:id/page/:page", get(get_comic_page))
        .with_state(state)
}

async fn index_page() -> Html<String> {
    info!("[HTTP] GET / - Sirviendo pagina web del lector");
    Html(WEB_PAGE.to_string())
}

const WEB_PAGE: &str = r#"<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Comic Reader</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0f0f0f; color: #e0e0e0; min-height: 100vh;
        }
        .header { 
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            padding: 1rem 2rem; border-bottom: 2px solid #e94560;
            display: flex; align-items: center; gap: 1rem;
        }
        .header h1 { font-size: 1.5rem; color: #e94560; }
        .header .back-btn { 
            background: none; border: 1px solid #e94560; color: #e94560;
            padding: 0.5rem 1rem; border-radius: 8px; cursor: pointer;
            display: none; font-size: 0.9rem;
        }
        .header .back-btn:hover { background: #e94560; color: white; }
        .container { padding: 2rem; max-width: 1200px; margin: 0 auto; }
        .grid { 
            display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
            gap: 1.5rem; 
        }
        .card {
            background: #1a1a2e; border-radius: 12px; overflow: hidden;
            transition: transform 0.2s, box-shadow 0.2s; cursor: pointer;
            border: 1px solid #222;
        }
        .card:hover { transform: translateY(-4px); box-shadow: 0 8px 24px rgba(233,69,96,0.2); }
        .card img { width: 100%; height: 240px; object-fit: cover; }
        .card .info { padding: 0.75rem; }
        .card .info h3 { font-size: 0.85rem; margin-bottom: 0.25rem; color: #fff; }
        .card .info p { font-size: 0.75rem; color: #888; }
        .collection-card { 
            background: linear-gradient(135deg, #1a1a2e, #16213e);
            padding: 2rem; text-align: center; display: flex;
            flex-direction: column; align-items: center; justify-content: center;
            min-height: 200px;
        }
        .collection-card h3 { font-size: 1.1rem; color: #e94560; }
        .collection-card .icon { font-size: 3rem; margin-bottom: 1rem; }
        .reader { 
            display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%;
            background: #000; z-index: 100; flex-direction: column; align-items: center;
        }
        .reader.active { display: flex; }
        .reader img { max-width: 100%; max-height: calc(100vh - 60px); object-fit: contain; }
        .reader-controls { 
            display: flex; gap: 1rem; padding: 0.75rem; background: #111;
            width: 100%; justify-content: center; align-items: center;
        }
        .reader-controls button {
            background: #e94560; border: none; color: white; padding: 0.5rem 1.5rem;
            border-radius: 8px; cursor: pointer; font-size: 1rem;
        }
        .reader-controls button:hover { background: #d63851; }
        .reader-controls span { color: #888; }
        .placeholder-cover {
            width: 100%; height: 240px; background: linear-gradient(135deg, #2a2a3e, #1a1a2e);
            display: flex; align-items: center; justify-content: center;
            font-size: 3rem; color: #444;
        }
        .section-title { font-size: 1.2rem; margin-bottom: 1.5rem; color: #e94560; }
        .loading { text-align: center; padding: 3rem; color: #888; font-size: 1.2rem; }
        .error { text-align: center; padding: 3rem; color: #e94560; font-size: 1rem; }
    </style>
</head>
<body>
    <div class="header">
        <button class="back-btn" id="backBtn" onclick="goBack()">Volver</button>
        <h1>Comic Reader</h1>
    </div>
    <div class="container" id="content"><div class="loading">Cargando...</div></div>
    <div class="reader" id="reader">
        <div style="flex:1; display:flex; align-items:center; justify-content:center; overflow:auto;">
            <img id="readerImage" src="" alt="Comic Page">
        </div>
        <div class="reader-controls">
            <button onclick="prevPage()">Anterior</button>
            <span id="pageInfo">1 / 1</span>
            <button onclick="nextPage()">Siguiente</button>
            <button onclick="closeReader()">Cerrar</button>
        </div>
    </div>
    <script>
        let currentView = 'collections';
        let currentComicId = null;
        let currentPage = 0;
        let totalPages = 0;

        async function loadCollections() {
            currentView = 'collections';
            document.getElementById('backBtn').style.display = 'none';
            const content = document.getElementById('content');
            content.innerHTML = '<div class="loading">Cargando colecciones...</div>';
            
            try {
                console.log('[Comic] Fetching /api/collections ...');
                const res = await fetch('/api/collections');
                console.log('[Comic] Response status:', res.status);
                if (!res.ok) throw new Error('HTTP ' + res.status);
                const collections = await res.json();
                console.log('[Comic] Collections received:', collections.length);

                if (collections.length === 0) {
                    content.innerHTML = '<div class="loading">No hay colecciones. Crea una en la app de escritorio.</div>';
                    return;
                }

                let html = '<h2 class="section-title">Colecciones</h2><div class="grid">';
                for (const c of collections) {
                    const safeName = escapeHtml(c.name);
                    html += '<div class="card collection-card" data-id="' + c.id + '" data-name="' + safeName + '">';
                    html += '<div class="icon">&#128218;</div>';
                    html += '<h3>' + safeName + '</h3>';
                    html += '</div>';
                }
                html += '</div>';
                content.innerHTML = html;

                // Attach click handlers via event delegation
                content.querySelectorAll('.collection-card').forEach(card => {
                    card.addEventListener('click', function() {
                        const id = parseInt(this.getAttribute('data-id'));
                        const name = this.getAttribute('data-name');
                        console.log('[Comic] Collection clicked: id=' + id + ' name=' + name);
                        loadComics(id, name);
                    });
                });
            } catch (err) {
                console.error('[Comic] Error loading collections:', err);
                content.innerHTML = '<div class="error">Error cargando colecciones: ' + err.message + '</div>';
            }
        }

        async function loadComics(collectionId, name) {
            currentView = 'comics';
            document.getElementById('backBtn').style.display = 'block';
            const content = document.getElementById('content');
            content.innerHTML = '<div class="loading">Cargando comics...</div>';

            try {
                const url = '/api/collections/' + collectionId + '/comics';
                console.log('[Comic] Fetching', url);
                const res = await fetch(url);
                console.log('[Comic] Response status:', res.status);
                if (!res.ok) throw new Error('HTTP ' + res.status);
                const comics = await res.json();
                console.log('[Comic] Comics received:', comics.length);

                if (comics.length === 0) {
                    content.innerHTML = '<h2 class="section-title">' + escapeHtml(name) + '</h2>' +
                        '<div class="loading">No hay comics en esta coleccion.</div>';
                    return;
                }

                let html = '<h2 class="section-title">' + escapeHtml(name) + '</h2><div class="grid">';
                for (const c of comics) {
                    html += '<div class="card" data-comic-id="' + c.id + '" data-pages="' + c.page_count + '">';
                    if (c.has_cover) {
                        html += '<img src="/api/comics/' + c.id + '/cover" alt="' + escapeHtml(c.title) + '" loading="lazy">';
                    } else {
                        html += '<div class="placeholder-cover">&#128214;</div>';
                    }
                    html += '<div class="info">';
                    html += '<h3>' + escapeHtml(c.title) + '</h3>';
                    let meta = c.file_type.toUpperCase();
                    if (c.year) meta = (c.issue_number ? '#' + c.issue_number + ' - ' : '') + c.year;
                    html += '<p>' + meta + '</p>';
                    html += '</div></div>';
                }
                html += '</div>';
                content.innerHTML = html;

                // Attach click handlers
                content.querySelectorAll('.card[data-comic-id]').forEach(card => {
                    card.addEventListener('click', function() {
                        const comicId = parseInt(this.getAttribute('data-comic-id'));
                        const pages = parseInt(this.getAttribute('data-pages'));
                        console.log('[Comic] Opening comic id=' + comicId + ' pages=' + pages);
                        openReader(comicId, pages);
                    });
                });
            } catch (err) {
                console.error('[Comic] Error loading comics:', err);
                content.innerHTML = '<div class="error">Error cargando comics: ' + err.message + '</div>';
            }
        }

        function openReader(comicId, pages) {
            currentComicId = comicId;
            currentPage = 0;
            totalPages = pages;
            loadPage();
            document.getElementById('reader').classList.add('active');
        }

        function loadPage() {
            const url = '/api/comics/' + currentComicId + '/page/' + currentPage;
            console.log('[Comic] Loading page:', url);
            document.getElementById('readerImage').src = url;
            document.getElementById('pageInfo').textContent = (currentPage + 1) + ' / ' + totalPages;
        }

        function prevPage() { if (currentPage > 0) { currentPage--; loadPage(); } }
        function nextPage() { if (currentPage < totalPages - 1) { currentPage++; loadPage(); } }
        function closeReader() { document.getElementById('reader').classList.remove('active'); }
        function goBack() { loadCollections(); }

        function escapeHtml(str) {
            const div = document.createElement('div');
            div.appendChild(document.createTextNode(str || ''));
            return div.innerHTML;
        }

        document.addEventListener('keydown', e => {
            if (!document.getElementById('reader').classList.contains('active')) return;
            if (e.key === 'ArrowLeft') prevPage();
            if (e.key === 'ArrowRight') nextPage();
            if (e.key === 'Escape') closeReader();
        });

        // Touch swipe support for mobile
        let touchStartX = 0;
        document.getElementById('reader').addEventListener('touchstart', e => {
            touchStartX = e.changedTouches[0].screenX;
        });
        document.getElementById('reader').addEventListener('touchend', e => {
            const diff = e.changedTouches[0].screenX - touchStartX;
            if (Math.abs(diff) > 50) {
                if (diff > 0) prevPage();
                else nextPage();
            }
        });

        loadCollections();
    </script>
</body>
</html>"#;

async fn list_collections(State(state): State<ServerState>) -> impl IntoResponse {
    info!("[HTTP] GET /api/collections");
    match state.db.get_collections().await {
        Ok(collections) => {
            info!("[HTTP] Respondiendo con {} colecciones", collections.len());
            let data: Vec<serde_json::Value> = collections
                .iter()
                .map(|c| serde_json::json!({ "id": c.id, "name": c.name }))
                .collect();
            Json(data).into_response()
        }
        Err(e) => {
            error!("[HTTP] Error listando colecciones: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

async fn list_comics(
    State(state): State<ServerState>,
    Path(collection_id): Path<i64>,
) -> impl IntoResponse {
    info!("[HTTP] GET /api/collections/{}/comics", collection_id);
    match state.db.get_comics_by_collection(collection_id).await {
        Ok(comics) => {
            info!("[HTTP] Respondiendo con {} comics para coleccion {}", comics.len(), collection_id);
            let data: Vec<serde_json::Value> = comics
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "title": c.title,
                        "file_type": c.file_type,
                        "year": c.year,
                        "issue_number": c.issue_number,
                        "saga": c.saga,
                        "page_count": c.page_count,
                        "has_cover": c.cover_data.is_some(),
                    })
                })
                .collect();
            Json(data).into_response()
        }
        Err(e) => {
            error!("[HTTP] Error listando comics para coleccion {}: {}", collection_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response()
        }
    }
}

async fn get_cover(
    State(state): State<ServerState>,
    Path(comic_id): Path<i64>,
) -> impl IntoResponse {
    info!("[HTTP] GET /api/comics/{}/cover", comic_id);
    match state.db.get_comic_by_id(comic_id).await {
        Ok(Some(comic)) => {
            if let Some(cover) = comic.cover_data {
                info!("[HTTP] Enviando portada de comic {} ({} bytes)", comic_id, cover.len());
                Response::builder()
                    .header("Content-Type", "image/jpeg")
                    .header("Cache-Control", "public, max-age=3600")
                    .body(axum::body::Body::from(cover))
                    .unwrap()
                    .into_response()
            } else {
                info!("[HTTP] Comic {} no tiene portada", comic_id);
                StatusCode::NOT_FOUND.into_response()
            }
        }
        Ok(None) => {
            info!("[HTTP] Comic {} no encontrado", comic_id);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(e) => {
            error!("[HTTP] Error obteniendo portada del comic {}: {}", comic_id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_comic_page(
    State(state): State<ServerState>,
    Path((comic_id, page)): Path<(i64, usize)>,
) -> impl IntoResponse {
    info!("[HTTP] GET /api/comics/{}/page/{}", comic_id, page);
    match state.db.get_comic_by_id(comic_id).await {
        Ok(Some(comic)) => {
            info!("[HTTP] Extrayendo pagina {} de '{}' ({})", page, comic.title, comic.file_path);
            if let Some(data) = comic_reader::get_full_page(&comic.file_path, page) {
                let content_type = if data.starts_with(&[0xFF, 0xD8]) {
                    "image/jpeg"
                } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                    "image/png"
                } else {
                    "image/jpeg"
                };

                info!("[HTTP] Enviando pagina {} ({} bytes, {})", page, data.len(), content_type);
                Response::builder()
                    .header("Content-Type", content_type)
                    .body(axum::body::Body::from(data))
                    .unwrap()
                    .into_response()
            } else {
                error!("[HTTP] No se pudo extraer pagina {} de comic {}", page, comic_id);
                StatusCode::NOT_FOUND.into_response()
            }
        }
        Ok(None) => {
            info!("[HTTP] Comic {} no encontrado", comic_id);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(e) => {
            error!("[HTTP] Error obteniendo pagina del comic {}: {}", comic_id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
