//! Capa de aplicación y orquestación de casos de uso.
//!
//! Contiene la fachada unificada `ComicFacade` y los servicios de catálogo y lectura.

pub mod facade;
pub mod catalog_service;
pub mod reader_service;

pub use facade::ComicFacade;
pub use catalog_service::CatalogService;
pub use reader_service::{ReaderService, ReadingMode};
