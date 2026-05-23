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
    pub token: String,
}

/// Create the Axum router with all API endpoints
pub fn create_router(state: ServerState) -> Router {
    use axum::middleware::from_fn_with_state;
    use tower_http::cors::{Any, CorsLayer};

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        .route("/collections", get(list_collections))
        .route("/collections/:id/icon", get(get_collection_icon))
        .route("/collections/:id/comics", get(list_comics))
        .route("/comics/:id/cover", get(get_cover))
        .route("/comics/:id/page/:page", get(get_comic_page))
        .route("/icons/:name", get(get_icon))
        .route_layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state.clone());

    Router::new()
        .route("/", get(index_page))
        .route("/ping", get(|| async { "pong" }))
        .nest("/api", api_routes)
        .layer(tower_http::limit::RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(axum::middleware::from_fn(security_headers))
        .layer(cors)
}

async fn security_headers(req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Content-Security-Policy", "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:;".parse().unwrap());
    response
}

async fn auth_middleware(
    State(state): State<ServerState>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let uri = req.uri().to_string();
    let method = req.method().to_string();
    
    // 1. Check Authorization header
    let auth_header = req.headers().get(axum::http::header::AUTHORIZATION);
    
    info!("[CONN] Requerido: {} {}. Header Auth: {:?}", method, uri, auth_header.is_some());

    let mut token_to_check = None;

    if let Some(header_val) = auth_header.and_then(|h| h.to_str().ok()) {
        if let Some(token) = header_val.strip_prefix("Bearer ") {
            token_to_check = Some(token.to_string());
        }
    }

    // 2. Check query parameter 't'
    if token_to_check.is_none() {
        if let Some(query) = req.uri().query() {
            let params: Vec<&str> = query.split('&').collect();
            for param in params {
                if let Some(val) = param.strip_prefix("token=") {
                    token_to_check = Some(val.to_string());
                    break;
                }
                if let Some(val) = param.strip_prefix("t=") {
                    token_to_check = Some(val.to_string());
                    break;
                }
            }
        }
    }

    if let Some(token) = token_to_check {
        // Validate against session token
        if token == state.token {
            info!("[AUTH] OK: Token de sesion validado para {}", uri);
            return Ok(next.run(req).await);
        }

        // Validate against database (trusted devices)
        match state.db.is_token_trusted(&token).await {
            Ok(true) => {
                info!("[AUTH] OK: Token RECURRENTE validado para {}", uri);
                return Ok(next.run(req).await);
            }
            Ok(false) => {
                error!("[AUTH] FAIL: Token no reconocido en session ni trusted DB: {}", token);
            }
            Err(e) => {
                error!("[AUTH] ERROR consultando trusted devices: {}", e);
            }
        }
    }

    error!("[AUTH] DENY: No se proporcionó token válido para {}. Header: {:?}", uri, auth_header);
    Err(StatusCode::UNAUTHORIZED)
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
        :root {
            --bg: #0a0a0c;
            --surface: #16161a;
            --primary: #ff3e5e;
            --primary-hover: #ff5e7a;
            --text: #e1e1e6;
            --text-dim: #a1a1aa;
            --glass: rgba(22, 22, 26, 0.7);
        }
        * { margin: 0; padding: 0; box-sizing: border-box; -webkit-tap-highlight-color: transparent; }
        body { 
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--bg); color: var(--text); min-height: 100vh;
            line-height: 1.5; overflow-x: hidden;
        }
        .header { 
            background: var(--glass);
            backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px);
            padding: 1.2rem 1.5rem; border-bottom: 1px solid rgba(255, 62, 94, 0.2);
            display: flex; align-items: center; gap: 1rem;
            position: sticky; top: 0; z-index: 50;
        }
        .header h1 { font-size: 1.25rem; font-weight: 700; color: var(--primary); letter-spacing: -0.5px; }
        .header .back-btn { 
            background: rgba(255, 62, 94, 0.1); border: 1px solid var(--primary); color: var(--primary);
            padding: 0.5rem 0.8rem; border-radius: 10px; cursor: pointer;
            display: none; font-size: 0.85rem; font-weight: 600;
            transition: all 0.2s;
        }
        .header .back-btn:active { transform: scale(0.95); background: var(--primary); color: white; }
        
        .container { padding: 1.5rem; max-width: 1400px; margin: 0 auto; animation: fadeInUp 0.4s ease-out; }
        @keyframes fadeInUp {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
        }

        .section-title { font-size: 1rem; font-weight: 600; margin-bottom: 1.25rem; color: var(--text-dim); text-transform: uppercase; letter-spacing: 1px; }

        .grid { 
            display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
            gap: 1.25rem; 
        }
        @media (max-width: 480px) {
            .grid { grid-template-columns: repeat(2, 1fr); gap: 1rem; }
            .container { padding: 1rem; }
        }

        .card {
            background: var(--surface); border-radius: 16px; overflow: hidden;
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); cursor: pointer;
            border: 1px solid rgba(255,255,255,0.05);
            position: relative;
        }
        .card:active { transform: scale(0.97); }
        .card img { width: 100%; height: 220px; object-fit: cover; transition: transform 0.5s; }
        .card:hover img { transform: scale(1.05); }
        
        .card .info { 
            padding: 0.75rem; background: linear-gradient(to top, var(--surface) 80%, transparent);
            position: absolute; bottom: 0; width: 100%;
        }
        .card .info h3 { font-size: 0.85rem; font-weight: 600; margin-bottom: 0.15rem; color: #fff; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        .card .info p { font-size: 0.7rem; color: var(--text-dim); }

        .collection-card { 
            background: linear-gradient(145deg, #1e1e24, #16161a);
            padding: 1.5rem; text-align: center; display: flex;
            flex-direction: column; align-items: center; justify-content: center;
            min-height: 160px; border: 1px solid rgba(255, 62, 94, 0.15);
            position: relative;
        }
        .collection-card:hover {
            transform: translateY(-5px);
            border-color: var(--primary);
            background: linear-gradient(145deg, #25252d, #1c1c21);
            box-shadow: 0 10px 20px rgba(255, 62, 94, 0.1);
        }
        .collection-card .icon-img { transition: transform 0.3s; }
        .collection-card:hover .icon-img { transform: scale(1.1); }
        .collection-card h3 { color: var(--primary); font-size: 1rem; margin-top: 0.5rem; }

        .reader { 
            display: none; position: fixed; top: 0; left: 0; width: 100%; height: 100%;
            background: #000; z-index: 100; flex-direction: column;
        }
        .reader.active { display: flex; }
        .reader-viewport { 
            flex: 1; display: flex; align-items: center; justify-content: center; 
            overflow: auto; background: #000; position: relative;
        }
        .reader img { max-width: 100%; max-height: 100%; object-fit: contain; }
        
        .reader-controls { 
            display: flex; gap: 0.5rem; padding: 1rem; background: rgba(10, 10, 12, 0.9);
            backdrop-filter: blur(10px); border-top: 1px solid rgba(255,255,255,0.05);
            width: 100%; justify-content: space-between; align-items: center;
        }
        .reader-btn {
            background: rgba(255,255,255,0.05); border: none; color: white; padding: 0.7rem 1.2rem;
            border-radius: 12px; cursor: pointer; font-size: 0.9rem; font-weight: 600;
            transition: all 0.2s; flex: 1; max-width: 120px;
        }
        .reader-btn.primary { background: var(--primary); }
        .reader-btn:active { transform: scale(0.9); }
        #pageInfo { font-size: 0.85rem; color: var(--text-dim); font-variant-numeric: tabular-nums; }

        .loading { text-align: center; padding: 4rem; color: var(--text-dim); }
        .error { 
            text-align: center; padding: 2rem; background: rgba(255, 62, 94, 0.1); 
            border-radius: 12px; border: 1px solid var(--primary); color: var(--primary);
            margin: 2rem; font-size: 0.9rem;
        }
    </style>
</head>
<body>
    <div class="header" style="position: relative; z-index: 10;">
        <button class="back-btn" id="backBtn" onclick="goBack()">Volver</button>
        <h1 id="mainTitle">Comic App</h1>
        <button onclick="clearSession()" style="background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.2); color: white; padding: 6px 12px; font-size: 12px; border-radius: 8px; margin-left: auto; cursor: pointer; pointer-events: auto;">Limpiar</button>
    </div>
    <div class="container" id="content"></div>
    <div id="debugInfo" style="display:none; position:fixed; bottom: 80px; left: 10px; right: 10px; background: rgba(0,0,0,0.95); color: #0f0; font-family: monospace; font-size: 11px; padding: 15px; border: 1px solid #0f0; z-index: 2000; max-height: 60vh; overflow: auto; border-radius: 12px; box-shadow: 0 0 30px rgba(0,0,0,0.5);"></div>
    <button onclick="toggleDebug()" style="position:fixed; bottom: 20px; left: 20px; border:1px solid rgba(255,255,255,0.3); background: rgba(0,0,0,0.5); backdrop-filter: blur(5px); color: white; border-radius: 50%; width: 50px; height: 50px; z-index: 2000; cursor: pointer; font-weight: bold; pointer-events: auto;">ID</button>
    <div class="reader" id="reader">
        <div class="reader-viewport">
            <img id="readerImage" src="" alt="Comic Page">
        </div>
        <div class="reader-controls">
            <button class="reader-btn" onclick="prevPage()">Ant.</button>
            <span id="pageInfo">1 / 1</span>
            <button class="reader-btn" onclick="nextPage()">Sig.</button>
            <button class="reader-btn primary" onclick="closeReader()">Salir</button>
        </div>
    </div>
    <script>
        let state = {
            currentView: 'collections',
            currentComicId: null,
            currentPage: 0,
            totalPages: 0,
            token: null
        };

        function init() {
            const urlParams = new URLSearchParams(window.location.search);
            const urlToken = urlParams.get('token');
            
            if (urlToken) {
                state.token = urlToken;
                localStorage.setItem('comic_token', urlToken);
            } else {
                state.token = localStorage.getItem('comic_token');
            }

            if (!state.token || state.token === 'null') {
                document.getElementById('content').innerHTML = `
                    <div class="error">
                        <p style="font-size: 1.2rem; margin-bottom: 1rem;">Sesión no autorizada</p>
                        <p>Por favor, escanea el código QR de nuevo desde la aplicación de escritorio.</p>
                    </div>`;
                return;
            }

            // Automatic prompt removed in refactored flow
            loadCollections();
        }

        async function saveAsRecurring() {
            // Deprecated: Recurring devices are now added from the desktop app via specific QR
        }

        async function apiFetch(url) {
            log("Fetch: " + url);
            try {
                const res = await fetch(url, {
                    headers: { 'Authorization': 'Bearer ' + state.token }
                });
                log("Response: " + url + " [" + res.status + "]");
                
                if (res.status === 401) {
                    errorLog("ERROR 401: Token invalido");
                    throw new Error('401: No autorizado. El token puede ser viejo.');
                }
                if (!res.ok) throw new Error('Status Error: ' + res.status);
                return res;
            } catch (err) {
                errorLog("Fetch Error: " + err.message);
                throw err;
            }
        }

        function log(msg) {
            console.log(msg);
            const di = document.getElementById('debugInfo');
            const entry = document.createElement('div');
            entry.style.borderBottom = "1px solid #1a1a1a";
            entry.style.padding = "2px 0";
            entry.innerHTML = `<span style="color:#888">[${new Date().toLocaleTimeString()}]</span> ${msg}`;
            di.prepend(entry);
        }

        function errorLog(msg) {
            console.error(msg);
            log(`<span style="color:#ff3e5e">ERROR: ${msg}</span>`);
        }

        async function loadCollections() {
            console.log("loadCollections: Iniciando sincronizacion");
            state.currentView = 'collections';
            document.getElementById('backBtn').style.display = 'none';
            const content = document.getElementById('content');
            content.innerHTML = '<div class="loading">Sincronizando colecciones...</div>';
            window.scrollTo({ top: 0, behavior: 'smooth' });
            
            try {
                const res = await apiFetch('/api/collections');
                const collections = await res.json();
                console.log("loadCollections: Recibidas " + collections.length + " colecciones");

                if (collections.length === 0) {
                    content.innerHTML = '<div class="loading">No se encontraron colecciones activas.</div>';
                    return;
                }

                let html = '<h2 class="section-title">Mis Colecciones</h2><div class="grid">';
                for (const c of collections) {
                    const safeName = escapeHtml(c.name);
                    const iconUrl = c.has_icon 
                        ? `/api/collections/${c.id}/icon?token=${state.token}`
                        : `/api/icons/layer-icon.png?token=${state.token}`;
                    
                    html += `<div class="card collection-card" data-id="${c.id}" data-name="${safeName}">
                        <img src="${iconUrl}" class="icon-img" style="width: 80px; height: 80px; margin-bottom: 0.5rem; object-fit: contain; filter: drop-shadow(0 4px 8px rgba(255, 62, 94, 0.3)); border-radius: 8px;">
                        <h3>${safeName}</h3>
                    </div>`;
                }
                html += '</div>';
                content.innerHTML = html;

                content.querySelectorAll('.collection-card').forEach(card => {
                    card.onclick = () => loadComics(card.dataset.id, card.dataset.name);
                });
            } catch (err) {
                content.innerHTML = `
                    <div class="error">
                        <p style="font-size: 1.1rem; margin-bottom: 0.5rem;">Error de conexión</p>
                        <p style="font-size: 0.85rem; opacity: 0.8;">${err.message}</p>
                        <button onclick="loadCollections()" style="margin-top: 1rem; border: 1px solid white; background: transparent; color: white; padding: 0.5rem 1rem; border-radius: 8px;">Reintentar</button>
                    </div>`;
            }
        }

        async function loadComics(collectionId, name) {
            state.currentView = 'comics';
            document.getElementById('backBtn').style.display = 'block';
            const content = document.getElementById('content');
            content.innerHTML = '<div class="loading">Abriendo biblioteca...</div>';
            window.scrollTo({ top: 0, behavior: 'smooth' });

            try {
                const res = await apiFetch('/api/collections/' + collectionId + '/comics');
                const comics = await res.json();

                if (comics.length === 0) {
                    content.innerHTML = `<h2 class="section-title">${escapeHtml(name)}</h2><div class="loading">Esta colección está vacía.</div>`;
                    return;
                }

                let html = `<h2 class="section-title">${escapeHtml(name)}</h2><div class="grid">`;
                for (const c of comics) {
                    html += `<div class="card" data-comic-id="${c.id}" data-pages="${c.page_count}">
                        <img src="/api/comics/${c.id}/cover?t=${state.token}" alt="${escapeHtml(c.title)}" loading="lazy">
                        <div class="info">
                            <h3>${escapeHtml(c.title)}</h3>
                            <p>${c.year ? c.year : c.file_type.toUpperCase()}</p>
                        </div>
                    </div>`;
                }
                html += '</div>';
                content.innerHTML = html;

                content.querySelectorAll('.card[data-comic-id]').forEach(card => {
                    card.onclick = () => openReader(card.dataset.comicId, card.dataset.pages);
                });
            } catch (err) {
                content.innerHTML = `<div class="error">Error al cargar comics: ${err.message}</div>`;
            }
        }

        async function openReader(comicId, pages) {
            state.currentComicId = comicId;
            state.currentPage = 0;
            state.totalPages = parseInt(pages);
            
            loadPage();
            document.getElementById('reader').classList.add('active');
            document.body.style.overflow = 'hidden';
        }

        function loadPage() {
            const img = document.getElementById('readerImage');
            img.style.opacity = '0.5';
            const url = `/api/comics/${state.currentComicId}/page/${state.currentPage}?t=${state.token}`;
            img.src = url;
            img.onload = () => { img.style.opacity = '1'; };
            document.getElementById('pageInfo').textContent = (state.currentPage + 1) + ' / ' + state.totalPages;
            document.querySelector('.reader-viewport').scrollTop = 0;
        }

        function prevPage() { if (state.currentPage > 0) { state.currentPage--; loadPage(); } }
        function nextPage() { if (state.currentPage < state.totalPages - 1) { state.currentPage++; loadPage(); } }
        function closeReader() { 
            document.getElementById('reader').classList.remove('active'); 
            document.body.style.overflow = '';
        }
        function goBack() { loadCollections(); }
        
        function clearSession() {
            if(confirm("¿Limpiar sesión y caché?")) {
                localStorage.clear();
                window.location.href = "/";
            }
        }

        function toggleDebug() {
            const di = document.getElementById('debugInfo');
            di.style.display = di.style.display === 'none' ? 'block' : 'none';
            if (di.style.display === 'block') {
                di.innerHTML = `
                    IP: ${window.location.hostname}<br>
                    Token: ${state.token ? state.token.substring(0,6) + '...' : 'null'}<br>
                    UA: ${navigator.userAgent.substring(0,30)}...<br>
                    LocalTime: ${new Date().toLocaleTimeString()}
                `;
            }
        }

        function escapeHtml(str) {
            const div = document.createElement('div');
            div.textContent = str || '';
            return div.innerHTML;
        }

        document.addEventListener('keydown', e => {
            if (!document.getElementById('reader').classList.contains('active')) return;
            if (e.key === 'ArrowLeft') prevPage();
            if (e.key === 'ArrowRight') nextPage();
            if (e.key === 'Escape') closeReader();
        });

        let touchStartX = 0;
        const reader = document.getElementById('reader');
        reader.addEventListener('touchstart', e => { touchStartX = e.changedTouches[0].screenX; }, {passive: true});
        reader.addEventListener('touchend', e => {
            const diff = e.changedTouches[0].screenX - touchStartX;
            if (Math.abs(diff) > 60) { if (diff > 0) prevPage(); else nextPage(); }
        }, {passive: true});

        init();
    </script>
</body>
</html>"#;

async fn list_collections(State(state): State<ServerState>) -> impl IntoResponse {
    info!("[HTTP] GET /api/collections - Solicitando colecciones");
    match state.db.get_collections().await {
        Ok(collections) => {
            info!("[HTTP] OK: {} colecciones encontradas en la DB", collections.len());
            for c in &collections {
                info!("  - ID: {}, Nombre: {}", c.id, c.name);
            }
            let data: Vec<serde_json::Value> = collections
                .iter()
                .map(|c| serde_json::json!({ 
                    "id": c.id, 
                    "name": c.name,
                    "has_icon": c.icon_data.is_some()
                }))
                .collect();
            Json(data).into_response()
        }
        Err(e) => {
            error!("[HTTP] ERROR listando colecciones: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error de base de datos: {}", e)).into_response()
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

async fn get_icon(
    State(_state): State<ServerState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let data = match name.as_str() {
        "layer-icon.png" => Some(include_bytes!("../assets/layer-icon.png").as_slice()),
        "pause-round-icon.png" => Some(include_bytes!("../assets/pause-round-icon.png").as_slice()),
        "book-icon.png" => Some(include_bytes!("../assets/book-icon.png").as_slice()),
        "reading-icon.png" => Some(include_bytes!("../assets/reading-icon.png").as_slice()),
        _ => None,
    };

    if let Some(content) = data {
        Response::builder()
            .header("Content-Type", "image/png")
            .header("Cache-Control", "public, max-age=86400")
            .body(axum::body::Body::from(content.to_vec()))
            .unwrap()
            .into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn get_collection_icon(
    State(state): State<ServerState>,
    Path(collection_id): Path<i64>,
) -> impl IntoResponse {
    info!("[HTTP] GET /api/collections/{}/icon", collection_id);
    match state.db.get_collection_by_id(collection_id).await {
        Ok(Some(col)) => {
            if let Some(data) = col.icon_data {
                info!("[HTTP] Enviando icono de coleccion {} ({} bytes)", collection_id, data.len());
                let content_type = if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                    "image/png"
                } else if data.starts_with(&[0xFF, 0xD8]) {
                    "image/jpeg"
                } else {
                    "image/jpeg"
                };

                Response::builder()
                    .header("Content-Type", content_type)
                    .header("Cache-Control", "public, max-age=3600")
                    .body(axum::body::Body::from(data))
                    .unwrap()
                    .into_response()
            } else {
                info!("[HTTP] Coleccion {} no tiene icono personalizado", collection_id);
                StatusCode::NOT_FOUND.into_response()
            }
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!("[HTTP] Error obteniendo icono de coleccion {}: {}", collection_id, e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// trust_device endpoint removed in refactored flow
