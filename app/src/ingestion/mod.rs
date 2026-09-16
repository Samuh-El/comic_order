//! Módulo de escaneo reactivo e ingesta asíncrona de archivos.

pub mod comic_info;
pub mod watcher;
pub mod worker;

pub use comic_info::ParsedComicInfo;
pub use watcher::{DirectoryWatcher, WatchEvent};
pub use worker::{IngestionNotification, IngestionWorker};
