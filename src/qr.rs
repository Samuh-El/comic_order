use qrcode::QrCode;


/// Generate a QR code as raw pixel data (RGBA) for display in Iced
pub fn generate_qr_image(url: &str, size: u32) -> Option<(Vec<u8>, u32, u32)> {
    let code = QrCode::new(url.as_bytes()).ok()?;
    
    let image = code.render::<image::Luma<u8>>()
        .quiet_zone(true)
        .min_dimensions(size, size)
        .build();
    
    let width = image.width();
    let height = image.height();
    
    // Convert to RGBA
    let rgba: Vec<u8> = image
        .pixels()
        .flat_map(|p| {
            let v = p.0[0];
            [v, v, v, 255u8]
        })
        .collect();
    
    Some((rgba, width, height))
}

/// Get the local IP address for the server URL
pub fn get_server_url(port: u16) -> String {
    match local_ip_address::local_ip() {
        Ok(ip) => format!("http://{}:{}", ip, port),
        Err(_) => format!("http://127.0.0.1:{}", port),
    }
}
