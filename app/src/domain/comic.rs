//! Entidades y tipos de valor para la gestión de cómics y formatos.

use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::errors::DomainError;

/// Formatos de empaquetado de cómic soportados o proyectados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComicFormat {
    Cbz,
    Cbr,
    Pdf,
    Epub,
}

impl ComicFormat {
    /// Determina el formato de cómic a partir de la extensión de archivo (insensible a mayúsculas).
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "cbz" | "zip" => Some(ComicFormat::Cbz),
            "cbr" | "rar" => Some(ComicFormat::Cbr),
            "pdf" => Some(ComicFormat::Pdf),
            "epub" => Some(ComicFormat::Epub),
            _ => None,
        }
    }

    /// Representación canónica en cadena de texto.
    pub fn as_str(&self) -> &'static str {
        match self {
            ComicFormat::Cbz => "cbz",
            ComicFormat::Cbr => "cbr",
            ComicFormat::Pdf => "pdf",
            ComicFormat::Epub => "epub",
        }
    }
}

/// Metadatos bibliográficos adicionales del cómic (extraídos de ComicInfo.xml o manuales).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComicMetadata {
    pub series: Option<String>,
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub summary: Option<String>,
    pub publisher: Option<String>,
    pub tags: Vec<String>,
}

/// Entidad pura que representa un volumen de cómic digital.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comic {
    pub id: i64,
    pub collection_id: i64,
    pub title: String,
    pub file_path: PathBuf,
    pub format: ComicFormat,
    pub year: Option<i32>,
    pub issue_number: Option<i32>,
    pub saga: Option<String>,
    #[serde(skip)]
    pub cover_data: Option<Vec<u8>>,
    pub page_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Comic {
    /// Crea un nuevo cómic aplicando validaciones de dominio básicas.
    pub fn new(
        id: i64,
        collection_id: i64,
        title: String,
        file_path: PathBuf,
        format: ComicFormat,
        page_count: usize,
    ) -> Result<Self, DomainError> {
        let trimmed_title = title.trim();
        if trimmed_title.is_empty() {
            return Err(DomainError::ValidationError(
                "El título del cómic no puede estar vacío".to_string(),
            ));
        }

        let title_capped: String = trimmed_title.chars().take(40).collect();

        let now = Utc::now();
        Ok(Self {
            id,
            collection_id,
            title: title_capped,
            file_path,
            format,
            year: None,
            issue_number: None,
            saga: None,
            cover_data: None,
            page_count,
            created_at: now,
            updated_at: now,
        })
    }

    /// Genera un título visual amigable combinando saga, número de entrega y año si están disponibles.
    pub fn display_title(&self) -> String {
        let mut display = self.title.clone();
        if let Some(issue) = self.issue_number {
            display = format!("{} #{}", display, issue);
        }
        if let Some(ref saga) = self.saga {
            display = format!("{} ({})", display, saga);
        }
        if let Some(year) = self.year {
            display = format!("{} [{}]", display, year);
        }
        display
    }

    /// Valida que el año de publicación esté dentro de un rango cronológico razonable.
    pub fn validate_year(year: i32) -> bool {
        (1900..=2100).contains(&year)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_extension() {
        assert_eq!(ComicFormat::from_extension("cbz"), Some(ComicFormat::Cbz));
        assert_eq!(ComicFormat::from_extension("CBZ"), Some(ComicFormat::Cbz));
        assert_eq!(ComicFormat::from_extension("zip"), Some(ComicFormat::Cbz));
        assert_eq!(ComicFormat::from_extension("cbr"), Some(ComicFormat::Cbr));
        assert_eq!(ComicFormat::from_extension("RAR"), Some(ComicFormat::Cbr));
        assert_eq!(ComicFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_comic_display_title() {
        let mut comic = Comic::new(
            1,
            10,
            "Batman".to_string(),
            PathBuf::from("/comics/batman.cbz"),
            ComicFormat::Cbz,
            32,
        )
        .expect("Debería crear el cómic");

        assert_eq!(comic.display_title(), "Batman");
        comic.issue_number = Some(1);
        assert_eq!(comic.display_title(), "Batman #1");
        comic.saga = Some("Año Uno".to_string());
        assert_eq!(comic.display_title(), "Batman #1 (Año Uno)");
        comic.year = Some(1987);
        assert_eq!(comic.display_title(), "Batman #1 (Año Uno) [1987]");
    }

    #[test]
    fn test_comic_title_truncation_to_40_chars() {
        let long_title = "A".repeat(80);
        let comic = Comic::new(
            1,
            10,
            long_title,
            PathBuf::from("/comics/batman.cbz"),
            ComicFormat::Cbz,
            32,
        )
        .expect("Debería crear el cómic");

        assert_eq!(comic.title.chars().count(), 40);
    }
}
