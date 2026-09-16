//! Adaptador para lectura de cómics empaquetados en formato ZIP / CBZ.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use image::ImageFormat;
use crate::domain::ports::ComicFileReader;
use crate::domain::errors::ComicReaderError;

/// Adaptador para archivos de formato CBZ / ZIP.
#[derive(Debug, Default, Clone)]
pub struct CbzAdapter;

impl CbzAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Comprueba si el nombre de archivo corresponde a una extensión gráfica reconocida.
    fn is_image_entry(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".png")
            || lower.ends_with(".webp")
            || lower.ends_with(".gif")
            || lower.ends_with(".bmp")
    }

    /// Obtiene las entradas ordenadas alfanuméricamente que correspondan a imágenes dentro del archivo ZIP.
    fn get_image_entries(&self, path: &Path) -> Result<Vec<String>, ComicReaderError> {
        let file = File::open(path).map_err(|e| ComicReaderError::FileNotFound(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ComicReaderError::CorruptArchive(e.to_string()))?;

        let mut entries: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                archive.by_index(i).ok().and_then(|entry| {
                    let name = entry.name().to_string();
                    if entry.is_dir() || name.starts_with("__MACOSX") || name.starts_with('.') {
                        return None;
                    }
                    if Self::is_image_entry(&name) {
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
}

impl ComicFileReader for CbzAdapter {
    fn can_handle(&self, extension: &str) -> bool {
        let ext = extension.to_lowercase();
        ext == "cbz" || ext == "zip"
    }

    fn get_page_count(&self, path: &Path) -> Result<usize, ComicReaderError> {
        let entries = self.get_image_entries(path)?;
        Ok(entries.len())
    }

    fn extract_cover(&self, path: &Path) -> Result<Vec<u8>, ComicReaderError> {
        let raw_page = self.extract_page(path, 0)?;

        // Generar miniatura JPEG 300x450 directamente en memoria
        if let Ok(img) = image::load_from_memory(&raw_page) {
            let thumb = img.thumbnail(300, 450);
            let mut buf = std::io::Cursor::new(Vec::new());
            if thumb.write_to(&mut buf, ImageFormat::Jpeg).is_ok() {
                return Ok(buf.into_inner());
            }
        }

        Ok(raw_page)
    }

    fn extract_page(&self, path: &Path, page_index: usize) -> Result<Vec<u8>, ComicReaderError> {
        let entries = self.get_image_entries(path)?;
        if entries.is_empty() {
            return Err(ComicReaderError::NoImagesFound);
        }

        let entry_name = entries
            .get(page_index)
            .ok_or(ComicReaderError::PageOutOfRange(page_index))?;

        let file = File::open(path).map_err(|e| ComicReaderError::FileNotFound(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ComicReaderError::CorruptArchive(e.to_string()))?;

        let mut entry = archive
            .by_name(entry_name)
            .map_err(|e| ComicReaderError::CorruptArchive(e.to_string()))?;

        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .map_err(ComicReaderError::IoError)?;

        Ok(data)
    }

    fn extract_comic_info(&self, path: &Path) -> Result<Option<String>, ComicReaderError> {
        let file = File::open(path).map_err(|e| ComicReaderError::FileNotFound(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ComicReaderError::CorruptArchive(e.to_string()))?;

        for i in 0..archive.len() {
            if let Ok(mut entry) = archive.by_index(i) {
                let name = entry.name().to_string();
                if name.ends_with("ComicInfo.xml") && !name.starts_with("__MACOSX") {
                    let mut content = String::new();
                    if entry.read_to_string(&mut content).is_ok() {
                        return Ok(Some(content));
                    }
                }
            }
        }

        Ok(None)
    }
}
