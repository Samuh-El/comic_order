//! Adaptadores de salida (secundarios) de la arquitectura hexagonal.
//!
//! Encapsula la comunicación con la infraestructura externa:
//! - Persistencia en base de datos relacional SQLite (`persistence`).
//! - Lectores de archivos comprimidos CBZ/CBR (`file_readers`).
//! - Proxy de almacenamiento en caché en disco (`cache`).

pub mod persistence;
pub mod file_readers;
pub mod cache;
