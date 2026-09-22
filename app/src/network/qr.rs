//! Generación de códigos QR matriciales y detección de IP en red local (LAN).

use qrcode::QrCode;
use qrcode::render::svg;

/// Genera un código QR en formato SVG como cadena de texto.
pub fn generate_qr_svg(url: &str) -> String {
    if let Ok(code) = QrCode::new(url.as_bytes()) {
        code.render::<svg::Color>()
            .min_dimensions(200, 200)
            .build()
    } else {
        String::new()
    }
}

/// Genera un código QR como datos de píxeles sin procesar (RGBA) para su visualización en Iced.
///
/// Retorna `Some((rgba_bytes, width, height))` o `None` si la codificación falla.
pub fn generate_qr_image(url: &str, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    let code = QrCode::new(url.as_bytes()).ok()?;

    let image = code
        .render::<image::Luma<u8>>()
        .quiet_zone(true)
        .min_dimensions(size, size)
        .build();

    let width = image.width();
    let height = image.height();

    // Convertir a formato RGBA de 32 bits
    let rgba: Vec<u8> = image
        .pixels()
        .flat_map(|p| {
            let v = p.0[0];
            [v, v, v, 255u8]
        })
        .collect();

    Some((rgba, width, height))
}

/// Obtiene la dirección IP local para la URL del servidor con un token de seguridad opcional.
pub fn get_server_url(port: u16, token: Option<&str>) -> String {
    let base = match local_ip_address::local_ip() {
        Ok(ip) => format!("http://{}:{}", ip, port),
        Err(_) => format!("http://127.0.0.1:{}", port),
    };

    if let Some(t) = token {
        format!("{}/?token={}", base, t)
    } else {
        base
    }
}
