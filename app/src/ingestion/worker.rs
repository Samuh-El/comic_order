//! Trabajador de ingesta asíncrono para barrido inicial y procesamiento de eventos en segundo plano.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use walkdir::WalkDir;

use crate::application::facade::ComicFacade;
use crate::ingestion::watcher::WatchEvent;

/// Notificaciones emitidas por el trabajador de ingesta hacia la interfaz de usuario.
#[derive(Debug, Clone)]
pub enum IngestionNotification {
    ScanStarted,
    ComicIndexed { path: PathBuf, title: String },
    ScanFinished { total_new: usize },
}

/// Trabajador asíncrono responsable de sincronizar archivos físicos con la base de datos.
pub struct IngestionWorker {
    facade: ComicFacade,
    notification_tx: Option<mpsc::Sender<IngestionNotification>>,
}

impl IngestionWorker {
    pub fn new(
        facade: ComicFacade,
        notification_tx: Option<mpsc::Sender<IngestionNotification>>,
    ) -> Self {
        Self {
            facade,
            notification_tx,
        }
    }

    /// Realiza un escaneo inicial recursivo de todas las rutas registradas en la colección.
    pub async fn scan_collection(&self, collection_id: i64) -> Result<usize, crate::domain::errors::DomainError> {
        if let Some(tx) = &self.notification_tx {
            let _ = tx.send(IngestionNotification::ScanStarted).await;
        }

        let paths = self.facade.get_collection_paths(collection_id).await?;
        let mut new_count = 0;

        for coll_path in paths {
            let dir_path = PathBuf::from(&coll_path.path);
            if !dir_path.exists() {
                warn!("[INGESTION] Ruta no existe en disco: {:?}", dir_path);
                continue;
            }

            info!("[INGESTION] Iniciando barrido en: {:?}", dir_path);
            for entry in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && is_supported_comic(path) {
                    let path_str = path.to_string_lossy().to_string();
                    if !self.facade.comic_exists(&path_str).await? {
                        match self.facade.index_comic_file(collection_id, path).await {
                            Ok(comic_id) => {
                                new_count += 1;
                                info!("[INGESTION] Comic indexado exitosamente: {:?} (ID: {})", path, comic_id);

                                if let Some(tx) = &self.notification_tx {
                                    let _ = tx.send(IngestionNotification::ComicIndexed {
                                        path: path.to_path_buf(),
                                        title: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                                    }).await;
                                }
                            }
                            Err(e) => {
                                error!("[INGESTION] Fallo indexando cómic {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        if let Some(tx) = &self.notification_tx {
            let _ = tx.send(IngestionNotification::ScanFinished { total_new: new_count }).await;
        }

        Ok(new_count)
    }

    /// Inicia el bucle de procesamiento continuo de eventos reactivos del File Watcher.
    pub async fn process_watcher_events(
        self: Arc<Self>,
        collection_id: i64,
        mut event_rx: mpsc::Receiver<WatchEvent>,
    ) {
        info!("[INGESTION] Bucle de procesamiento reactivo iniciado para colección {}", collection_id);
        while let Some(event) = event_rx.recv().await {
            match event {
                WatchEvent::Created(path) | WatchEvent::Modified(path) => {
                    info!("[INGESTION] Evento de archivo detectado: {:?}", path);
                    let path_str = path.to_string_lossy().to_string();
                    let exists = self.facade.comic_exists(&path_str).await.unwrap_or(false);
                    if !exists {
                        if let Ok(id) = self.facade.index_comic_file(collection_id, &path).await {
                            info!("[INGESTION] Nuevo cómic detectado e indexado: {:?} (ID: {})", path, id);
                            if let Some(tx) = &self.notification_tx {
                                let _ = tx.send(IngestionNotification::ComicIndexed {
                                    path: path.clone(),
                                    title: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                                }).await;
                            }
                        }
                    }
                }
                WatchEvent::Deleted(path) => {
                    info!("[INGESTION] Archivo eliminado en disco: {:?}", path);
                    // Opcionalmente se puede limpiar de la base de datos en futuras iteraciones
                }
            }
        }
    }
}

/// Comprueba si la extensión del archivo corresponde a un formato de cómic soportado.
fn is_supported_comic(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let lower = ext.to_lowercase();
        matches!(lower.as_str(), "cbz" | "zip" | "cbr" | "rar")
    } else {
        false
    }
}
