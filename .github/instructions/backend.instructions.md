# Instrucciones Canónicas de Backend (Monolito Modular y Arquitectura Hexagonal)

Este documento establece las directrices arquitectónicas, estándares de codificación y reglas obligatorias para todo el desarrollo del backend, el motor de lectura de cómics y los servicios de red en **Rust** para el proyecto **Comic**.

---

## 1. Estilo Arquitectónico: Monolito Modular Hexagonal

El backend se estructura siguiendo los principios de la **Arquitectura Hexagonal (Puertos y Adaptadores / Clean Architecture)** dentro de un **Monolito Modular**:

1. **Aislamiento del Dominio**:
   - El núcleo del dominio (`app/src/domain/`) contiene entidades puras, objetos de valor y contratos de puertos. Queda estrictamente prohibido importar librerías de infraestructura, frameworks web (Axum) o GUI (Iced) dentro del dominio.
2. **Puertos de Entrada (Inbound / Primary Ports)**:
   - Definen las interfaces que los actores externos (clientes web, lectores OPDS, GUI) usan para interactuar con la aplicación.
3. **Puertos de Salida (Outbound / Secondary Ports)**:
   - Definen las abstracciones requeridas por el sistema para interactuar con recursos externos: sistema de archivos (`ComicFileReader`), almacenamiento relacional (`ComicRepository`), y caché de medios (`ImageCache`).

---

## 2. Organización de Módulos en `app/src/`

```
app/src/
├── domain/                  # Entidades puras y puertos (traits)
│   ├── mod.rs
│   ├── comic.rs             # Entidad Comic y metadatos
│   ├── collection.rs        # Entidad Collection y rutas asociadas
│   ├── progress.rs          # Progreso de lectura por usuario/dispositivo
│   └── ports.rs             # Traits para lectores de cómic, persistencia y caché
├── application/             # Casos de uso y Fachada
│   ├── mod.rs
│   ├── facade.rs            # ComicFacade (punto unificado de operaciones compuestas)
│   └── services/            # Servicios de orquestación de lectura, catalogación y búsqueda
├── adapters/                # Adaptadores de entrada y salida
│   ├── mod.rs
│   ├── inbound/
│   │   ├── rest_api/        # Endpoints Axum REST, middlewares y autenticación
│   │   ├── opds/            # Feed estándar Atom/OPDS para clientes móviles
│   │   └── websocket/       # Sincronización de progreso en tiempo real
│   └── outbound/
│       ├── persistence/     # Implementación SQLite con SQLx
│       ├── file_readers/    # Adaptadores específicos: CBZ (zip), CBR (unrar)
│       └── cache/           # Proxy de caché de imágenes y páginas en memoria/disco
├── ingestion/               # File Watcher reactivo y procesamiento en segundo plano
│   ├── mod.rs
│   ├── watcher.rs           # Observador de eventos del sistema de archivos
│   ├── worker.rs            # Trabajador asíncrono de ingesta
│   └── metadata_parser.rs   # Extracción de ComicInfo.xml
└── network/                 # Utilidades de red local
    ├── mod.rs
    ├── mdns.rs              # Anuncio y descubrimiento de servicios en LAN
    └── qr.rs                # Generación de códigos QR y detección de IP
```

---

## 3. Patrones de Diseño Obligatorios

### A. Patrón Adaptador (*Adapter Pattern*)
- Todos los formatos de archivo de cómic deben implementar el trait canónico `ComicFileReader`:
  ```rust
  pub trait ComicFileReader: Send + Sync {
      fn can_handle(&self, extension: &str) -> bool;
      fn get_page_count(&self, file_path: &str) -> Result<usize, ComicError>;
      fn extract_cover(&self, file_path: &str) -> Result<Vec<u8>, ComicError>;
      fn extract_page(&self, file_path: &str, page_index: usize) -> Result<Vec<u8>, ComicError>;
  }
  ```
- Se implementan adaptadores específicos para:
  - `CbzFileReader` (basado en `zip`).
  - `CbrFileReader` (basado en `unrar`).
  - Futuros adaptadores (`PdfFileReader`, `EpubFileReader`).

### B. Patrón Fachada (*Facade Pattern*)
- `ComicFacade` proporciona un API simple y de alto nivel a las capas de presentación:
  - `get_comic_page(id, page, max_width, max_height)`: orquesta la consulta de metadatos, la comprobación en el proxy de caché, la descompresión a través del adaptador y el escalado opcional.

### C. Patrón Observador (*Observer Pattern*)
- El módulo de ingesta escucha eventos del sistema de archivos (`create`, `modify`, `delete`) y notifica al servicio de catálogo para actualizar el índice en SQLite automáticamente sin intervención manual.

### D. Patrón Estrategia (*Strategy Pattern*)
- Soporte para políticas intercambiables de compresión de imágenes y modos de visualización (página simple, doble página o lectura vertical continua tipo Webtoon) en función del cliente solicitante.

### E. Patrón Proxy de Caché (*Cache Proxy*)
- Intercepta las solicitudes hacia los adaptadores de archivos: si la miniatura o página redimensionada ya se encuentra en la caché, se entrega en < 10 ms sin tocar los ficheros comprimidos.

---

## 4. Características de Red Local (LAN) y Escalabilidad

1. **Streaming Bajo Demanda**:
   - Se prohíbe enviar cómics enteros al cliente. Las páginas se sirven individualmente en respuesta a peticiones HTTP puntuales.
2. **Descubrimiento mDNS / Zeroconf**:
   - El servidor se anuncia en la red local bajo el tipo de servicio `_comic._tcp.local` para que clientes móviles puedan autodescubrirlo sin escribir IPs manualmente.
3. **Control de Concurrencia**:
   - Toda operación de I/O de archivos pesados o descompresión DEBE aislarse en `tokio::task::spawn_blocking` para mantener libre el pool de trabajadores asíncronos de Tokio.

---

## 5. Estándares de Calidad y Convenciones de Código

- **Tolerancia Cero a Pánicos**: Prohibido el uso de `.unwrap()` y `.expect()` en rutas ejecutadas en runtime.
- **Trazas y Logs**: Uso exclusivo de `tracing` canalizado hacia `app/log/comic.YYYY-MM-DD.log`.
- **Idioma**: 100% en español (código, nombres descriptivos, comentarios y docstrings).
