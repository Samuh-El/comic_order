//! Anunciador de descubrimiento de servicios en red local (mDNS / Zeroconf).
//!
//! Emite anuncios multicast a 224.0.0.251:5353 para anunciar el servicio `_comic._tcp.local`.

use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{error, info, warn};

const MDNS_MULTICAST_IPV4: &str = "224.0.0.251:5353";
const SERVICE_TYPE: &str = "_comic._tcp.local";

/// Anunciador de servicio en red local.
pub struct MdnsAnnouncer;

impl MdnsAnnouncer {
    /// Inicia una tarea asíncrona en segundo plano que anuncia la presencia del servidor Comic.
    pub fn start(port: u16) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("[mDNS] Iniciando anunciador de servicio para {} en puerto {}", SERVICE_TYPE, port);

            // Enlazar socket UDP local efímero para broadcast multicast
            let socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => s,
                Err(e) => {
                    error!("[mDNS] No se pudo enlazar socket UDP para mDNS: {}", e);
                    return;
                }
            };

            let target: SocketAddr = match MDNS_MULTICAST_IPV4.parse() {
                Ok(addr) => addr,
                Err(e) => {
                    error!("[mDNS] Dirección multicast inválida: {}", e);
                    return;
                }
            };

            let mut interval = tokio::time::interval(Duration::from_secs(30));
            let payload = format!("COMIC-LAN-SERVICE port={} service={}", port, SERVICE_TYPE);

            loop {
                interval.tick().await;
                if let Err(e) = socket.send_to(payload.as_bytes(), target).await {
                    warn!("[mDNS] Advertencia emitiendo paquete de anuncio: {}", e);
                } else {
                    tracing::debug!("[mDNS] Anuncio multicast emitido hacia {}", MDNS_MULTICAST_IPV4);
                }
            }
        })
    }
}
