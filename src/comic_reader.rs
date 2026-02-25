use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::{info, warn, debug};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq)]
pub enum ComicFormat {
    Cbz,
    Cbr,
    Pdf,
}

impl ComicFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "cbz" | "zip" => Some(ComicFormat::Cbz),
            "cbr" | "rar" => Some(ComicFormat::Cbr),
            "pdf" => Some(ComicFormat::Pdf),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ComicFormat::Cbz => "cbz",
            ComicFormat::Cbr => "cbr",
            ComicFormat::Pdf => "pdf",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScannedComic {
    pub file_path: String,
    pub file_name: String,
    pub format: ComicFormat,
}

/// Scan a directory for comic files
pub fn scan_directory(dir_path: &str) -> Vec<ScannedComic> {
    info!("[Scanner] Escaneando directorio: {}", dir_path);
    let mut comics = Vec::new();
    let mut total_files = 0;
    let mut skipped_files = Vec::new();

    for entry in WalkDir::new(dir_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        total_files += 1;
        let file_name_full = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(format) = ComicFormat::from_extension(ext) {
                let file_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                info!("[Scanner] Comic encontrado: '{}' (formato: {:?})", file_name, format);
                comics.push(ScannedComic {
                    file_path: path.to_string_lossy().to_string(),
                    file_name,
                    format,
                });
            } else {
                skipped_files.push(format!("{} (.{})", file_name_full, ext));
            }
        } else {
            skipped_files.push(format!("{} (sin extensión)", file_name_full));
        }
    }

    comics.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));

    info!("[Scanner] Resumen: {} archivos totales, {} comics detectados, {} ignorados",
        total_files, comics.len(), skipped_files.len());
    
    for s in &skipped_files {
        debug!("[Scanner] Archivo ignorado: {}", s);
    }

    comics
}

/// Check if a filename is an image
fn is_image_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".webp")
        || lower.ends_with(".gif")
        || lower.ends_with(".bmp")
}

/// Get sorted image entries from a CBZ file
fn get_cbz_image_entries(file_path: &str) -> Result<Vec<String>, String> {
    let file = File::open(file_path).map_err(|e| format!("Cannot open file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid CBZ: {}", e))?;

    let mut entries: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            archive.by_index(i).ok().and_then(|entry| {
                let name = entry.name().to_string();
                if entry.is_dir() || name.starts_with("__MACOSX") || name.starts_with('.') {
                    return None;
                }
                if is_image_file(&name) {
                    Some(name)
                } else {
                    None
                }
            })
        })
        .collect();

    entries.sort();
    Ok(entries)
}

/// Get sorted image entries from a CBR (RAR) file
fn get_cbr_image_entries(file_path: &str) -> Result<Vec<String>, String> {
    let archive = unrar::Archive::new(file_path)
        .open_for_listing()
        .map_err(|e| format!("Cannot open CBR: {:?}", e))?;

    let mut entries: Vec<String> = Vec::new();
    let mut cursor = archive;
    loop {
        match cursor.read_header() {
            Ok(Some(header)) => {
                let name = header.entry().filename.to_string_lossy().to_string();
                if !header.entry().is_directory() && is_image_file(&name) {
                    entries.push(name);
                }
                cursor = header.skip().map_err(|e| format!("CBR skip error: {:?}", e))?;
            }
            Ok(None) => break,
            Err(e) => {
                warn!("[CBR] Error leyendo header: {:?}", e);
                break;
            }
        }
    }

    entries.sort();
    Ok(entries)
}

/// Extract a page from a CBR (RAR) file by reading into memory
fn get_cbr_page(file_path: &str, page_num: usize) -> Option<Vec<u8>> {
    let entries = get_cbr_image_entries(file_path).ok()?;
    let entry_name = entries.get(page_num)?.clone();

    let archive = unrar::Archive::new(file_path)
        .open_for_processing()
        .ok()?;

    let mut cursor = archive;
    loop {
        match cursor.read_header() {
            Ok(Some(header)) => {
                let name = header.entry().filename.to_string_lossy().to_string();
                if name == entry_name {
                    let (data, _rest) = header.read().ok()?;

                    if page_num == 0 {
                        // Create thumbnail for covers
                        if let Ok(img) = image::load_from_memory(&data) {
                            let thumb = img.thumbnail(300, 450);
                            let mut buf = std::io::Cursor::new(Vec::new());
                            if thumb.write_to(&mut buf, image::ImageFormat::Jpeg).is_ok() {
                                return Some(buf.into_inner());
                            }
                        }
                    }
                    return Some(data);
                }
                cursor = header.skip().ok()?;
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }
    None
}

/// Get the full-resolution page from a CBR file
fn get_cbr_full_page(file_path: &str, page_num: usize) -> Option<Vec<u8>> {
    let entries = get_cbr_image_entries(file_path).ok()?;
    let entry_name = entries.get(page_num)?.clone();

    let archive = unrar::Archive::new(file_path)
        .open_for_processing()
        .ok()?;

    let mut cursor = archive;
    loop {
        match cursor.read_header() {
            Ok(Some(header)) => {
                let name = header.entry().filename.to_string_lossy().to_string();
                if name == entry_name {
                    let (data, _rest) = header.read().ok()?;
                    return Some(data);
                }
                cursor = header.skip().ok()?;
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }
    None
}

/// Extract the cover image (first page) from a comic file
pub fn extract_cover(file_path: &str) -> Option<Vec<u8>> {
    get_page(file_path, 0)
}

/// Get a specific page from a comic file (0-indexed)
pub fn get_page(file_path: &str, page_num: usize) -> Option<Vec<u8>> {
    let path = Path::new(file_path);
    let ext = path.extension()?.to_str()?;
    let format = ComicFormat::from_extension(ext)?;

    match format {
        ComicFormat::Cbz => get_cbz_page(file_path, page_num),
        ComicFormat::Cbr => get_cbr_page(file_path, page_num),
        ComicFormat::Pdf => None,
    }
}

/// Extract a page from a CBZ file
fn get_cbz_page(file_path: &str, page_num: usize) -> Option<Vec<u8>> {
    let entries = get_cbz_image_entries(file_path).ok()?;
    let entry_name = entries.get(page_num)?;

    let file = File::open(file_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let mut entry = archive.by_name(entry_name).ok()?;

    let mut data = Vec::new();
    entry.read_to_end(&mut data).ok()?;

    // Create thumbnail for covers
    if page_num == 0 {
        if let Ok(img) = image::load_from_memory(&data) {
            let thumb = img.thumbnail(300, 450);
            let mut buf = std::io::Cursor::new(Vec::new());
            if thumb
                .write_to(&mut buf, image::ImageFormat::Jpeg)
                .is_ok()
            {
                return Some(buf.into_inner());
            }
        }
    }

    Some(data)
}

/// Get the full-resolution page from a comic
pub fn get_full_page(file_path: &str, page_num: usize) -> Option<Vec<u8>> {
    let path = Path::new(file_path);
    let ext = path.extension()?.to_str()?;
    let format = ComicFormat::from_extension(ext)?;

    match format {
        ComicFormat::Cbz => {
            let entries = get_cbz_image_entries(file_path).ok()?;
            let entry_name = entries.get(page_num)?;

            let file = File::open(file_path).ok()?;
            let mut archive = zip::ZipArchive::new(file).ok()?;
            let mut entry = archive.by_name(entry_name).ok()?;

            let mut data = Vec::new();
            entry.read_to_end(&mut data).ok()?;
            Some(data)
        }
        ComicFormat::Cbr => get_cbr_full_page(file_path, page_num),
        ComicFormat::Pdf => None,
    }
}

/// Count the number of pages in a comic
pub fn get_page_count(file_path: &str) -> usize {
    let path = Path::new(file_path);
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) => e,
        None => return 0,
    };

    match ComicFormat::from_extension(ext) {
        Some(ComicFormat::Cbz) => get_cbz_image_entries(file_path)
            .map(|e| e.len())
            .unwrap_or(0),
        Some(ComicFormat::Cbr) => get_cbr_image_entries(file_path)
            .map(|e| e.len())
            .unwrap_or(0),
        Some(ComicFormat::Pdf) => 0,
        None => 0,
    }
}
