//! Servicio de aplicación para la lectura, streaming de páginas y progreso de lectura.

use std::sync::Arc;
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use crate::domain::errors::DomainError;
use crate::domain::ports::{ComicFileReader, ComicRepository, ImageCache, ProgressRepository};
use crate::domain::progress::ReadingProgress;

/// Modos de lectura soportados por la estrategia de visualización (Patrón Estrategia).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadingMode {
    SinglePage,
    DoublePage,
    WebtoonContinuous,
}

impl Default for ReadingMode {
    fn default() -> Self {
        ReadingMode::SinglePage
    }
}

/// Servicio encargado de la extracción selectiva de páginas, escalado y persistencia de avance.
#[derive(Clone)]
pub struct ReaderService {
    comic_repo: Arc<dyn ComicRepository>,
    progress_repo: Arc<dyn ProgressRepository>,
    cache: Arc<dyn ImageCache>,
    readers: Vec<Arc<dyn ComicFileReader>>,
}

impl ReaderService {
    pub fn new(
        comic_repo: Arc<dyn ComicRepository>,
        progress_repo: Arc<dyn ProgressRepository>,
        cache: Arc<dyn ImageCache>,
        readers: Vec<Arc<dyn ComicFileReader>>,
    ) -> Self {
        Self {
            comic_repo,
            progress_repo,
            cache,
            readers,
        }
    }

    /// Obtiene la imagen de portada de un cómic, prefiriendo la miniatura almacenada.
    pub async fn get_cover(&self, comic_id: i64) -> Result<Vec<u8>, DomainError> {
        let comic = self
            .comic_repo
            .get_by_id(comic_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Cómic con ID {}", comic_id)))?;

        if let Some(cover) = comic.cover_data {
            return Ok(cover);
        }

        // Si no tenía miniatura en BD, extraer la primera página
        self.get_page(comic_id, 0, Some(300)).await
    }

    /// Obtiene una página específica de un cómic, consultando la caché o extrayéndola al vuelo.
    pub async fn get_page(
        &self,
        comic_id: i64,
        page_index: usize,
        max_width: Option<u32>,
    ) -> Result<Vec<u8>, DomainError> {
        // 1. Intentar servir desde el proxy de caché en disco
        if let Some(cached_data) = self.cache.get_cached(comic_id, page_index, max_width).await {
            return Ok(cached_data);
        }

        // 2. Localizar el cómic en el repositorio
        let comic = self
            .comic_repo
            .get_by_id(comic_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Cómic con ID {}", comic_id)))?;

        let file_path = comic.file_path.clone();
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let reader = self
            .readers
            .iter()
            .find(|r| r.can_handle(&ext))
            .ok_or_else(|| DomainError::UnsupportedFormat(ext.clone()))?
            .clone();

        // 3. Extraer la página física en tarea bloqueante sin congelar el runtime
        let raw_data = tokio::task::spawn_blocking(move || {
            reader.extract_page(&file_path, page_index)
        })
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))??;

        // 4. Si se solicitó redimensionado (ej. streaming móvil), escalar la imagen
        let final_data = if let Some(width) = max_width {
            tokio::task::spawn_blocking(move || {
                if let Ok(img) = image::load_from_memory(&raw_data) {
                    if img.width() > width {
                        let scaled = img.resize(width, u32::MAX, image::imageops::FilterType::Lanczos3);
                        let mut buf = std::io::Cursor::new(Vec::new());
                        if scaled.write_to(&mut buf, ImageFormat::Jpeg).is_ok() {
                            return buf.into_inner();
                        }
                    }
                }
                raw_data
            })
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
        } else {
            raw_data
        };

        // 5. Guardar asíncronamente en el proxy de caché en disco para futuras solicitudes
        let _ = self
            .cache
            .store(comic_id, page_index, max_width, &final_data)
            .await;

        Ok(final_data)
    }

    /// Consulta el avance de lectura registrado para un cómic y dispositivo.
    pub async fn get_progress(
        &self,
        comic_id: i64,
        device_id: &str,
    ) -> Result<Option<ReadingProgress>, DomainError> {
        Ok(self.progress_repo.get_progress(comic_id, device_id).await?)
    }

    /// Guarda o actualiza el avance de lectura para un cómic y dispositivo.
    pub async fn save_progress(
        &self,
        comic_id: i64,
        device_id: &str,
        page: usize,
    ) -> Result<(), DomainError> {
        let comic = self
            .comic_repo
            .get_by_id(comic_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Cómic con ID {}", comic_id)))?;

        let completed = comic.page_count > 0 && page >= comic.page_count - 1;
        let progress = ReadingProgress::new(comic_id, device_id.to_string(), page, completed);
        Ok(self.progress_repo.save_progress(&progress).await?)
    }
}
