//! Adaptador de caché en disco para páginas y miniaturas procesadas bajo demanda.

use std::fs;
use std::path::PathBuf;
use async_trait::async_trait;
use tracing::{debug, warn};
use crate::domain::ports::ImageCache;

/// Implementación de ImageCache que persiste imágenes en disco bajo `app/.cache/`.
#[derive(Debug, Clone)]
pub struct DiskImageCache {
    cache_dir: PathBuf,
}

impl DiskImageCache {
    /// Inicializa la caché en la ruta provista, asegurando la existencia del directorio.
    pub fn new(cache_dir: PathBuf) -> Self {
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            warn!("[Cache] No se pudo crear el directorio de caché {:?}: {}", cache_dir, e);
        }
        Self { cache_dir }
    }

    /// Genera la ruta canónica del archivo de caché para un cómic, página y ancho dado.
    fn get_cache_path(&self, comic_id: i64, page: usize, max_width: Option<u32>) -> PathBuf {
        let file_name = match max_width {
            Some(w) => format!("c{}_p{}_w{}.img", comic_id, page, w),
            None => format!("c{}_p{}_orig.img", comic_id, page),
        };
        self.cache_dir.join(file_name)
    }
}

#[async_trait]
impl ImageCache for DiskImageCache {
    async fn get_cached(&self, comic_id: i64, page: usize, max_width: Option<u32>) -> Option<Vec<u8>> {
        let path = self.get_cache_path(comic_id, page, max_width);
        tokio::fs::read(&path).await.ok()
    }

    async fn store(&self, comic_id: i64, page: usize, max_width: Option<u32>, data: &[u8]) -> Result<(), std::io::Error> {
        let path = self.get_cache_path(comic_id, page, max_width);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        tokio::fs::write(&path, data).await
    }

    async fn prune(&self, max_bytes: u64) -> Result<u64, std::io::Error> {
        let cache_dir = self.cache_dir.clone();
        tokio::task::spawn_blocking(move || {
            let mut entries = Vec::new();
            let mut total_size = 0u64;

            if let Ok(read_dir) = fs::read_dir(&cache_dir) {
                for entry in read_dir.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() {
                            let len = meta.len();
                            total_size += len;
                            let modified = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                            entries.push((entry.path(), len, modified));
                        }
                    }
                }
            }

            let mut deleted_bytes = 0u64;
            if total_size > max_bytes {
                // Ordenar por tiempo de modificación ascendente (los más antiguos primero)
                entries.sort_by_key(|(_, _, modified)| *modified);

                for (path, len, _) in entries {
                    if total_size - deleted_bytes <= max_bytes {
                        break;
                    }
                    if fs::remove_file(&path).is_ok() {
                        deleted_bytes += len;
                        debug!("[Cache] Archivo eliminado por cuota: {:?}", path);
                    }
                }
            }

            Ok(deleted_bytes)
        })
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
    }
}
