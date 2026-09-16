//! Definición de contratos de puertos (traits) de la Arquitectura Hexagonal.

use std::path::Path;
use async_trait::async_trait;
use crate::domain::comic::Comic;
use crate::domain::collection::{Collection, CollectionPath};
use crate::domain::device::TrustedDevice;
use crate::domain::progress::ReadingProgress;
use crate::domain::errors::{ComicReaderError, RepositoryError};

/// Puerto de salida para adaptadores de lectura de cómics empaquetados (Patrón Adaptador).
pub trait ComicFileReader: Send + Sync {
    /// Determina si este adaptador puede procesar la extensión del archivo.
    fn can_handle(&self, extension: &str) -> bool;

    /// Obtiene el total de páginas de imagen válidas en el archivo.
    fn get_page_count(&self, path: &Path) -> Result<usize, ComicReaderError>;

    /// Extrae la carátula o primera página del cómic en bytes.
    fn extract_cover(&self, path: &Path) -> Result<Vec<u8>, ComicReaderError>;

    /// Extrae selectivamente una página específica en memoria RAM por índice (0-based).
    fn extract_page(&self, path: &Path, page_index: usize) -> Result<Vec<u8>, ComicReaderError>;

    /// Extrae el contenido XML embebido de ComicInfo.xml si está presente.
    fn extract_comic_info(&self, path: &Path) -> Result<Option<String>, ComicReaderError>;
}

/// Puerto de salida para la persistencia del catálogo de cómics.
#[async_trait]
pub trait ComicRepository: Send + Sync {
    async fn get_by_id(&self, id: i64) -> Result<Option<Comic>, RepositoryError>;
    async fn get_by_collection(&self, collection_id: i64) -> Result<Vec<Comic>, RepositoryError>;
    async fn exists_by_path(&self, file_path: &str) -> Result<bool, RepositoryError>;
    async fn upsert(&self, comic: &Comic) -> Result<i64, RepositoryError>;
    async fn delete(&self, id: i64) -> Result<(), RepositoryError>;
    async fn update_metadata(
        &self,
        id: i64,
        title: &str,
        year: Option<i32>,
        issue_number: Option<i32>,
        saga: Option<&str>,
    ) -> Result<(), RepositoryError>;
}

/// Puerto de salida para la persistencia de colecciones y sus rutas monitoreadas.
#[async_trait]
pub trait CollectionRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Collection>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Collection>, RepositoryError>;
    async fn create(&self, name: &str) -> Result<i64, RepositoryError>;
    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<(), RepositoryError>;
    async fn get_paths(&self, collection_id: i64) -> Result<Vec<CollectionPath>, RepositoryError>;
    async fn add_path(&self, collection_id: i64, path: &str) -> Result<i64, RepositoryError>;
    async fn remove_path(&self, path_id: i64) -> Result<(), RepositoryError>;
    async fn set_icon(&self, collection_id: i64, icon_data: &[u8]) -> Result<(), RepositoryError>;
}

/// Puerto de salida para la persistencia de dispositivos de confianza autorizados.
#[async_trait]
pub trait DeviceRepository: Send + Sync {
    async fn get_all_trusted_devices(&self) -> Result<Vec<TrustedDevice>, RepositoryError>;
    async fn create_trusted_device(&self, device_name: &str) -> Result<TrustedDevice, RepositoryError>;
    async fn validate_token(&self, token: &str) -> Result<bool, RepositoryError>;
    async fn delete_trusted_device(&self, id: i64) -> Result<(), RepositoryError>;
}

/// Puerto de salida para la persistencia del avance de lectura.
#[async_trait]
pub trait ProgressRepository: Send + Sync {
    async fn get_progress(&self, comic_id: i64, device_id: &str) -> Result<Option<ReadingProgress>, RepositoryError>;
    async fn save_progress(&self, progress: &ReadingProgress) -> Result<(), RepositoryError>;
}

/// Puerto de salida para el almacenamiento en caché de imágenes y páginas (Patrón Proxy / Caché).
#[async_trait]
pub trait ImageCache: Send + Sync {
    /// Obtiene una página o miniatura desde la caché en disco si existe.
    async fn get_cached(&self, comic_id: i64, page: usize, max_width: Option<u32>) -> Option<Vec<u8>>;

    /// Almacena una página o miniatura en disco.
    async fn store(&self, comic_id: i64, page: usize, max_width: Option<u32>, data: &[u8]) -> Result<(), std::io::Error>;

    /// Aplica una política de limpieza eliminando entradas antiguas si se supera el límite de bytes.
    async fn prune(&self, max_bytes: u64) -> Result<u64, std::io::Error>;
}

/// Puerto de entrada/salida para la notificación de sincronización en tiempo real (WebSockets).
#[async_trait]
pub trait ReadingProgressNotifier: Send + Sync {
    /// Difunde la actualización de progreso a los clientes conectados.
    async fn broadcast_progress(&self, progress: &ReadingProgress) -> Result<(), String>;
}
