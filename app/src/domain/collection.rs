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
    pub protagonist: Option<String>,
    pub description: Option<String>,
    pub background_image_path: Option<String>,
    pub hero_image_path: Option<String>,
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
            protagonist: None,
            description: None,
            background_image_path: None,
            hero_image_path: None,
            icon_data: None,
            created_at: Utc::now(),
        })
    }

    /// Devuelve el nombre del protagonista o fallback al nombre de la colección.
    pub fn protagonist_display(&self) -> &str {
        self.protagonist
            .as_deref()
            .filter(|p| !p.trim().is_empty())
            .unwrap_or(&self.name)
    }

    /// Trunca los campos de texto según las reglas de negocio (protagonista: 25, descripción: 600).
    pub fn truncate_fields(&mut self) {
        if let Some(protagonist) = &self.protagonist {
            if protagonist.chars().count() > 25 {
                self.protagonist = Some(protagonist.chars().take(25).collect());
            }
        }
        if let Some(desc) = &self.description {
            if desc.chars().count() > 600 {
                self.description = Some(desc.chars().take(600).collect());
            }
        }
    }

    /// Valida los invariantes de negocio de la colección.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() || self.name.chars().count() > 100 {
            return Err(DomainError::ValidationError(
                "Invariante de colección inválido: el nombre debe tener entre 1 y 100 caracteres"
                    .to_string(),
            ));
        }
        if let Some(protagonist) = &self.protagonist {
            if protagonist.chars().count() > 25 {
                return Err(DomainError::ValidationError(
                    "El nombre del superhéroe/protagonista no puede exceder 25 caracteres".to_string(),
                ));
            }
        }
        if let Some(desc) = &self.description {
            if desc.chars().count() > 600 {
                return Err(DomainError::ValidationError(
                    "La descripción de la colección no puede exceder 600 caracteres".to_string(),
                ));
            }
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

    #[test]
    fn test_collection_with_valid_description() {
        let mut coll = Collection::new(1, "Batman".to_string()).expect("Debería crearse correctamente");
        coll.protagonist = Some("BATMAN".to_string());
        coll.description = Some("El caballero de la noche protege Gotham.".to_string());
        coll.background_image_path = Some("data/media/collections/batman_bg.png".to_string());
        coll.hero_image_path = Some("data/media/collections/batman_hero.png".to_string());
        assert_eq!(coll.protagonist_display(), "BATMAN");
        assert!(coll.validate().is_ok());
    }

    #[test]
    fn test_collection_protagonist_fallback() {
        let coll = Collection::new(1, "Spider-Man".to_string()).expect("Debería crearse correctamente");
        assert_eq!(coll.protagonist_display(), "Spider-Man");
    }

    #[test]
    fn test_collection_with_too_long_description_fails() {
        let mut coll = Collection::new(1, "Spiderman".to_string()).expect("Debería crearse correctamente");
        coll.description = Some("A".repeat(601));
        assert!(coll.validate().is_err());
    }

    #[test]
    fn test_collection_with_too_long_protagonist_fails() {
        let mut coll = Collection::new(1, "Hero".to_string()).expect("Debería crearse correctamente");
        coll.protagonist = Some("A".repeat(26));
        assert!(coll.validate().is_err());
    }

    #[test]
    fn test_collection_truncate_fields() {
        let mut coll = Collection::new(1, "Hero".to_string()).expect("Debería crearse correctamente");
        coll.protagonist = Some("A".repeat(30));
        coll.description = Some("B".repeat(700));
        coll.truncate_fields();
        assert_eq!(coll.protagonist.as_ref().unwrap().len(), 25);
        assert_eq!(coll.description.as_ref().unwrap().len(), 600);
        assert!(coll.validate().is_ok());
    }
}
