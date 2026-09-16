# Instrucciones Canónicas de Backend

Este documento establece las directrices arquitectónicas, estándares de codificación y reglas obligatorias para todo el desarrollo del backend, el motor de lectura de cómics y el servidor HTTP en **Rust**.

---

## 1. Stack y Runtime del Backend

- **Lenguaje**: Rust (Edition 2021).
- **Runtime Asíncrono**: `tokio` (v1.x, feature `full`).
- **Servidor HTTP**: `axum` (v0.7) con extensiones de `tower` y `tower-http` (v0.5).
- **Formatos de Cómic**: `zip` (v2.2) para archivos `.cbz`/`.zip`, `unrar` (v0.5) para archivos `.cbr`/`.rar`.
- **Procesamiento Gráfico**: `image` (v0.25).
- **Generación de QR y Red**: `qrcode` (v0.14) y `local-ip-address` (v0.6).
- **Logging**: `tracing`, `tracing-subscriber`, `tracing-appender`.

---

## 2. Organización Modular del Backend

El código del backend reside en los siguientes módulos bajo `app/src/`:

```
app/src/
├── comic_reader.rs   # Motor de detección, descompresión en memoria y miniaturas
├── db.rs             # Capa de persistencia asíncrona y repositorio SQLite
├── qr.rs             # Resolución de red local y generación de matrices QR
├── server.rs         # Servidor HTTP Axum, middlewares, API REST y SPA web
└── main.rs           # Orquestador del ciclo de vida y despacho de tareas Tokio
```

---

## 3. Motor de Lectura de Cómics (`comic_reader.rs`)

### Reglas de Procesamiento de Archivos:
1. **Extracción Exclusiva en Memoria RAM**:
   - Queda estrictamente prohibido descomprimir cómics completos al disco duro. Toda lectura de páginas debe realizarse en streams o buffers de memoria RAM.
2. **Filtrado de Archivos no Deseados**:
   - Al listar el contenido de archivos comprimidos, DEBEN descartarse directorios, metadatos del sistema operativo (carpetas que comiencen con `__MACOSX`, archivos que inicien con `.`) y ficheros que no tengan extensiones de imagen válidas (`.jpg`, `.jpeg`, `.png`, `.webp`, `.gif`, `.bmp`).
3. **Ordenamiento Alfabético Canónico**:
   - Las páginas dentro de un cómic deben ordenarse alfabéticamente de forma natural para garantizar la secuencia de lectura correcta.
4. **Generación de Miniaturas (*Thumbnails*)**:
   - La portada (página de índice 0) debe extraerse y redimensionarse a una resolución máxima de 300x450 píxeles con formato JPEG para su almacenamiento como BLOB en la base de datos, optimizando el consumo de memoria y la velocidad de carga.
5. **Aislamiento de Operaciones Bloqueantes**:
   - La descompresión de archivos grandes (especialmente a través de la biblioteca nativa `unrar`) o el escaneo recursivo mediante `walkdir` DEBEN ejecutarse dentro de `tokio::task::spawn_blocking` para no bloquear el runtime asíncrono de Tokio.

---

## 4. Servidor Web y API REST (`server.rs`)

### Arquitectura de Rutas y Seguridad:
1. **Rutas Públicas**:
   - `GET /`: Sirve la SPA HTML/CSS/JS autocontenida (`WEB_PAGE`).
   - `GET /ping`: Responde `"pong"` para comprobación de estado de servicio.
2. **Rutas Protegidas (`/api/...`)**:
   - Todas las rutas de la API deben estar agrupadas bajo el router `/api` y protegidas obligatoriamente por el middleware de autenticación `auth_middleware`.
   - Endpoints estándar:
     - `GET /api/collections` -> Lista de colecciones en JSON.
     - `GET /api/collections/:id/icon` -> Icono binario con cabecera `Cache-Control`.
     - `GET /api/collections/:id/comics` -> Lista de cómics en JSON.
     - `GET /api/comics/:id/cover` -> Portada JPEG en miniatura con cabecera `Cache-Control`.
     - `GET /api/comics/:id/page/:page` -> Extracción y servicio en tiempo real de la página solicitada.
     - `GET /api/icons/:name` -> Iconos estáticos del sistema empaquetados.
3. **Middleware de Autenticación (`auth_middleware`)**:
   - Debe aceptar el token mediante:
     1. Cabecera `Authorization: Bearer <token>`.
     2. Parámetro en la URL `?token=<token>` o `?t=<token>`.
   - Un token es válido si coincide con el token de sesión actual en memoria (`state.token`) O si existe y está activo en la tabla `trusted_devices` de la base de datos.
   - En caso de token ausente o inválido, DEBE retornar inmediatamente `401 Unauthorized`.
4. **Middlewares Obligatorios**:
   - **CORS**: Permitir orígenes, métodos y cabeceras necesarios para clientes locales.
   - **Límite de Carga**: `RequestBodyLimitLayer` configurado a 10 MB.
   - **Cabeceras de Seguridad**: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `X-XSS-Protection: 1; mode=block`, y CSP estricto.
5. **Detección de Tipos MIME en Páginas**:
   - Al servir páginas binarias de cómics, la respuesta debe detectar el tipo de contenido inspeccionando los números mágicos del buffer:
     - JPEG: prefijo `[0xFF, 0xD8]` -> `image/jpeg`.
     - PNG: prefijo `[0x89, 0x50, 0x4E, 0x47]` -> `image/png`.
     - Por defecto: `image/jpeg`.

---

## 5. Estándares de Codificación en Rust

1. **Manejo Idiomático de Errores**:
   - PROHIBIDO el uso de `.unwrap()` o `.expect()` en código de producción del backend.
   - Todo posible error (lectura de disco, descompresión, conexión a base de datos, parsing numérico) debe manejarse mediante `Result<T, E>`, el operador `?`, o funciones de combinación (`map_err`, `and_then`).
2. **Logging y Trazas**:
   - Utilizar exclusivamente las macros del ecosistema `tracing`:
     - `error!`: Fallos críticos de conexión a BD, lectura de archivos corruptos o rechazos de autenticación.
     - `warn!`: Condiciones inesperadas no fatales (ej. archivos sin extensión ignorados en el escaneo).
     - `info!`: Eventos de ciclo de vida (inicio de servidor, peticiones HTTP recibidas, cómics agregados).
     - `debug!`: Detalles granulares de depuración (rutas internas, análisis de cabeceras).
3. **Manejo de Ciclo de Vida y Apagado Seguro (*Graceful Shutdown*)**:
   - El servidor Axum debe vincularse a un canal `tokio::sync::oneshot` para poder cerrarse limpiamente sin dejar puertos bloqueados cuando el usuario detiene el servicio en la GUI de escritorio.
4. **Idioma**:
   - Todos los comentarios de código, docstrings y mensajes de log DEBEN estar redactados exclusivamente en español.
