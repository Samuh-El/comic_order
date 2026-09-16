//! Definición de errores fuertemente tipados del dominio.
//!
//! Todos los errores se expresan mediante enumeraciones idiomáticas utilizando `thiserror`,
//! evitando el uso de pánicos en runtime.

use thiserror::Error;

/// Errores generales ocurridos dentro del núcleo del dominio o casos de uso.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Entidad no encontrada: {0}")]
    NotFound(String),

    #[error("Error de validación: {0}")]
    ValidationError(String),

    #[error("Formato de cómic no soportado o inválido: {0}")]
    UnsupportedFormat(String),

    #[error("Error al leer el archivo de cómic: {0}")]
    ReaderError(#[from] ComicReaderError),

    #[error("Error de persistencia en repositorio: {0}")]
    RepositoryError(#[from] RepositoryError),

    #[error("Error de I/O en sistema de archivos o caché: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error interno del sistema: {0}")]
    Internal(String),
}

/// Errores específicos generados por los lectores de formatos comprimidos.
#[derive(Debug, Error)]
pub enum ComicReaderError {
    #[error("Archivo no encontrado en la ruta: {0}")]
    FileNotFound(String),

    #[error("Formato inválido o archivo corrupto: {0}")]
    CorruptArchive(String),

    #[error("Índice de página fuera de rango: {0}")]
    PageOutOfRange(usize),

    #[error("No se encontraron páginas de imagen válidas en el archivo")]
    NoImagesFound,

    #[error("Error de I/O al leer el cómic: {0}")]
    IoError(#[from] std::io::Error),
}

/// Errores producidos en la capa de persistencia relacional o almacenamiento.
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Error en la base de datos: {0}")]
    DatabaseError(String),

    #[error("Violación de restricción de integridad única: {0}")]
    ConstraintViolation(String),

    #[error("Registro no encontrado: {0}")]
    NotFound(String),
}
