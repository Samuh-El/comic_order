//! Entidad para gestión de dispositivos autorizados de confianza (trusted devices).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Dispositivo autorizado con credencial permanente para acceso remoto en la red local.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub id: i64,
    pub token: String,
    pub device_name: String,
    pub created_at: DateTime<Utc>,
}

impl TrustedDevice {
    /// Crea un nuevo dispositivo de confianza generando un token seguro de 16 caracteres.
    pub fn new(id: i64, device_name: String) -> Self {
        Self {
            id,
            token: Self::generate_token(),
            device_name,
            created_at: Utc::now(),
        }
    }

    /// Genera un token alfanumérico compacto y seguro de 16 caracteres.
    pub fn generate_token() -> String {
        Uuid::new_v4().simple().to_string()[..16].to_string()
    }

    /// Valida que el token y el nombre del dispositivo cumplan los criterios de integridad.
    pub fn is_valid(&self) -> bool {
        !self.token.trim().is_empty() && !self.device_name.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_token_generation() {
        let token = TrustedDevice::generate_token();
        assert_eq!(token.len(), 16);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_trusted_device_validity() {
        let device = TrustedDevice::new(1, "iPad Air".to_string());
        assert!(device.is_valid());
        assert_eq!(device.device_name, "iPad Air");
    }
}
