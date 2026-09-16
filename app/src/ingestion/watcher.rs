//! File Watcher reactivo basado en `notify` con debouncing de 500 ms (Patrón Observador).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Tipos de eventos emitidos al detectar modificaciones en el sistema de archivos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

/// Monitorea directorios del sistema de archivos y emite eventos debounced hacia un canal Tokio.
pub struct DirectoryWatcher {
    _watcher: RecommendedWatcher,
    watched_paths: Vec<PathBuf>,
}

impl DirectoryWatcher {
    /// Inicia el monitoreo de un conjunto de directorios, emitiendo eventos al canal `event_tx`.
    pub fn start(
        paths: Vec<PathBuf>,
        event_tx: mpsc::Sender<WatchEvent>,
    ) -> Result<Self, notify::Error> {
        let (raw_tx, mut raw_rx) = mpsc::channel::<notify::Result<Event>>(100);

        // Canal síncrono para el callback de notify hacia Tokio mpsc
        let watcher_tx = raw_tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = watcher_tx.blocking_send(res);
            },
            Config::default(),
        )?;

        for p in &paths {
            if p.exists() {
                watcher.watch(p, RecursiveMode::Recursive)?;
                info!("[WATCHER] Monitoreando directorio: {:?}", p);
            } else {
                warn!("[WATCHER] Ruta no encontrada para monitoreo: {:?}", p);
            }
        }

        // Tarea asíncrona de debouncing (500 ms)
        tokio::spawn(async move {
            let mut pending_events: HashMap<PathBuf, (WatchEvent, Instant)> = HashMap::new();
            let debounce_duration = Duration::from_millis(500);
            let mut interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                tokio::select! {
                    Some(res) = raw_rx.recv() => {
                        match res {
                            Ok(event) => {
                                for path in event.paths {
                                    if !is_comic_file(&path) {
                                        continue;
                                    }
                                    let now = Instant::now();
                                    match event.kind {
                                        EventKind::Create(_) => {
                                            pending_events.insert(path.clone(), (WatchEvent::Created(path), now));
                                        }
                                        EventKind::Modify(_) => {
                                            pending_events.insert(path.clone(), (WatchEvent::Modified(path), now));
                                        }
                                        EventKind::Remove(_) => {
                                            pending_events.insert(path.clone(), (WatchEvent::Deleted(path), now));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Err(e) => {
                                error!("[WATCHER] Error en evento de notify: {}", e);
                            }
                        }
                    }
                    _ = interval.tick() => {
                        let now = Instant::now();
                        let ready_paths: Vec<PathBuf> = pending_events
                            .iter()
                            .filter(|(_, (_, timestamp))| now.duration_since(*timestamp) >= debounce_duration)
                            .map(|(path, _)| path.clone())
                            .collect();

                        for path in ready_paths {
                            if let Some((event, _)) = pending_events.remove(&path) {
                                debug!("[WATCHER] Despachando evento debounced: {:?}", event);
                                if event_tx.send(event).await.is_err() {
                                    info!("[WATCHER] Canal de eventos cerrado, deteniendo watcher loop");
                                    return;
                                }
                            }
                        }
                    }
                    else => break,
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            watched_paths: paths,
        })
    }

    /// Retorna la lista de rutas que están siendo monitoreadas.
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }
}

/// Comprueba si la ruta corresponde a una extensión de cómic soportada.
fn is_comic_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let lower = ext.to_lowercase();
        matches!(lower.as_str(), "cbz" | "zip" | "cbr" | "rar")
    } else {
        false
    }
}
