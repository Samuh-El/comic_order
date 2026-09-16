//! Entidad de progreso de lectura por cómic y dispositivo.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Estado de lectura asociado a un cómic para un dispositivo cliente específico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub comic_id: i64,
    pub device_id: String,
    pub current_page: usize,
    pub completed: bool,
    pub updated_at: DateTime<Utc>,
}

impl ReadingProgress {
    /// Crea un nuevo registro de progreso de lectura.
    pub fn new(comic_id: i64, device_id: String, current_page: usize, completed: bool) -> Self {
        Self {
            comic_id,
            device_id,
            current_page,
            completed,
            updated_at: Utc::now(),
        }
    }

    /// Calcula el porcentaje de avance respecto al total de páginas del cómic.
    pub fn calculate_percentage(&self, total_pages: usize) -> f32 {
        if total_pages == 0 {
            return 0.0;
        }
        if self.completed {
            return 100.0;
        }
        let percentage = ((self.current_page + 1) as f32 / total_pages as f32) * 100.0;
        percentage.clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_percentage() {
        let progress = ReadingProgress::new(1, "tablet".to_string(), 9, false);
        assert!((progress.calculate_percentage(20) - 50.0).abs() < f32::EPSILON);

        let completed_progress = ReadingProgress::new(1, "tablet".to_string(), 19, true);
        assert!((completed_progress.calculate_percentage(20) - 100.0).abs() < f32::EPSILON);
    }
}
