//! Comandos IPC de Tauri para la interfaz de usuario de Comic Showcase.

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

use crate::application::facade::ComicFacade;
use crate::domain::collection::Collection;

/// Estado compartido inyectado en el runtime de Tauri.
pub struct AppState {
    pub facade: Arc<ComicFacade>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionDto {
    pub id: i64,
    pub name: String,
    pub protagonist: String,
    pub description: String,
    pub background_image_path: Option<String>,
    pub hero_image_path: Option<String>,
    pub comics_path: Option<String>,
    pub background_image_base64: Option<String>,
    pub hero_image_base64: Option<String>,
    pub icon_base64: Option<String>,
    pub comic_count: usize,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveCollectionPayload {
    pub id: Option<i64>,
    pub name: String,
    pub protagonist: Option<String>,
    pub description: Option<String>,
    pub comics_path: Option<String>,
    pub background_image_path: Option<String>,
    pub hero_image_path: Option<String>,
    pub icon_data: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComicSummaryDto {
    pub id: i64,
    pub collection_id: i64,
    pub title: String,
    pub file_path: String,
    pub series: Option<String>,
    pub number: Option<i32>,
    pub year: Option<i32>,
    pub page_count: i32,
    pub has_cover: bool,
    pub cover_base64: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PageResponseDto {
    pub comic_id: i64,
    pub page_index: u32,
    pub total_pages: u32,
    pub image_data_base64: String,
}

#[derive(Debug, Serialize)]
pub struct ServerStatusDto {
    pub running: bool,
    pub port: u16,
    pub local_ip: String,
    pub url: String,
    pub qr_base64: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TrustedDeviceDto {
    pub id: i64,
    pub device_name: String,
    pub token: String,
    pub created_at: String,
}

/// Lee una imagen almacenada en disco y la devuelve como Data URL Base64
fn load_image_as_data_url(path_opt: Option<&str>) -> Option<String> {
    let path_str = path_opt?;
    let path = std::path::Path::new(path_str);

    let resolved_path = if path.exists() {
        path.to_path_buf()
    } else {
        let relative = std::path::Path::new("data").join(path_str);
        if relative.exists() {
            relative
        } else {
            return None;
        }
    };

    let ext = resolved_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        _ => "image/jpeg",
    };
    let bytes = std::fs::read(&resolved_path).ok()?;
    Some(format!("data:{};base64,{}", mime, simple_base64(&bytes)))
}

// === Comandos de Colecciones ===

#[tauri::command]
pub async fn get_collections(state: State<'_, AppState>) -> Result<Vec<CollectionDto>, String> {
    info!("[IPC] get_collections invocado");
    let collections = state.facade.get_collections().await.map_err(|e| e.to_string())?;

    let mut dtos = Vec::with_capacity(collections.len());
    for c in collections {
        let comics = state.facade.get_comics_by_collection(c.id).await.unwrap_or_default();
        let paths = state.facade.get_collection_paths(c.id).await.unwrap_or_default();
        let comics_path = paths.first().map(|p| p.path.to_string_lossy().replace('\\', "/"));
        let protagonist = c.protagonist_display().to_string();
        let icon_base64 = c.icon_data.as_ref().map(|b| {
            format!("data:image/jpeg;base64,{}", simple_base64(b))
        });
        let bg_b64 = load_image_as_data_url(c.background_image_path.as_deref());
        let hero_b64 = load_image_as_data_url(c.hero_image_path.as_deref());

        dtos.push(CollectionDto {
            id: c.id,
            name: c.name,
            protagonist,
            description: c.description.unwrap_or_default(),
            background_image_path: c.background_image_path,
            hero_image_path: c.hero_image_path,
            comics_path,
            background_image_base64: bg_b64,
            hero_image_base64: hero_b64,
            icon_base64,
            comic_count: comics.len(),
            created_at: c.created_at.format("%Y-%m-%d %H:%M").to_string(),
        });
    }

    Ok(dtos)
}

#[tauri::command]
pub async fn get_collection(id: i64, state: State<'_, AppState>) -> Result<Option<CollectionDto>, String> {
    info!("[IPC] get_collection id={}", id);
    let col = state.facade.get_collection_by_id(id).await.map_err(|e| e.to_string())?;
    if let Some(c) = col {
        let comics = state.facade.get_comics_by_collection(c.id).await.unwrap_or_default();
        let paths = state.facade.get_collection_paths(c.id).await.unwrap_or_default();
        let comics_path = paths.first().map(|p| p.path.to_string_lossy().replace('\\', "/"));
        let protagonist = c.protagonist_display().to_string();
        let bg_b64 = load_image_as_data_url(c.background_image_path.as_deref());
        let hero_b64 = load_image_as_data_url(c.hero_image_path.as_deref());
        Ok(Some(CollectionDto {
            id: c.id,
            name: c.name,
            protagonist,
            description: c.description.unwrap_or_default(),
            background_image_path: c.background_image_path,
            hero_image_path: c.hero_image_path,
            comics_path,
            background_image_base64: bg_b64,
            hero_image_base64: hero_b64,
            icon_base64: None,
            comic_count: comics.len(),
            created_at: c.created_at.format("%Y-%m-%d %H:%M").to_string(),
        }))
    } else {
        Ok(None)
    }
}

/// Copia una imagen local a la carpeta de almacenamiento de medios de la aplicación y normaliza las barras diagonales.
fn copy_collection_media(src: &str, prefix: &str) -> String {
    let src_clean = src.replace('\\', "/");
    let src_path = std::path::Path::new(&src_clean);
    if !src_path.exists() {
        return src_clean;
    }
    if src_clean.contains("data/media/collections") {
        if let Ok(abs) = std::fs::canonicalize(src_path) {
            let mut s = abs.to_string_lossy().to_string();
            if s.starts_with(r"\\?\") {
                s = s[4..].to_string();
            }
            return s.replace('\\', "/");
        }
        return src_clean;
    }

    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let target_dir = std::path::PathBuf::from("data/media/collections");
    let _ = std::fs::create_dir_all(&target_dir);
    let target_name = format!("{}_{}.{}", prefix, uuid::Uuid::new_v4(), ext);
    let target_path = target_dir.join(target_name);

    if std::fs::copy(src_path, &target_path).is_ok() {
        if let Ok(abs) = std::fs::canonicalize(&target_path) {
            let mut s = abs.to_string_lossy().to_string();
            if s.starts_with(r"\\?\") {
                s = s[4..].to_string();
            }
            return s.replace('\\', "/");
        }
        return target_path.to_string_lossy().replace('\\', "/");
    }
    src_clean
}

#[tauri::command]
pub async fn save_collection(
    payload: SaveCollectionPayload,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    info!("[IPC] save_collection: '{}' protagonist: '{:?}'", payload.name, payload.protagonist);

    let desc: Option<String> = payload
        .description
        .as_deref()
        .filter(|d| !d.trim().is_empty())
        .map(|d| d.chars().take(600).collect());
    let prot: Option<String> = payload
        .protagonist
        .as_deref()
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.chars().take(25).collect());
    let bg = payload
        .background_image_path
        .as_deref()
        .filter(|b| !b.trim().is_empty())
        .map(|b| copy_collection_media(b, "bg"));
    let hero = payload
        .hero_image_path
        .as_deref()
        .filter(|h| !h.trim().is_empty())
        .map(|h| copy_collection_media(h, "hero"));

    let target_id = if let Some(id) = payload.id {
        if id > 0 {
            let col = Collection {
                id,
                name: payload.name.trim().to_string(),
                protagonist: prot,
                description: desc,
                background_image_path: bg,
                hero_image_path: hero,
                icon_data: payload.icon_data,
                created_at: chrono::Utc::now(),
            };
            state.facade.update_collection(&col).await.map_err(|e| e.to_string())?;
            id
        } else {
            let new_id = state
                .facade
                .create_collection_with_details(
                    &payload.name,
                    prot.as_deref(),
                    desc.as_deref(),
                    bg.as_deref(),
                    hero.as_deref(),
                )
                .await
                .map_err(|e| e.to_string())?;

            if let Some(icon) = payload.icon_data {
                let _ = state.facade.set_collection_icon(new_id, &icon).await;
            }
            new_id
        }
    } else {
        let new_id = state
            .facade
            .create_collection_with_details(
                &payload.name,
                prot.as_deref(),
                desc.as_deref(),
                bg.as_deref(),
                hero.as_deref(),
            )
            .await
            .map_err(|e| e.to_string())?;

        if let Some(icon) = payload.icon_data {
            let _ = state.facade.set_collection_icon(new_id, &icon).await;
        }
        new_id
    };

    // Asociar ruta/carpeta de cómics e indexar inmediatamente si se proporcionó
    if let Some(comics_path) = payload.comics_path {
        let trimmed = comics_path.trim();
        if !trimmed.is_empty() {
            let normalized = trimmed.replace('\\', "/");
            let _ = state.facade.add_collection_path(target_id, &normalized).await;

            let worker = crate::ingestion::worker::IngestionWorker::new((*state.facade).clone(), None);
            let count = worker.scan_collection(target_id).await.unwrap_or(0);
            info!("[IPC] Colección {} indexada en '{}'. Cómics encontrados: {}", target_id, normalized, count);
        }
    }

    Ok(target_id)
}

#[tauri::command]
pub async fn delete_collection(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    info!("[IPC] delete_collection id={}", id);
    state.facade.delete_collection(id).await.map_err(|e| e.to_string())
}

// === Comandos de Cómics y Lectura ===

#[tauri::command]
pub async fn get_comics_by_collection(
    collection_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ComicSummaryDto>, String> {
    info!("[IPC] get_comics_by_collection id={}", collection_id);
    let comics = state
        .facade
        .get_comics_by_collection(collection_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(comics
        .into_iter()
        .map(|c| {
            let has_cover = c.cover_data.is_some();
            let cover_base64 = c.cover_data.as_ref().map(|b| {
                format!("data:image/jpeg;base64,{}", simple_base64(b))
            });
            ComicSummaryDto {
                id: c.id,
                collection_id: c.collection_id,
                title: c.title,
                file_path: c.file_path.to_string_lossy().to_string(),
                series: c.saga,
                number: c.issue_number,
                year: c.year,
                page_count: c.page_count as i32,
                has_cover,
                cover_base64,
            }
        })
        .collect())
}

#[tauri::command]
pub async fn get_comic_page(
    comic_id: i64,
    page_index: u32,
    max_width: Option<u32>,
    state: State<'_, AppState>,
) -> Result<PageResponseDto, String> {
    let comic = state
        .facade
        .get_comic_by_id(comic_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Cómic no encontrado".to_string())?;

    let total_pages = comic.page_count as u32;
    let page_bytes = state
        .facade
        .get_page_image(comic_id, page_index as usize, max_width)
        .await
        .map_err(|e| e.to_string())?;

    let b64 = format!("data:image/jpeg;base64,{}", simple_base64(&page_bytes));

    Ok(PageResponseDto {
        comic_id,
        page_index,
        total_pages,
        image_data_base64: b64,
    })
}

#[tauri::command]
pub async fn scan_monitored_paths(state: State<'_, AppState>) -> Result<usize, String> {
    info!("[IPC] scan_monitored_paths disparado");
    let collections = state.facade.get_collections().await.map_err(|e| e.to_string())?;
    let mut total_found = 0;
    let worker = crate::ingestion::worker::IngestionWorker::new((*state.facade).clone(), None);
    for col in collections {
        let found = worker.scan_collection(col.id).await.unwrap_or_default();
        total_found += found;
    }
    info!("[IPC] Escaneo finalizado. {} cómics procesados", total_found);
    Ok(total_found)
}

// === Comandos de Servidor y Dispositivos de Confianza ===

#[tauri::command]
pub async fn get_server_status(_state: State<'_, AppState>) -> Result<ServerStatusDto, String> {
    let local_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let url = format!("http://{}:8080", local_ip);

    let qr_base64 = crate::network::qr::generate_qr_svg(&url);

    Ok(ServerStatusDto {
        running: true,
        port: 8080,
        local_ip,
        url,
        qr_base64: Some(qr_base64),
    })
}

#[tauri::command]
pub async fn get_trusted_devices(state: State<'_, AppState>) -> Result<Vec<TrustedDeviceDto>, String> {
    let devices = state.facade.get_trusted_devices().await.map_err(|e| e.to_string())?;
    Ok(devices
        .into_iter()
        .map(|d| TrustedDeviceDto {
            id: d.id,
            device_name: d.device_name,
            token: d.token,
            created_at: d.created_at.format("%Y-%m-%d %H:%M").to_string(),
        })
        .collect())
}

#[tauri::command]
pub async fn add_trusted_device(
    device_name: String,
    state: State<'_, AppState>,
) -> Result<TrustedDeviceDto, String> {
    let dev = state
        .facade
        .add_trusted_device(&device_name)
        .await
        .map_err(|e| e.to_string())?;

    Ok(TrustedDeviceDto {
        id: dev.id,
        device_name: dev.device_name,
        token: dev.token,
        created_at: dev.created_at.format("%Y-%m-%d %H:%M").to_string(),
    })
}

#[tauri::command]
pub async fn remove_trusted_device(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state.facade.delete_trusted_device(id).await.map_err(|e| e.to_string())
}

// === Selector de Archivo Nativo y Carpetas ===

#[tauri::command]
pub async fn pick_folder() -> Result<Option<String>, String> {
    tokio::task::spawn_blocking(|| {
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                r#"
                Add-Type -AssemblyName System.Windows.Forms
                $dialog = New-Object System.Windows.Forms.FolderBrowserDialog
                $dialog.Description = 'Selecciona la carpeta donde están los cómics (.cbz, .cbr)'
                $dialog.ShowNewFolderButton = $false
                $result = $dialog.ShowDialog()
                if ($result -eq [System.Windows.Forms.DialogResult]::OK) {
                    Write-Output $dialog.SelectedPath
                }
                "#,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            Ok(None)
        } else {
            let normalized = path.replace('\\', "/");
            Ok(Some(normalized))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pick_image_file() -> Result<Option<String>, String> {
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
            .map_err(|e| e.to_string())?;

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            Ok(None)
        } else {
            let normalized = path.replace('\\', "/");
            Ok(Some(normalized))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Helper para codificar Base64 estándar
fn simple_base64(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len() * 4 / 3 + 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        result.push(CHARSET[(b0 >> 2) as usize] as char);
        result.push(CHARSET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARSET[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARSET[(b2 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
