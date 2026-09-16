//! Servicio de aplicación para la gestión del catálogo de colecciones y cómics.

use std::path::Path;
use std::sync::Arc;
use tracing::info;
use crate::domain::comic::{Comic, ComicFormat};
use crate::domain::collection::{Collection, CollectionPath};
use crate::domain::errors::DomainError;
use crate::domain::ports::{CollectionRepository, ComicFileReader, ComicRepository};

/// Servicio de catálogo para colecciones, rutas e indexación de volúmenes.
#[derive(Clone)]
pub struct CatalogService {
    comic_repo: Arc<dyn ComicRepository>,
    collection_repo: Arc<dyn CollectionRepository>,
    readers: Vec<Arc<dyn ComicFileReader>>,
}

impl CatalogService {
    pub fn new(
        comic_repo: Arc<dyn ComicRepository>,
        collection_repo: Arc<dyn CollectionRepository>,
        readers: Vec<Arc<dyn ComicFileReader>>,
    ) -> Self {
        Self {
            comic_repo,
            collection_repo,
            readers,
        }
    }

    // === Colecciones ===

    pub async fn list_collections(&self) -> Result<Vec<Collection>, DomainError> {
        Ok(self.collection_repo.get_all().await?)
    }

    pub async fn get_collection(&self, id: i64) -> Result<Option<Collection>, DomainError> {
        Ok(self.collection_repo.get_by_id(id).await?)
    }

    pub async fn create_collection(&self, name: &str) -> Result<i64, DomainError> {
        let coll = Collection::new(0, name.to_string())?;
        Ok(self.collection_repo.create(&coll.name).await?)
    }

    pub async fn update_collection(&self, collection: &Collection) -> Result<(), DomainError> {
        collection.validate()?;
        Ok(self.collection_repo.update(collection).await?)
    }

    pub async fn delete_collection(&self, id: i64) -> Result<(), DomainError> {
        Ok(self.collection_repo.delete(id).await?)
    }

    pub async fn set_collection_icon(&self, id: i64, icon_data: &[u8]) -> Result<(), DomainError> {
        Ok(self.collection_repo.set_icon(id, icon_data).await?)
    }

    // === Rutas de Colección ===

    pub async fn get_collection_paths(&self, collection_id: i64) -> Result<Vec<CollectionPath>, DomainError> {
        Ok(self.collection_repo.get_paths(collection_id).await?)
    }

    pub async fn add_collection_path(&self, collection_id: i64, path: &str) -> Result<i64, DomainError> {
        Ok(self.collection_repo.add_path(collection_id, path).await?)
    }

    pub async fn remove_collection_path(&self, path_id: i64) -> Result<(), DomainError> {
        Ok(self.collection_repo.remove_path(path_id).await?)
    }

    // === Cómics del Catálogo ===

    pub async fn list_comics(&self, collection_id: i64) -> Result<Vec<Comic>, DomainError> {
        Ok(self.comic_repo.get_by_collection(collection_id).await?)
    }

    pub async fn get_comic(&self, id: i64) -> Result<Option<Comic>, DomainError> {
        Ok(self.comic_repo.get_by_id(id).await?)
    }

    pub async fn update_comic_metadata(
        &self,
        id: i64,
        title: &str,
        year: Option<i32>,
        issue_number: Option<i32>,
        saga: Option<&str>,
    ) -> Result<(), DomainError> {
        Ok(self
            .comic_repo
            .update_metadata(id, title, year, issue_number, saga)
            .await?)
    }

    pub async fn delete_comic(&self, id: i64) -> Result<(), DomainError> {
        Ok(self.comic_repo.delete(id).await?)
    }

    /// Comprueba si un cómic ya está indexado en la base de datos por su ruta física.
    pub async fn comic_exists(&self, file_path: &str) -> Result<bool, DomainError> {
        Ok(self.comic_repo.exists_by_path(file_path).await?)
    }

    /// Indexa un cómic físico en la colección extrayendo su portada y cantidad de páginas.
    pub async fn index_comic_file(&self, collection_id: i64, path: &Path) -> Result<i64, DomainError> {
        let path_str = path.to_string_lossy().to_string();
        if self.comic_exists(&path_str).await? {
            return Err(DomainError::ValidationError(format!(
                "El cómic ya se encuentra indexado: {}",
                path_str
            )));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let format = ComicFormat::from_extension(&ext)
            .ok_or_else(|| DomainError::UnsupportedFormat(ext.clone()))?;

        let reader = self
            .readers
            .iter()
            .find(|r| r.can_handle(&ext))
            .ok_or_else(|| DomainError::UnsupportedFormat(ext.clone()))?;

        let page_count = reader.get_page_count(path)?;
        let cover_data = reader.extract_cover(path).ok();

        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Sin Título")
            .to_string();

        let mut comic = Comic::new(0, collection_id, file_stem, path.to_path_buf(), format, page_count)?;
        comic.cover_data = cover_data;

        let comic_id = self.comic_repo.upsert(&comic).await?;
        info!("[Catálogo] Cómic indexado exitosamente: ID {}, '{}'", comic_id, comic.title);
        Ok(comic_id)
    }
}
