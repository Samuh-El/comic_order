//! Adaptador para lectura de cómics empaquetados en formato RAR / CBR.

use std::path::Path;
use image::ImageFormat;
use tracing::warn;
use crate::domain::ports::ComicFileReader;
use crate::domain::errors::ComicReaderError;

/// Adaptador para archivos de formato CBR / RAR.
#[derive(Debug, Default, Clone)]
pub struct CbrAdapter;

impl CbrAdapter {
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

    /// Obtiene las entradas ordenadas alfanuméricamente que correspondan a imágenes dentro del archivo RAR.
    fn get_image_entries(&self, path: &Path) -> Result<Vec<String>, ComicReaderError> {
        let archive = unrar::Archive::new(path)
            .open_for_listing()
            .map_err(|e| ComicReaderError::CorruptArchive(format!("{:?}", e)))?;

        let mut entries: Vec<String> = Vec::new();
        let mut cursor = archive;
        loop {
            match cursor.read_header() {
                Ok(Some(header)) => {
                    let name = header.entry().filename.to_string_lossy().to_string();
                    if !header.entry().is_directory() && Self::is_image_entry(&name) {
                        entries.push(name);
                    }
                    match header.skip() {
                        Ok(next) => cursor = next,
                        Err(e) => {
                            warn!("[CBR] Error saltando entrada: {:?}", e);
                            break;
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    warn!("[CBR] Error leyendo cabecera: {:?}", e);
                    break;
                }
            }
        }

        entries.sort();
        Ok(entries)
    }
}

impl ComicFileReader for CbrAdapter {
    fn can_handle(&self, extension: &str) -> bool {
        let ext = extension.to_lowercase();
        ext == "cbr" || ext == "rar"
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

        let target_name = entries
            .get(page_index)
            .ok_or(ComicReaderError::PageOutOfRange(page_index))?
            .clone();

        let archive = unrar::Archive::new(path)
            .open_for_processing()
            .map_err(|e| ComicReaderError::CorruptArchive(format!("{:?}", e)))?;

        let mut cursor = archive;
        loop {
            match cursor.read_header() {
                Ok(Some(header)) => {
                    let name = header.entry().filename.to_string_lossy().to_string();
                    if name == target_name {
                        let (data, _) = header
                            .read()
                            .map_err(|e| ComicReaderError::CorruptArchive(format!("{:?}", e)))?;
                        return Ok(data);
                    }
                    match header.skip() {
                        Ok(next) => cursor = next,
                        Err(e) => return Err(ComicReaderError::CorruptArchive(format!("{:?}", e))),
                    }
                }
                Ok(None) => break,
                Err(e) => return Err(ComicReaderError::CorruptArchive(format!("{:?}", e))),
            }
        }

        Err(ComicReaderError::PageOutOfRange(page_index))
    }

    fn extract_comic_info(&self, path: &Path) -> Result<Option<String>, ComicReaderError> {
        let archive = match unrar::Archive::new(path).open_for_processing() {
            Ok(a) => a,
            Err(_) => return Ok(None),
        };

        let mut cursor = archive;
        loop {
            match cursor.read_header() {
                Ok(Some(header)) => {
                    let name = header.entry().filename.to_string_lossy().to_string();
                    if name.ends_with("ComicInfo.xml") && !name.starts_with("__MACOSX") {
                        if let Ok((data, _)) = header.read() {
                            if let Ok(content) = String::from_utf8(data) {
                                return Ok(Some(content));
                            }
                        }
                        break;
                    }
                    match header.skip() {
                        Ok(next) => cursor = next,
                        Err(_) => break,
                    }
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }

        Ok(None)
    }
}
