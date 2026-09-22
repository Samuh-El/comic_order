# Implementation Plan: Adopción de Tauri y Nueva Interfaz Gráfica Comic Showcase

**Branch**: `003-tauri-hero-interface` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

**Input**: Especificación de requerimientos de la feature `003-tauri-hero-interface`.

---

## Summary

Migrar la capa de presentación de escritorio hacia **Tauri v2**, implementando la interfaz visual de cómic inmersiva basada en el prototipo validado de `docs/example`. Se preserva el backend modular en Rust bajo Arquitectura Hexagonal (servicios de catálogo, lectura, escaneo, persistencia SQLite con SQLx, servidor REST Axum y OPDS), extendiendo el modelo de colección para soportar el campo `protagonist` y conectando el frontend mediante comandos IPC asíncronos (`#[tauri::command]`).

---

## Technical Context

**Primary Dependencies**:
- `tauri` (v2), `tauri-build` (v2), `serde`, `serde_json`, `tokio` (v1.x full).
- Frontend: HTML5 semántico, CSS3 moderno (Custom Properties, Glassmorphism, CSS Grid), JavaScript ES6+ autocontenido (sin frameworks pesados ni CDNs).
- Backend: `sqlx` (v0.8, SQLite con WAL), `axum` (v0.7), `zip` (v2.2), `unrar` (v0.5), `image` (v0.25), `tracing`.

**Storage**:
- SQLite 3 local embebido en `app/data/comic.db` con modo WAL y migración idempotente para la columna `protagonist`.

**Testing**:
- `cargo test` para pruebas unitarias de dominio, persistencia, comandos y servicios de catálogo.
- Pruebas E2E de interfaz de usuario mediante `quickstart.md`.

**Target Platform**:
- Windows 10/11 x64 (MSVC) con WebView2 nativo.

**Project Type**:
- Monolito Modular de escritorio y servidor LAN (*Offline-First*).

**Performance Goals**:
- Arranque de la interfaz en menos de 1.0 segundo.
- Conmutación de colecciones en el Dock inferior en menos de 50 milisegundos.
- Consumo de CPU en reposo inferior al 1.0%.

**Constraints**:
- 100% *Offline-First* (sin CDNs, fuentes externas ni telemetría).
- Idioma 100% en español (código, comentarios, docstrings y documentación).
- 0 `panic!`/`unwrap()` no controlados en tiempo de ejecución.

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Regla 1 (Sin nube ni telemetría)**: Cumple. Todo el procesamiento y almacenamiento es estrictamente local y offline.
- [x] **Regla 2 (No bloquear hilo principal)**: Cumple. Toda I/O de disco y base de datos corre en tareas asíncronas de Tokio mediante comandos IPC de Tauri.
- [x] **Regla 3 (No extraer cómics completos a disco)**: Cumple. La extracción de páginas se realiza bajo demanda directamente en memoria RAM.
- [x] **Regla 4 (Validación de tokens en API)**: Cumple. La API REST mantiene intacto su middleware de autenticación por tokens y dispositivos de confianza.
- [x] **Regla 5 (Sin librerías externas ni CDNs en frontend)**: Cumple. Los assets, fuentes (Bebas Neue, Montserrat, Inter) y scripts residen localmente en el paquete de la app.
- [x] **Regla 6 (Consultas SQL parametrizadas)**: Cumple. Todas las consultas a SQLite continúan utilizando `.bind()`.
- [x] **Regla 7 (Sin pánico descontrolado)**: Cumple. Todo manejo de errores mediante `Result`/`Option` y `tracing`.
- [x] **Regla 8 (Idioma español)**: Cumple.
- [x] **Regla 9 (Flujo SDD)**: Cumple. Spec, Plan y Tasks formalizados.
- [x] **Regla 10 (Encapsulación Hexagonal)**: Cumple. El dominio y los servicios de aplicación no dependen de Tauri; Tauri actúa únicamente como un adaptador de entrada (*Inbound Adapter*).

---

## Project Structure

### Documentation (this feature)

```text
.specify/specs/003-tauri-hero-interface/
├── spec.md              # Especificación funcional y de negocio
├── plan.md              # Plan de arquitectura e implementación (este archivo)
├── research.md          # Investigación técnica y decisiones de arquitectura
├── data-model.md        # Modelo de entidades y esquema SQLite
├── quickstart.md        # Guía de ejecución y validación
├── contracts/
│   └── tauri-commands.contract.md # Especificación de comandos IPC de Tauri
└── checklists/
    └── requirements.md  # Checklist de calidad de requerimientos
```

### Source Code Layout

```text
app/
├── Cargo.toml                   # Dependencias Rust (incluyendo tauri v2)
├── tauri.conf.json              # Configuración de ventana, seguridad y assets de Tauri
├── build.rs                     # Script de construcción de Tauri (tauri_build)
├── ui/                          # Frontend autocontenido (HTML5, CSS3, JS)
│   ├── index.html               # Estructura principal adaptada de docs/example
│   ├── styles.css               # Estilos globales, temas de cómic y animaciones
│   ├── script.js                # Lógica de interacción, Dock, Navbar y llamadas IPC
│   └── assets/                  # Fuentes locales, iconos, texturas e imágenes base
│       ├── fonts/               # Bebas Neue, Montserrat, Inter (.woff2 / .ttf)
│       └── images/              # Overlays, viñetas, texturas y logos
├── src/
│   ├── main.rs                  # Punto de entrada de la aplicación y runtime de Tauri
│   ├── domain/                  # Entidades puras y reglas de negocio
│   │   ├── collection.rs        # Entidad Collection con campo 'protagonist'
│   │   ├── comic.rs
│   │   ├── device.rs
│   │   ├── progress.rs
│   │   └── ports.rs             # Puertos de entrada y salida
│   ├── application/             # Servicios de aplicación y Fachada
│   │   ├── facade.rs            # ComicFacade
│   │   ├── catalog_service.rs   # Gestión de catálogo y colecciones
│   │   └── reader_service.rs    # Extracción y renderizado de páginas
│   ├── ingestion/               # Escaneo asíncrono y extracción de ComicInfo.xml
│   ├── network/                 # Servidor mDNS y generación de códigos QR
│   └── adapters/
│       ├── inbound/
│       │   ├── tauri_bridge/    # Comandos IPC #[tauri::command] para la UI
│       │   ├── rest_api/        # Servidor HTTP Axum para lectura remota
│       │   └── opds/            # Feed estándar OPDS
│       └── outbound/
│           ├── persistence/     # SQLite con SQLx y migraciones idempotentes
│           ├── file_readers/    # Adaptadores de descompresión CBZ y CBR
│           └── cache/           # Caché de imágenes en disco
└── data/                        # Base de datos SQLite y medios desacoplados
```

**Structure Decision**: Monolito Modular con arquitectura Hexagonal. La interfaz de usuario reside en `app/ui/` y se enlaza mediante el adaptador `adapters/inbound/tauri_bridge/`, manteniendo el dominio y la persistencia completamente desacoplados.

---

## Implementation Phases

### Fase 1: Extensión del Dominio y Persistencia
- Modificar `domain/collection.rs` para añadir el campo `protagonist: Option<String>` con sus invariantes de validación.
- Actualizar `adapters/outbound/persistence/schema.rs` con migración idempotente para la columna `protagonist`.
- Actualizar `adapters/outbound/persistence/sqlite_repo.rs` (`get_all`, `get_by_id`, `get_recent`, `create_with_details`, `update`) para leer y guardar `protagonist`.
- Actualizar `application/catalog_service.rs` y `application/facade.rs`.

### Fase 2: Configuración de Tauri v2 y Adaptador IPC
- Configurar dependencias en `app/Cargo.toml` (`tauri`, `tauri-build`, `serde`, etc.).
- Crear `app/tauri.conf.json` con configuración de ventana maximizada, título, iconos y directivas de seguridad CSP local.
- Crear el adaptador de entrada `adapters/inbound/tauri_bridge/` con todos los comandos IPC (`get_collections`, `save_collection`, `delete_collection`, `get_comics_by_collection`, `get_comic_page`, `scan_monitored_paths`, `get_server_status`, `toggle_server`, `get_trusted_devices`, `add_trusted_device`, `remove_trusted_device`, `pick_image_file`).
- Integrar el builder de Tauri en `app/src/main.rs`.

### Fase 3: Construcción de la Interfaz Visual Webview
- Adaptar `docs/example/index.html` en `app/ui/index.html`, estructurando:
  - Navbar superior con menús desplegables (`HOME`, `COLECCIONES ▾`, `CÓMICS ▾`, `SERVIDOR / QR ▾`, conmutador de audio).
  - Escenario central Hero Showcase con bloque de protagonista rojo en relieve, caja de sinopsis con Glassmorphism y botón CTA ("READ MORE").
  - Dock inferior flotante con soporte de scroll horizontal continuo.
  - Modales: Dossier de colección/cómics, Formulario de Colección (con campo Protagonista), Código QR y Dispositivos de Confianza.
  - Visor de lectura de cómics embebido.
- Adaptar `docs/example/styles.css` en `app/ui/styles.css` con tipografías locales, animaciones a 60 FPS y temas dinámicos.
- Desarrollar `app/ui/script.js` con las llamadas a `window.__TAURI__.core.invoke` para sincronizar el estado reactivo con el backend en Rust.

### Fase 4: Verificación y Pruebas
- Ejecutar pruebas unitarias completas con `cargo test`.
- Validar los escenarios end-to-end descritos en `quickstart.md` (arranque limpio sin colecciones, creación de colecciones, dock, dossier y navbar).
