# Plan de Implementación: Reorganización Modular Hexagonal de la Aplicación

**Branch**: `001-reorganize-app-modular` | **Fecha**: 2026-09-15 | **Spec**: [spec.md](./spec.md)  
**Input**: Especificación de características en `.specify/specs/001-reorganize-app-modular/spec.md`

---

## 1. Resumen Ejecutivo

Reorganizar el software alojado en la carpeta `app/` para transformarlo en un **Monolito Modular** bajo los principios de **Clean Architecture y Arquitectura Hexagonal (Puertos y Adaptadores)**. Se mantiene el 100% de las funcionalidades actuales (lectura desktop con zoom/pan, catálogo de colecciones, escaneo de cómics CBZ y CBR, edición de metadatos, servidor web local Axum y acceso remoto mediante código QR y tokens de dispositivos de confianza). La nueva estructura desacopla el núcleo del dominio de los frameworks externos, introduce los patrones de diseño **Adaptador** (para lectores de archivos), **Fachada** (`ComicFacade`), **Observador** (File Watcher reactivo con `notify`), **Estrategia** (modos de lectura y redimensionado) y **Proxy de Caché** en disco (`app/.cache/`), e incorpora soporte para el protocolo estándar **OPDS 1.2** y descubrimiento **mDNS**.

---

## 2. Contexto Técnico

- **Dependencias Principales**:
  - `iced` (v0.13) con features `image`, `tokio`, `canvas`, `svg`.
  - `axum` (v0.7), `tokio` (v1.x full), `tower-http` (v0.5).
  - `sqlx` (v0.8) con `runtime-tokio`, `sqlite`, `macros`.
  - `zip` (v2.2), `unrar` (v0.5), `image` (v0.25).
  - `notify` (v6.x) para el *File Watcher* asíncrono nativo del SO.
  - `qrcode` (v0.14), `local-ip-address` (v0.6), `uuid` (v1.x), `chrono` (v0.4).
  - `tracing`, `tracing-subscriber`, `tracing-appender`.
- **Almacenamiento y Persistencia**:
  - Base de datos relacional ligera: SQLite 3 en `app/comic.db` (modo WAL activado, pool de 5 conexiones).
  - Almacenamiento de caché: `app/.cache/` para páginas e imágenes bajo demanda con política de límite máximo y limpieza.
- **Estrategia de Pruebas**:
  - Pruebas unitarias en Rust con `cargo test` para entidades de dominio y adaptadores de lectura.
  - Pruebas de integración para repositorios SQLite y endpoints HTTP.
- **Plataforma Objetivo**:
  - Windows 11 / 10 (MSVC con `-C target-feature=+crt-static`) como objetivo principal, con compatibilidad multiplataforma lista para Linux y macOS.
- **Tipo de Proyecto**:
  - Aplicación de escritorio nativa + Servidor HTTP/OPDS embebido para red de área local (Monolito Modular).
- **Metas de Rendimiento**:
  - GUI de escritorio fluida a 60 FPS (< 16 ms por cuadro).
  - Respuesta de entrega de páginas cacheadas en < 50 ms.
  - Consumo de RAM en reposo < 120 MB.
- **Restricciones Clave**:
  - 100% *Offline-First* y privado: cero telemetría externa o dependencias en la nube.
  - SPA web servida de forma autónoma sin CDNs ni librerías de internet.
  - Consultas SQL 100% parametrizadas con `.bind()`.
  - Tolerancia cero a pánicos no controlados (`unwrap`/`expect` en runtime).
- **Alcance y Escala**:
  - Soporte de bibliotecas de miles de cómics y hasta 4 lectores concurrentes en streaming local.

---

## 3. Evaluación de la Constitución (Constitution Check)

| Principio Constitucional | Estado | Justificación y Cumplimiento |
| :--- | :---: | :--- |
| **1. Cero Telemetría / 100% Offline** | **PASS** | Todas las conexiones se restringen a `localhost` y la red LAN (`0.0.0.0:8080`). No hay llamadas a internet ni analíticas. |
| **2. Hilo de UI no bloqueado** | **PASS** | Todo el I/O de archivos, consultas a base de datos y descompresión se despacha asíncronamente con Tokio y `spawn_blocking`. |
| **3. Extracción en memoria / Cero dump masivo** | **PASS** | Los adaptadores `ComicFileReader` extraen páginas individuales selectivamente. Las páginas intermedias se almacenan en `app/.cache/` bajo demanda. |
| **4. Validación de tokens en `/api/`** | **PASS** | Se mantiene `auth_middleware` validando tokens de sesión y dispositivos de confianza (`trusted_devices`). |
| **5. SPA Web 100% autónoma** | **PASS** | `WEB_PAGE` no incluye fuentes ni scripts externos en CDN. |
| **6. Consultas SQL parametrizadas** | **PASS** | Todas las consultas en el repositorio utilizan `.bind()` de `sqlx`. |
| **7. Manejo idiomático de errores** | **PASS** | Tipos `Result<T, E>` y propagación con `?`. Cero pánicos en runtime. |
| **8. Idioma 100% español** | **PASS** | Toda la documentación, comentarios y nombres descriptivos se redactan en español. |
| **9. Flujo Spec-Driven Development** | **PASS** | La feature cuenta con `spec.md`, `checklists/requirements.md`, `research.md`, `data-model.md`, `contracts/` y `quickstart.md`. |
| **10. Encapsulación Hexagonal** | **PASS** | El núcleo de `domain/` no depende de Iced, Axum, SQLite ni del sistema de archivos. |

---

## 4. Estructura del Proyecto

### A. Artefactos de Documentación y Diseño (esta feature)
```text
.specify/specs/001-reorganize-app-modular/
├── spec.md              # Especificación de requisitos y clarificaciones
├── plan.md              # Este plan de implementación
├── research.md          # Investigación técnica y justificación de decisiones
├── data-model.md        # Modelo de datos, entidades y DDL de SQLite
├── quickstart.md        # Guía paso a paso de validación y pruebas end-to-end
├── checklists/
│   └── requirements.md  # Validación de completitud de requisitos
└── contracts/
    ├── ports.contract.md    # Contratos de Rust (traits) del dominio
    ├── rest-api.contract.md # Contrato de la API REST y esquemas JSON
    └── opds.contract.md     # Contrato del feed Atom XML OPDS 1.2
```

### B. Estructura Modular del Código Fuente (`app/src/`)
```text
app/
├── Cargo.toml
├── Cargo.lock
├── build.cmd
├── genera_exe.cmd
├── assets/
│   └── [iconos y recursos visuales]
└── src/
    ├── main.rs                      # Punto de entrada y orquestador TEA
    ├── domain/                      # NÚCLEO PURO DEL DOMINIO
    │   ├── mod.rs
    │   ├── comic.rs                 # Entidades Comic, ComicFormat, ComicMetadata
    │   ├── collection.rs            # Entidades Collection, CollectionPath
    │   ├── progress.rs              # Entidad ReadingProgress
    │   ├── device.rs                # Entidad TrustedDevice
    │   ├── ports.rs                 # Traits ComicFileReader, Repositorios, ImageCache
    │   └── errors.rs                # DomainError y enumeraciones de fallos
    ├── application/                 # CASOS DE USO Y FACHADA
    │   ├── mod.rs
    │   ├── facade.rs                # ComicFacade (interfaz simplificada para UI y API)
    │   ├── catalog_service.rs       # Gestión de colecciones e indexación
    │   └── reader_service.rs        # Lectura, extracción y entrega de páginas
    ├── adapters/                    # PUERTOS Y ADAPTADORES
    │   ├── mod.rs
    │   ├── inbound/                 # ADAPTADORES DE ENTRADA (PRIMARY)
    │   │   ├── mod.rs
    │   │   ├── desktop_ui/          # GUI de escritorio en Iced (TEA)
    │   │   │   ├── mod.rs
    │   │   │   ├── sidebar.rs
    │   │   │   ├── comic_grid.rs
    │   │   │   ├── reader.rs
    │   │   │   ├── metadata_editor.rs
    │   │   │   ├── collection_editor.rs
    │   │   │   └── trusted_devices.rs
    │   │   ├── rest_api/            # Servidor Axum REST y SPA Web
    │   │   │   ├── mod.rs
    │   │   │   ├── routes.rs
    │   │   │   ├── auth.rs
    │   │   │   └── spa.rs
    │   │   └── opds/                # Feed Atom XML OPDS 1.2
    │   │       ├── mod.rs
    │   │       └── feed_builder.rs
    │   └── outbound/                # ADAPTADORES DE SALIDA (SECONDARY)
    │       ├── mod.rs
    │       ├── persistence/         # Repositorio SQLite con sqlx
    │       │   ├── mod.rs
    │       │   ├── sqlite_repo.rs
    │       │   └── schema.rs
    │       ├── file_readers/        # Lectores de formatos comprimidos
    │       │   ├── mod.rs
    │       │   ├── cbz_adapter.rs   # Lector ZIP (zip crate)
    │       │   └── cbr_adapter.rs   # Lector RAR (unrar crate)
    │       └── cache/               # Proxy de caché en disco
    │           ├── mod.rs
    │           └── disk_cache.rs    # Gestión de app/.cache/
    ├── ingestion/                   # MÓDULO DE ESCANEO REACTIVO
    │   ├── mod.rs
    │   ├── watcher.rs               # File Watcher (patrón Observador con notify)
    │   ├── worker.rs                # Trabajador asíncrono de escaneo e ingesta
    │   └── comic_info.rs            # Extracción de ComicInfo.xml
    └── network/                     # PROTOCOLOS Y RED LOCAL
        ├── mod.rs
        ├── mdns.rs                  # Descubrimiento de servicios LAN
        └── qr.rs                    # Generador de códigos QR e IP local
```

---

## 5. Registro de Complejidad y Trade-Offs

| Decisión Arquitectónica | Justificación | Alternativa Más Simple Rechazada y Razón |
| :--- | :--- | :--- |
| **Monolito Modular frente a Monolito Tradicional Plano** | Separa responsabilidades mediante módulos estrictos (`domain`, `application`, `adapters`), permitiendo evolucionar partes del sistema independientemente sin riesgo de acoplamiento espagueti. | *Mantener todos los structs en 4 archivos planos*: Rechazado porque al añadir OPDS, File Watcher y mDNS el código se volvería inmanejable y propenso a regresiones. |
| **Patrón Adaptador (`ComicFileReader`)** | Aísla los detalles de descompresión de `.cbz` y `.cbr`. Añadir PDF o EPUB en el futuro solo requerirá implementar un trait. | *Match de extensiones disperso en el código*: Rechazado porque mezclaría lógica de lectura de RAR y ZIP con la lógica de presentación y base de datos. |
| **Patrón Fachada (`ComicFacade`)** | La GUI de Iced y la API de Axum consumen un solo punto de entrada unificado para operaciones complejas de cómic, caché y metadatos. | *Llamadas directas desde los controladores al repositorio y al sistema de archivos*: Rechazado por violar el principio de responsabilidad única y duplicar flujos de descompresión. |
| **Caché en disco `app/.cache/`** | Agiliza el streaming de páginas a clientes móviles reduciendo el consumo de CPU y RAM. | *Caché puramente en RAM*: Rechazada para evitar riesgo de saturación de memoria anfitriona ante múltiples clientes leyendo a la vez. |
