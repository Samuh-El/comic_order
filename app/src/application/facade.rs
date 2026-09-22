//! Fachada unificada de aplicación (Patrón Fachada).
//!
//! Oculta la complejidad interna de repositorios, lectores y caches tras una
//! interfaz sencilla y cohesiva consumida por adaptadores de entrada (Iced, Axum, OPDS).

use std::path::Path;
use std::sync::Arc;
use crate::application::catalog_service::CatalogService;
use crate::application::reader_service::ReaderService;
use crate::domain::comic::Comic;
use crate::domain::collection::{Collection, CollectionPath};
use crate::domain::device::TrustedDevice;
use crate::domain::errors::DomainError;
use crate::domain::ports::DeviceRepository;
use crate::domain::progress::ReadingProgress;

/// Fachada unificada para operaciones del catálogo, lectura y autenticación.
#[derive(Clone)]
pub struct ComicFacade {
    catalog: CatalogService,
    reader: ReaderService,
    devices: Arc<dyn DeviceRepository>,
}

impl std::fmt::Debug for ComicFacade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComicFacade").finish()
    }
}

impl ComicFacade {
    pub fn new(
        catalog: CatalogService,
        reader: ReaderService,
        devices: Arc<dyn DeviceRepository>,
    ) -> Self {
        Self {
            catalog,
            reader,
            devices,
        }
    }

    // === Colecciones ===

    pub async fn get_collections(&self) -> Result<Vec<Collection>, DomainError> {
        self.catalog.list_collections().await
    }

    pub async fn get_recent_collections(&self, limit: usize) -> Result<Vec<Collection>, DomainError> {
        self.catalog.list_recent_collections(limit).await
    }

    pub async fn get_collection_by_id(&self, id: i64) -> Result<Option<Collection>, DomainError> {
        self.catalog.get_collection(id).await
    }

    pub async fn create_collection(&self, name: &str) -> Result<i64, DomainError> {
        self.catalog.create_collection(name).await
    }

    pub async fn create_collection_with_details(
        &self,
        name: &str,
        protagonist: Option<&str>,
        description: Option<&str>,
        background_image_path: Option<&str>,
        hero_image_path: Option<&str>,
    ) -> Result<i64, DomainError> {
        self.catalog.create_collection_with_details(name, protagonist, description, background_image_path, hero_image_path).await
    }

    pub async fn update_collection(&self, collection: &Collection) -> Result<(), DomainError> {
        self.catalog.update_collection(collection).await
    }

    pub async fn delete_collection(&self, id: i64) -> Result<(), DomainError> {
        self.catalog.delete_collection(id).await
    }

    pub async fn set_collection_icon(&self, id: i64, icon_data: &[u8]) -> Result<(), DomainError> {
        self.catalog.set_collection_icon(id, icon_data).await
    }

    // === Rutas de Colección ===

    pub async fn get_collection_paths(&self, collection_id: i64) -> Result<Vec<CollectionPath>, DomainError> {
        self.catalog.get_collection_paths(collection_id).await
    }

    pub async fn add_collection_path(&self, collection_id: i64, path: &str) -> Result<i64, DomainError> {
        self.catalog.add_collection_path(collection_id, path).await
    }

    pub async fn remove_collection_path(&self, path_id: i64) -> Result<(), DomainError> {
        self.catalog.remove_collection_path(path_id).await
    }

    // === Cómics ===

    pub async fn get_comics_by_collection(&self, collection_id: i64) -> Result<Vec<Comic>, DomainError> {
        self.catalog.list_comics(collection_id).await
    }

    pub async fn get_comic_by_id(&self, id: i64) -> Result<Option<Comic>, DomainError> {
        self.catalog.get_comic(id).await
    }

    pub async fn update_comic_metadata(
        &self,
        id: i64,
        title: &str,
        year: Option<i32>,
        issue_number: Option<i32>,
        saga: Option<&str>,
    ) -> Result<(), DomainError> {
        self.catalog.update_comic_metadata(id, title, year, issue_number, saga).await
    }

    pub async fn delete_comic(&self, id: i64) -> Result<(), DomainError> {
        self.catalog.delete_comic(id).await
    }

    pub async fn comic_exists(&self, file_path: &str) -> Result<bool, DomainError> {
        self.catalog.comic_exists(file_path).await
    }

    pub async fn index_comic_file(&self, collection_id: i64, path: &Path) -> Result<i64, DomainError> {
        self.catalog.index_comic_file(collection_id, path).await
    }

    // === Lectura y Entrega de Páginas ===

    pub async fn get_cover_image(&self, comic_id: i64) -> Result<Vec<u8>, DomainError> {
        self.reader.get_cover(comic_id).await
    }

    pub async fn get_page_image(
        &self,
        comic_id: i64,
        page_index: usize,
        max_width: Option<u32>,
    ) -> Result<Vec<u8>, DomainError> {
        self.reader.get_page(comic_id, page_index, max_width).await
    }

    pub async fn get_reading_progress(
        &self,
        comic_id: i64,
        device_id: &str,
    ) -> Result<Option<ReadingProgress>, DomainError> {
        self.reader.get_progress(comic_id, device_id).await
    }

    pub async fn save_reading_progress(
        &self,
        comic_id: i64,
        device_id: &str,
        page: usize,
    ) -> Result<(), DomainError> {
        self.reader.save_progress(comic_id, device_id, page).await
    }

    // === Dispositivos de Confianza ===

    pub async fn get_trusted_devices(&self) -> Result<Vec<TrustedDevice>, DomainError> {
        Ok(self.devices.get_all_trusted_devices().await?)
    }

    pub async fn add_trusted_device(&self, device_name: &str) -> Result<TrustedDevice, DomainError> {
        Ok(self.devices.create_trusted_device(device_name).await?)
    }

    pub async fn delete_trusted_device(&self, id: i64) -> Result<(), DomainError> {
        Ok(self.devices.delete_trusted_device(id).await?)
    }

    pub async fn is_token_trusted(&self, token: &str) -> Result<bool, DomainError> {
        Ok(self.devices.validate_token(token).await?)
    }
}
