//! Entidades del dominio para la gestión de colecciones y rutas monitoreadas.

use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::errors::DomainError;

/// Entidad que representa una colección lógica de cómics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    #[serde(skip)]
    pub icon_data: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

impl Collection {
    /// Crea una nueva colección validando las reglas de negocio.
    pub fn new(id: i64, name: String) -> Result<Self, DomainError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DomainError::ValidationError(
                "El nombre de la colección no puede estar vacío".to_string(),
            ));
        }
        if trimmed.len() > 100 {
            return Err(DomainError::ValidationError(
                "El nombre de la colección no puede exceder 100 caracteres".to_string(),
            ));
        }

        Ok(Self {
            id,
            name: trimmed.to_string(),
            icon_data: None,
            created_at: Utc::now(),
        })
    }

    /// Valida los invariantes de negocio de la colección.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() || self.name.len() > 100 {
            return Err(DomainError::ValidationError(
                "Invariante de colección inválido: el nombre debe tener entre 1 y 100 caracteres"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

/// Vínculo entre una colección y una carpeta física en disco.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionPath {
    pub id: i64,
    pub collection_id: i64,
    pub path: PathBuf,
}

impl CollectionPath {
    /// Crea una nueva ruta de colección asociada.
    pub fn new(id: i64, collection_id: i64, path: impl AsRef<Path>) -> Self {
        Self {
            id,
            collection_id,
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Comprueba si la ruta existe físicamente y es un directorio accesible.
    pub fn is_accessible(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_collection_creation() {
        let coll = Collection::new(1, "Marvel".to_string()).expect("Debería crearse correctamente");
        assert_eq!(coll.name, "Marvel");
        assert!(coll.validate().is_ok());
    }

    #[test]
    fn test_empty_collection_name_fails() {
        let res = Collection::new(1, "   ".to_string());
        assert!(res.is_err());
    }

    #[test]
    fn test_too_long_collection_name_fails() {
        let long_name = "A".repeat(101);
        let res = Collection::new(1, long_name);
        assert!(res.is_err());
    }
}
