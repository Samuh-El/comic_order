# Implementation Plan: Vista de Cómics de Colección y Lectura Integrada (004-collection-comics-view)

**Branch**: `004-collection-comics-view` | **Date**: 2026-09-22 | **Spec**: [.specify/specs/004-collection-comics-view/spec.md](file:///c:/Workspace/comic_order/.specify/specs/004-collection-comics-view/spec.md)

## Summary

Implementar la vista dedicada y completa de exploración de cómics de una colección (`#collection-comics-view`), activable al hacer clic en el botón principal **"LEER MÁS"** del Hero Showcase o al seleccionar una colección en el menú superior del Navbar. La vista exhibirá una cabecera cinemática con la información de la colección, un botón de retorno rápido *"← Volver al Inicio"* y una grilla moderna inspirada en lectores de manga/cómics profesionales con portadas verticales (~2:3), chips superpuestos (número de tomo y páginas), ordenamiento alfabético estricto (A-Z) y apertura instantánea del visor de lectura al hacer clic en cualquier tomo.

---

## Technical Context

- **Primary Dependencies**: Rust 1.80+ (Edition 2021), Tauri v2 (`@tauri-apps/api`), Vanilla HTML5/CSS3/JavaScript ES6+ sin frameworks ni CDNs externas.
- **Storage**: SQLite 3 embebido (`app/data/comic.db`) con SQLx en modo WAL, tablas `collections`, `collection_paths`, `comics`.
- **Testing**: `cargo test` para tests unitarios y de integración de repositorios y adaptadores.
- **Target Platform**: Windows 11 / 10 Desktop vía Tauri v2 (WebView2).
- **Project Type**: Aplicación de escritorio modular híbrida (Rust Backend + Tauri WebView UI).
- **Performance Goals**: Transición de vistas en < 100ms; carga de portadas y apertura del lector en < 200ms.
- **Constraints**: 100% *Offline-First* (sin CDNs ni librerías externas), sin llamadas bloqueantes en el hilo de UI, textos en español.

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Principio 1 (Offline-First)**: Sin dependencias externas ni telemetría.
- [x] **Principio 2 (No bloquear UI)**: Toda lectura y extracción de cómics asíncrona mediante Tokio / Task.
- [x] **Principio 3 (Extracción en RAM)**: Páginas servidas directamente como buffers / Data URLs Base64.
- [x] **Principio 6 (SQL parametrizado)**: Todas las consultas con `.bind()` de `sqlx`.
- [x] **Principio 7 (Sin pánico en runtime)**: Manejo canónico con `Result` / `Option`.
- [x] **Principio 8 (Idioma Español)**: 100% en español.
- [x] **Principio 10 (Arquitectura Hexagonal)**: Dominio puro desacoplado de la interfaz.

---

## Project Structure

### Documentation (this feature)

```text
.specify/specs/004-collection-comics-view/
├── spec.md                  # Especificación de requerimientos y clarificaciones
├── plan.md                  # Plan técnico de implementación y arquitectura
├── research.md              # Decisiones de diseño y mejores prácticas
├── data-model.md            # Entidades, DTOs y modelos de estado
├── quickstart.md            # Escenarios de verificación y prueba
├── checklists/
│   └── requirements.md      # Checklist de calidad de la especificación
├── contracts/
│   └── tauri-ipc-contracts.md # Contratos de comandos IPC de Tauri
└── tasks.md                 # Tareas desglosadas por historia de usuario
```

### Source Code Modificado

```text
app/
├── src/
│   ├── adapters/
│   │   ├── inbound/
│   │   │   └── tauri_bridge/
│   │   │       └── commands.rs  # Asegurar orden alfabético y entrega de cover_base64
│   │   └── outbound/
│   │       └── persistence/
│   │           └── sqlite_repo.rs # Consulta SQL con ORDER BY title COLLATE NOCASE ASC
├── ui/
│   ├── index.html               # Estructura HTML de #collection-comics-view y header de saga
│   ├── styles.css               # Estilos modernos para grilla, tarjetas 2:3, chips y microinteracciones
│   └── script.js                # Lógica de navegación entre vistas, ordenamiento natural A-Z y lector
```
