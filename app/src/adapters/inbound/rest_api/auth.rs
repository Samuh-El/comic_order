//! Middleware de autenticación y cabeceras de seguridad para la API REST.

use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::{error, info};
use crate::application::facade::ComicFacade;

/// Estado compartido del servidor HTTP.
#[derive(Clone)]
pub struct ServerState {
    pub facade: ComicFacade,
    pub session_token: String,
}

/// Middleware que inyecta cabeceras de protección estándar (CSP, X-Frame-Options, etc.).
pub async fn security_headers(req: axum::extract::Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    if let Ok(val) = HeaderValue::from_str("nosniff") {
        headers.insert(header::X_CONTENT_TYPE_OPTIONS, val);
    }
    if let Ok(val) = HeaderValue::from_str("DENY") {
        headers.insert(header::X_FRAME_OPTIONS, val);
    }
    if let Ok(val) = HeaderValue::from_str("1; mode=block") {
        headers.insert(header::X_XSS_PROTECTION, val);
    }
    if let Ok(val) = HeaderValue::from_str("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:;") {
        headers.insert(header::CONTENT_SECURITY_POLICY, val);
    }

    response
}

/// Middleware que valida el token de autenticación contra la sesión en memoria o dispositivos de confianza.
pub async fn auth_middleware(
    State(state): State<ServerState>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let uri = req.uri().to_string();
    let method = req.method().to_string();

    let auth_header = req.headers().get(header::AUTHORIZATION);
    info!("[CONN] Requerido: {} {}. Header Auth: {:?}", method, uri, auth_header.is_some());

    let mut token_to_check = None;

    // 1. Extraer token de cabecera Authorization: Bearer <token>
    if let Some(header_val) = auth_header.and_then(|h| h.to_str().ok()) {
        if let Some(token) = header_val.strip_prefix("Bearer ") {
            token_to_check = Some(token.to_string());
        }
    }

    // 2. Extraer token desde query params (?token= o ?t=)
    if token_to_check.is_none() {
        if let Some(query) = req.uri().query() {
            for param in query.split('&') {
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
        // Validar contra el token de sesión volátil
        if token == state.session_token {
            info!("[AUTH] OK: Token de sesión validado para {}", uri);
            return Ok(next.run(req).await);
        }

        // Validar contra la base de datos de dispositivos de confianza
        match state.facade.is_token_trusted(&token).await {
            Ok(true) => {
                info!("[AUTH] OK: Token persistente recurrente validado para {}", uri);
                return Ok(next.run(req).await);
            }
            Ok(false) => {
                error!("[AUTH] FAIL: Token no reconocido en sesión ni dispositivos de confianza: {}", token);
            }
            Err(e) => {
                error!("[AUTH] ERROR consultando dispositivos de confianza: {}", e);
            }
        }
    }

    error!("[AUTH] DENY: Acceso no autorizado para {}. Header: {:?}", uri, auth_header);
    Err(StatusCode::UNAUTHORIZED)
}
