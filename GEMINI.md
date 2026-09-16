# Instrucciones de trabajo con Gemini y Antigravity

Este archivo resume las reglas obligatorias del proyecto y muestra cómo debe actuar el agente frente a cambios en el sistema. 
La fuente de verdad completa es `.specify/memory/constitution.md`. Ante cualquier conflicto, la constitution prevalece.
Considera este archivo como el archivo `instructions` central según el marco de trabajo de **Spec-Driven Development**, es decir, es el equivalente a `copilot-instructions.md` u otros sistemas de IA y su forma de trabajo en **Spec-Driven Development**.

## Idioma

Todo el contenido `.md`, los comentarios y los docstrings DEBEN estar en español. NUNCA mezclar idiomas dentro de un mismo archivo.

# Stack Tecnológico

- **Lenguaje**: Rust (Edition 2021).
- **GUI de Escritorio**: `iced` (v0.13) con backend acelerado por hardware y features `["image", "tokio", "canvas", "svg"]`, implementando *The Elm Architecture* (TEA).
- **Base de Datos y Persistencia**: SQLite 3 embebido (`app/data/comic.db`) gestionado de forma asíncrona mediante `sqlx` (v0.8) con `SqlitePool` y modo `WAL` (`PRAGMA journal_mode=WAL;`).
- **Servidor HTTP Embebido**: `axum` (v0.7) sobre runtime `tokio` (v1.x, feature `full`) y utilidades de middleware `tower-http` (v0.5) para CORS, límites de carga y cabeceras de seguridad.
- **Protocolos y Red**:
  - Servidor REST y WebSockets para clientes locales y remotos.
  - Soporte para protocolo estándar **OPDS** (*Open Publication Distribution System*).
  - Descubrimiento de servicios en red local (mDNS / Zeroconf).
  - Detección de IP local mediante `local-ip-address` (v0.6) y códigos QR matriciales con `qrcode` (v0.14).
- **Motor de Cómics**:
  - Lectura de archivos `.cbz` / `.zip` mediante `zip` (v2.2).
  - Lectura de archivos `.cbr` / `.rar` mediante `unrar` (v0.5).
  - Procesamiento gráfico, conversión a buffers RGBA y generación de miniaturas (thumbnails JPEG 300x450) con `image` (v0.25).
  - Extracción de metadatos desde `ComicInfo.xml`.
- **Logging y Trazas**: Ecosistema `tracing`, `tracing-subscriber` y `tracing-appender` con rotación diaria automática de archivos en `app/log/comic.YYYY-MM-DD.log`.
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
9. **PROHIBIDO crear specs o implementar código sin el flujo SDD**: Ninguna funcionalidad o refactorización debe implementarse sin estar precedida por una spec formal en `.specify/specs/`, su correspondiente `plan.md` y sus tareas en `tasks.md`.
10. **PROHIBIDO violar la encapsulación hexagonal**: El núcleo del dominio no debe depender de librerías de infraestructura, frameworks web ni motores de GUI.

# Arquitectura

El proyecto implementa un **Monolito Modular** bajo los principios de **Clean Architecture / Arquitectura Hexagonal (Puertos y Adaptadores)** y principios **SOLID**:

- **Núcleo del Dominio (`app/src/domain/`)**: Lógica pura para identificación de entidades (`Comic`, `Collection`, `Metadata`, `ReadingProgress`), cálculo de páginas y reglas de negocio, sin acoplamiento a frameworks.
- **Puertos de Entrada (Inbound Ports / Primary)**:
  - Servidor Web Axum (REST API).
  - Feed estándar **OPDS** para lectores móviles de terceros.
  - Adaptador de GUI de Escritorio (Iced TEA).
- **Puertos de Salida (Outbound Ports / Secondary)**:
  - Repositorio de base de datos SQLite.
  - Adaptadores de lectura de cómics (`ComicFileReader` para CBZ, CBR, etc.).
  - Proxy de caché de miniaturas y páginas.
- **Módulo de Escaneo e Ingesta (`app/src/ingestion/`)**: *File Watcher* reactivo (patrón Observador) y trabajador asíncrono para ingesta de archivos y extracción de `ComicInfo.xml`.
- **Patrones de Diseño Obligatorios**:
  - **Patrón Adaptador**: `ComicFileReader` unifica la lectura de múltiples formatos comprimidos.
  - **Patrón Fachada**: `ComicFacade` simplifica el acceso a páginas procesadas y metadatos.
  - **Patrón Observador**: Notificación de eventos de archivo hacia la base de datos.
  - **Patrón Estrategia**: Modos de visualización y políticas de compresión dinámicas.
  - **Patrón Proxy / Caché**: Entrega inmediata de imágenes cacheadas.

# Flujo Speckit — hooks obligatorios

Antes de ejecutar cualquier comando `speckit.*`, el agente DEBE leer `.specify/memory/constitution.md` completo para aplicar las reglas vigentes.

Antes de ejecutar cualquier comando `speckit.*`, el agente DEBE revisar `.specify/extensions.yml` (si existe) y ejecutar todos los hooks `before_<comando>` con `optional: false`.