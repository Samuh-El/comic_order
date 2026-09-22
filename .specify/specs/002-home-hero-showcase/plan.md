# Implementation Plan: 002-home-hero-showcase

**Branch**: `002-home-hero-showcase` | **Date**: 2026-09-15 | **Spec**: [.specify/specs/002-home-hero-showcase/spec.md](spec.md)

**Input**: Feature specification from `.specify/specs/002-home-hero-showcase/spec.md`

---

## Summary

Implementar la nueva interfaz de usuario cinematográfica **Home Hero Showcase** para la aplicación de escritorio en Rust con Iced v0.13, basada fielmente en la maqueta de referencia. La solución incorpora una barra superior fija con menús desplegables (`INICIO`, `COLECCIONES ▾`, `CÓMICS ▾`, `SERVIDOR / QR ▾`), vista central Hero con composición multicapa (`stack!`), fondo panorámico, personaje lateral recortado, badge tipo credencial, tipografía en relieve, panel de sinopsis con glassmorphism y botón angular "READ NOW". El sistema incluye un carrusel inferior con las últimas 5 colecciones ordenadas cronológicamente, rotación automática cada 6 segundos con *Hover Pause* interactivo, transición cinemática *Slide & Cross-fade* a 60 FPS, estado vacío elegante ("Crea tu primera colección"), formulario extendido con persistencia en `data/media/collections/` y arranque maximizado de ventana.

---

## Technical Context

**Primary Dependencies**: 
- `iced` v0.13 con features `["image", "tokio", "canvas", "svg"]` para la GUI acelerada por hardware y The Elm Architecture (TEA).
- `sqlx` v0.8 (SQLite asíncrono en modo WAL) para persistencia relacional.
- `tokio` v1.x (runtime asíncrono completo) para temporizadores y tareas en segundo plano.
- `rfd` (Rust File Dialog) asíncrono para selección nativa de imágenes.
- `image` v0.25 para redimensionamiento y optimización de medios.

**Storage**: 
- Base de datos SQLite local en `app/data/comic.db`.
- Almacenamiento desacoplado de medios en `app/data/media/collections/` con rutas relativas en SQLite.

**Testing**: 
- `cargo test` para pruebas unitarias de modelos y migraciones de persistencia.
- Verificación interactiva de componentes gráficos y transiciones según `quickstart.md`.

**Target Platform**: 
- Windows 11 (MSVC, enlace estático de CRT).

**Project Type**: 
- Monolito modular de escritorio con arquitectura hexagonal.

**Performance Goals**: 
- Transición *Slide & Cross-fade* suave a 60 FPS durante los 400 ms de animación.
- Suspensión de la suscripción de 60 FPS en reposo para 0% de sobrecarga de CPU/GPU.
- Tiempo de conmutación manual de colecciones < 100 ms.

**Constraints**: 
- 100% *Offline-first* (sin telemetría ni librerías cloud).
- Cero bloqueos en el hilo principal de la UI (I/O de archivos en `spawn_blocking`).
- Todo el código, comentarios y documentación exclusivamente en español.

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio Constitucional | Estado | Justificación |
| :--- | :---: | :--- |
| **1. Offline-First & Privacidad** | ✅ Pasa | Todo opera localmente sin servicios en la nube ni telemetría externa. |
| **2. No bloquear el hilo de la UI** | ✅ Pasa | El procesamiento y copia de imágenes se delega a `spawn_blocking` con Tokio. |
| **3. Extracción en memoria** | ✅ Pasa | No se descomprimen cómics completos a disco. |
| **4. Consultas SQL parametrizadas** | ✅ Pasa | Todas las consultas en SQLite usan `.bind()` sin interpolación de cadenas. |
| **5. Manejo idiomático de errores** | ✅ Pasa | Se utiliza `Result`/`Option` con logueo estructurado `tracing` sin `panic!` ni `unwrap()`. |
| **6. Idioma exclusivo en Español** | ✅ Pasa | Todos los identificadores en UI, comentarios y documentos en español. |
| **7. Arquitectura Hexagonal** | ✅ Pasa | El dominio permanece puro; la UI reside en `adapters/inbound/desktop_ui/`. |

---

## Project Structure

### Documentation (this feature)

```text
.specify/specs/002-home-hero-showcase/
├── spec.md                       # Especificación formal y clarificaciones
├── plan.md                       # Este plan de implementación
├── research.md                   # Fase 0: Decisiones arquitectónicas
├── data-model.md                 # Fase 1: Modelos de datos y esquema SQLite
├── quickstart.md                 # Fase 1: Guía de validación E2E
├── contracts/
│   ├── ui-components.contract.md # Contratos TEA para Iced
│   └── persistence.contract.md   # Contratos del repositorio SQLite
└── checklists/
    └── requirements.md           # Checklist de calidad
```

### Source Code (repository layout)

```text
app/src/
├── domain/
│   └── collection.rs                        # [MODIFICAR] Nuevos campos en struct Collection
├── adapters/
│   ├── outbound/
│   │   └── persistence/
│   │       ├── schema.rs                    # [MODIFICAR] Migración idempotente para description, bg y hero
│   │       └── sqlite_repo.rs               # [MODIFICAR] get_recent_collections y save_collection
│   └── inbound/
│       └── desktop_ui/
│           ├── mod.rs                       # [MODIFICAR] Integración de top_bar y hero_showcase
│           ├── top_bar.rs                   # [NUEVO] Barra superior fija con menús desplegables
│           ├── hero_showcase.rs             # [NUEVO] Componente Home Hero Showcase multicapa
│           └── collection_editor.rs         # [MODIFICAR] Selectores de fondo, arte lateral y sinopsis
└── main.rs                                  # [MODIFICAR] maximized: true en window::Settings
```

**Structure Decision**: Monolito Modular bajo Arquitectura Hexagonal. Los nuevos componentes de presentación se aíslan en `adapters/inbound/desktop_ui/`, la migración en `adapters/outbound/persistence/`, y la entidad pura en `domain/collection.rs`.

---

## Complexity Tracking

> **No existen violaciones a la constitución ni complejidades injustificadas.**
