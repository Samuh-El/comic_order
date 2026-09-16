# Contratos de Puertos e Interfaces en Rust

**Feature**: `001-reorganize-app-modular`  
**Fecha**: 2026-09-15  
**Estado**: Completado  

Este documento define los contratos (traits de Rust) que forman la frontera hexagonal entre el Dominio, los Casos de Uso y los Adaptadores de Entrada/Salida.

---

## 1. Puertos de Salida (Secondary / Outbound Ports)

### A. Trait `ComicFileReader` (Patrón Adaptador para Formatos de Archivo)
```rust
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComicReaderError {
    #[error("Archivo no encontrado: {0}")]
    FileNotFound(String),
    #[error("Formato inválido o archivo corrupto: {0}")]
    CorruptArchive(String),
    #[error("Índice de página fuera de rango: {0}")]
    PageOutOfRange(usize),
    #[error("Error de I/O: {0}")]
    IoError(#[from] std::io::Error),
}

pub trait ComicFileReader: Send + Sync {
    /// Determina si este adaptador puede procesar la extensión del archivo
    fn can_handle(&self, extension: &str) -> bool;
    
    /// Obtiene el total de páginas de imagen válidas en el archivo
    fn get_page_count(&self, path: &Path) -> Result<usize, ComicReaderError>;
    
    /// Extrae la portada o primera página en bytes
    fn extract_cover(&self, path: &Path) -> Result<Vec<u8>, ComicReaderError>;
    
    /// Extrae una página específica por índice (0-indexed)
    fn extract_page(&self, path: &Path, page_index: usize) -> Result<Vec<u8>, ComicReaderError>;
    
    /// Extrae metadatos embebidos ComicInfo.xml si existen
    fn extract_comic_info(&self, path: &Path) -> Result<Option<String>, ComicReaderError>;
}
```

### B. Trait `ComicRepository` (Persistencia del Catálogo)
```rust
use async_trait::async_trait;
use crate::domain::comic::{Comic, ComicMetadata};

#[async_trait]
pub trait ComicRepository: Send + Sync {
    async fn get_by_id(&self, id: i64) -> Result<Option<Comic>, RepositoryError>;
    async fn get_by_collection(&self, collection_id: i64) -> Result<Vec<Comic>, RepositoryError>;
    async fn exists_by_path(&self, file_path: &str) -> Result<bool, RepositoryError>;
    async fn upsert(&self, comic: &Comic) -> Result<i64, RepositoryError>;
    async fn delete(&self, id: i64) -> Result<(), RepositoryError>;
}
```

### C. Trait `CollectionRepository` (Persistencia de Colecciones)
```rust
use async_trait::async_trait;
use crate::domain::collection::{Collection, CollectionPath};

#[async_trait]
pub trait CollectionRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Collection>, RepositoryError>;
    async fn get_by_id(&self, id: i64) -> Result<Option<Collection>, RepositoryError>;
    async fn create(&self, name: &str) -> Result<i64, RepositoryError>;
    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<(), RepositoryError>;
    async fn get_paths(&self, collection_id: i64) -> Result<Vec<CollectionPath>, RepositoryError>;
    async fn add_path(&self, collection_id: i64, path: &str) -> Result<i64, RepositoryError>;
}
```

### D. Trait `ImageCache` (Patrón Proxy / Caché de Imágenes)
```rust
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait ImageCache: Send + Sync {
    /// Obtiene una página o miniatura desde la caché si existe
    async fn get_cached(&self, comic_id: i64, page: usize, max_width: Option<u32>) -> Option<Vec<u8>>;
    
    /// Guarda una página o miniatura en la caché en disco
    async fn store(&self, comic_id: i64, page: usize, max_width: Option<u32>, data: &[u8]) -> Result<(), std::io::Error>;
    
    /// Limpia entradas antiguas si se supera el límite de disco configurado
    async fn prune(&self, max_bytes: u64) -> Result<u64, std::io::Error>;
}
```

---

## 2. Puertos de Entrada y Fachada de Aplicación

### Trait `ComicService` / `ComicFacade`
Punto unificado consumido por la GUI Iced, la API Axum REST y el feed OPDS:
```rust
use async_trait::async_trait;

#[async_trait]
pub trait ComicApplicationFacade: Send + Sync {
    async fn list_collections(&self) -> Result<Vec<CollectionDto>, ApplicationError>;
    async fn get_comics_in_collection(&self, collection_id: i64) -> Result<Vec<ComicDto>, ApplicationError>;
    async fn get_cover_image(&self, comic_id: i64) -> Result<Vec<u8>, ApplicationError>;
    async fn get_comic_page(&self, comic_id: i64, page: usize, max_width: Option<u32>) -> Result<Vec<u8>, ApplicationError>;
    async fn save_reading_progress(&self, comic_id: i64, device_id: &str, page: usize) -> Result<(), ApplicationError>;
    async fn get_reading_progress(&self, comic_id: i64, device_id: &str) -> Result<Option<usize>, ApplicationError>;
}
```
