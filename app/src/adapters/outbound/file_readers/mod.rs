//! Adaptadores y factoría para lectura de archivos de cómics heterogéneos.

pub mod cbz_adapter;
pub mod cbr_adapter;

pub use cbz_adapter::CbzAdapter;
pub use cbr_adapter::CbrAdapter;

use std::path::Path;
use std::sync::Arc;
use crate::domain::ports::ComicFileReader;

/// Factoría para resolver el lector adecuado según la extensión del archivo.
#[derive(Clone)]
pub struct ComicReaderRegistry {
    readers: Vec<Arc<dyn ComicFileReader>>,
}

impl Default for ComicReaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ComicReaderRegistry {
    /// Inicializa el registro con los lectores por defecto (CBZ y CBR).
    pub fn new() -> Self {
        Self {
            readers: vec![
                Arc::new(CbzAdapter::new()),
                Arc::new(CbrAdapter::new()),
            ],
        }
    }

    /// Obtiene el lector capaz de procesar una ruta de archivo según su extensión.
    pub fn get_reader_for_path(&self, path: &Path) -> Option<Arc<dyn ComicFileReader>> {
        let ext = path.extension()?.to_str()?;
        self.get_reader_for_extension(ext)
    }

    /// Obtiene el lector capaz de procesar una extensión específica.
    pub fn get_reader_for_extension(&self, extension: &str) -> Option<Arc<dyn ComicFileReader>> {
        self.readers
            .iter()
            .find(|r| r.can_handle(extension))
            .cloned()
    }
}
