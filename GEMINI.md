# Instrucciones de trabajo con Gemini y Antigravity

Este archivo resume las reglas obligatorias del proyecto y muestra cómo debe actuar el agente frente a cambios en el sistema. 
La fuente de verdad completa es `.specify/memory/constitution.md`. Ante cualquier conflicto, la constitution prevalece.
Considera este archivo como el archivo `instructions` central según el marco de trabajo de **Spec-Driven Development**, es decir, es el equivalente a `copilot-instructions.md` u otros sistemas de IA y su forma de trabajo en **Spec-Driven Development**.

## Idioma

Todo el contenido `.md`, los comentarios y los docstrings DEBEN estar en español. NUNCA mezclar idiomas dentro de un mismo archivo.

# Stack Tecnológico

- **Lenguaje**: Rust (Edition 2021).
- **GUI de Escritorio**: `iced` (v0.13) con backend acelerado por hardware y features `["image", "tokio", "canvas", "svg"]`, implementando *The Elm Architecture* (TEA).
- **Base de Datos y Persistencia**: SQLite 3 embebido (`comic.db`) gestionado de forma asíncrona mediante `sqlx` (v0.8) con `SqlitePool` y modo `WAL` (`PRAGMA journal_mode=WAL;`).
- **Servidor HTTP Embebido**: `axum` (v0.7) sobre runtime `tokio` (v1.x, feature `full`) y utilidades de middleware `tower-http` (v0.5) para CORS, límites de carga y cabeceras de seguridad.
- **Motor de Cómics**:
  - Lectura de archivos `.cbz` / `.zip` mediante `zip` (v2.2).
  - Lectura de archivos `.cbr` / `.rar` mediante `unrar` (v0.5).
  - Procesamiento gráfico, conversión a buffers RGBA y generación de miniaturas (thumbnails JPEG 300x450) con `image` (v0.25).
- **Red y Utilidades**:
  - Detección de IP local mediante `local-ip-address` (v0.6).
  - Generación en memoria de códigos QR matriciales con `qrcode` (v0.14).
  - Identificadores únicos criptográficos (UUID v4) con `uuid` (v1.x).
  - Recorrido eficiente del sistema de archivos con `walkdir` (v2.x).
  - Formateo temporal y marcas de fecha con `chrono` (v0.4).
- **Logging y Trazas**: Ecosistema `tracing`, `tracing-subscriber` y `tracing-appender` con rotación diaria automática de archivos en `./log/comic.YYYY-MM-DD.log`.
- **Lector Remoto Web**: SPA ligera autocontenida desarrollada en HTML5, CSS3 moderno (Custom Properties, Glassmorphism, CSS Grid/Flexbox) y JavaScript ES6+ sin dependencias externas ni CDNs.
- **Entorno y Compilación**:
  - Compilador MSVC de Visual Studio con Windows 11 SDK.
  - Enlazado estático de CRT (`-C target-feature=+crt-static`) para binarios autónomos sin dependencia de `VCRUNTIME140.dll`.

# Prohibiciones absolutas

1. **PROHIBIDO el uso de servicios externos en la nube o telemetría**: La aplicación es estrictamente *offline-first* y local. Ningún módulo debe enviar datos, métricas ni telemetría fuera de la red local.
2. **PROHIBIDO bloquear el hilo principal de la UI**: Toda operación de I/O de disco (escaneo de carpetas, lectura de archivos `.cbz`/`.cbr`, descompresión, acceso a base de datos) DEBE ejecutarse de forma asíncrona mediante `Task::perform` o `tokio::task::spawn_blocking`.
3. **PROHIBIDO extraer cómics completos al disco para visualización**: La extracción de páginas y portadas DEBE realizarse selectivamente y directamente en memoria RAM; nunca descomprimir el archivo completo en carpetas temporales del disco duro.
4. **PROHIBIDO omitir la validación de tokens en la API remota**: Todos los endpoints bajo `/api/` (excepto `/ping` y `/`) DEBEN pasar estrictamente por el middleware de autenticación (`auth_middleware`), validando contra el token de sesión o contra la tabla de dispositivos de confianza (`trusted_devices`).
5. **PROHIBIDO incluir librerías o CDNs externos en la SPA web**: La página servida por el servidor HTTP (`WEB_PAGE`) debe ser 100% autónoma; no debe depender de fuentes externas, scripts alojados en CDN ni estilos de terceros.
6. **PROHIBIDO el uso de consultas SQL directas no parametrizadas**: Queda terminantemente prohibido interpolar o concatenar cadenas en sentencias SQL; todo parámetro debe pasarse con `.bind()` de `sqlx`.
7. **PROHIBIDO el pánico descontrolado (`panic!`, `unwrap()`, `expect()`) en runtime**: Todo fallo de I/O, formato de archivo corrupto o error de red debe capturarse idiomáticamente mediante `Result`/`Option` y registrarse con `tracing::error!` o `tracing::warn!`.
8. **PROHIBIDO mezclar idiomas**: El código, comentarios, docstrings y toda la documentación `.md` DEBEN escribirse en español.
9. **PROHIBIDO crear specs o implementar código sin el flujo SDD**: Ninguna funcionalidad o refactorización debe implementarse sin estar precedida por una spec formal en `specs/`, su correspondiente `plan.md` y sus tareas en `tasks.md`.
10. **PROHIBIDO crear specs fuera de `specs/`**: El directorio `.specify/` es de uso exclusivo para la infraestructura de spec-kit.

# Arquitectura

El proyecto implementa los principios de **Clean Architecture** y principios **SOLID** organizados en capas funcionales claras:

- **Capa de Dominio y Procesamiento (`src/comic_reader.rs`)**: Lógica pura para identificación de formatos, extracción en memoria de imágenes, cálculo de páginas y generación de miniaturas. No depende de la UI ni del servidor HTTP.
- **Capa de Persistencia y Datos (`src/db.rs`)**: Patrón *Repository* sobre SQLite, encapsulando consultas, mapeos y migraciones idempotentes tras el struct `Database`.
- **Capa de Red y Distribución (`src/server.rs`, `src/qr.rs`)**: Servidor web Axum, middlewares de seguridad, serialización JSON y servicio de recursos binarios.
- **Capa de Presentación Desktop (`src/ui/`)**: Arquitectura reactiva TEA (*The Elm Architecture*) modulada por componentes puros (`sidebar`, `comic_grid`, `reader`, `metadata_editor`, `collection_editor`, `trusted_devices`).
- **Capa de Orquestación (`src/main.rs`)**: Configuración de runtime, gestión del estado global (`ComicApp`), despacho de mensajes (`Message`) y coordinación de tareas asíncronas.

# Flujo Speckit — hooks obligatorios

Antes de ejecutar cualquier comando `speckit.*`, el agente DEBE leer `.specify/memory/constitution.md` completo para aplicar las reglas vigentes.

Antes de ejecutar cualquier comando `speckit.*`, el agente DEBE revisar `.specify/extensions.yml` (si existe) y ejecutar todos los hooks `before_<comando>` con `optional: false`.

Caso crítico: `before_specify` requiere ejecutar `speckit.git.feature` ANTES de crear el directorio de la spec o `spec.md`. Este hook crea el branch de la feature y actualiza `.specify/feature.json`.

PROHIBIDO crear archivos bajo `specs/<nueva-spec>/` sin haber ejecutado previamente el hook obligatorio correspondiente.