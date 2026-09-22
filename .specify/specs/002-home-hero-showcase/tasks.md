# Tasks: 002-home-hero-showcase

**Input**: Design documents from `.specify/specs/002-home-hero-showcase/`  
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/, quickstart.md  
**Organization**: Tareas agrupadas por fases y User Stories (P1 a P2) para implementación y pruebas independientes.

---

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Ejecutable en paralelo (archivos independientes sin dependencias incompletas)
- **[Story]**: Mapeo con User Stories del spec (`[US1]`, `[US2]`, `[US3]`, `[US4]`, `[US5]`)
- Rutas de archivo exactas en cada tarea

---

## Phase 1: Setup (Infraestructura Compartida)

**Propósito**: Preparación de directorios locales y soporte de medios para el Showcase Hero.

- [x] T001 Crear directorio de almacenamiento multimedia en `app/data/media/collections/`
- [x] T002 [P] Asegurar inclusión de directivas `.gitignore` para omitir la carpeta de medios `app/data/media/` del repositorio

---

## Phase 2: Foundational (Prerrequisitos Bloqueantes)

**Propósito**: Modelos de datos, persistencia SQLite y capa de aplicación indispensables para todas las historias de usuario.

- [x] T003 Extender la entidad `Collection` con los campos opcionales `description`, `background_image_path` y `hero_image_path` en `app/src/domain/collection.rs`
- [x] T004 Implementar migración idempotente en `Database::initialize_schema` agregando las columnas `description`, `background_image_path` y `hero_image_path` a la tabla `collections` en `app/src/adapters/outbound/persistence/schema.rs`
- [x] T005 Implementar la consulta SQL parametrizada `get_recent_collections` y actualizar `save_collection` en `app/src/adapters/outbound/persistence/sqlite_repo.rs`
- [x] T006 Exponer `get_recent_collections` y métodos asociados en `CatalogService` en `app/src/application/catalog_service.rs`
- [x] T007 Exponer la recuperación de colecciones recientes a través de la fachada `ComicFacade` en `app/src/application/facade.rs`

**Punto de Control**: La base de datos y la capa de dominio/aplicación están preparadas; la implementación de las historias de usuario puede comenzar.

---

## Phase 3: User Story 1 - Experiencia Visual Hero Showcase en Pantalla de Inicio (Priority: P1) 🎯 MVP

**Objetivo**: Presentar una pantalla de bienvenida cinematográfica impactante con fondo panorámico, arte lateral derecho, badge, tipografía en relieve, sinopsis y botón "READ NOW".

**Prueba Independiente**: Ejecutar la app con al menos 1 colección existente; verificar que se renderice la composición multicapa con todos los elementos gráficos alineados a la maqueta de referencia.

- [x] T008 [US1] Crear el módulo de interfaz `app/src/adapters/inbound/desktop_ui/hero_showcase.rs` con la estructura base de componentes
- [x] T009 [US1] Implementar la composición multicapa con `stack!` (fondo panorámico con degradado/viñeteado y contenedor central) en `app/src/adapters/inbound/desktop_ui/hero_showcase.rs`
- [x] T010 [US1] Implementar el badge tipo credencial ("Hello my name is..."), el título principal estilizado con relieve/sombra roja y el bloque de sinopsis con glassmorphism en `app/src/adapters/inbound/desktop_ui/hero_showcase.rs`
- [x] T011 [US1] Implementar la posición y renderizado del personaje lateral derecho (`hero_image`) con corte/transparencia y *fallback* inteligente a portada flotante en `app/src/adapters/inbound/desktop_ui/hero_showcase.rs`
- [x] T012 [US1] Diseñar e implementar el botón poligonal estilizado "READ NOW" con corte angular que emite la acción de navegación en `app/src/adapters/inbound/desktop_ui/hero_showcase.rs`
- [x] T013 [US1] Conectar el componente Hero central con el estado global de la aplicación en `app/src/main.rs`

**Punto de Control**: User Story 1 (MVP) es completamente funcional y visible en la aplicación de escritorio.

---

## Phase 4: User Story 2 - Carrusel Inferior de Últimas 5 Colecciones con Rotación Automática (Priority: P1)

**Objetivo**: Proveer una franja horizontal inferior con las últimas 5 colecciones (más reciente a la izquierda), rotación cada 6 segundos con *Hover Pause*, animación cinemática *Slide & Cross-fade* a 60 FPS y selección manual inmediata.

**Prueba Independiente**: Con 2 o más colecciones, observar que la vista rote cada 6s automáticamente, que se pause al posar el ratón y que al hacer clic en una miniatura cambie de inmediato la vista Hero.

- [x] T014 [US2] Diseñar e implementar la franja horizontal inferior con tarjetas translúcidas de las últimas 5 colecciones en `app/src/adapters/inbound/desktop_ui/hero_showcase.rs`
- [x] T015 [US2] Implementar el temporizador de rotación de 6 segundos mediante suscripción reactiva en `app/src/main.rs`
- [x] T016 [US2] Implementar la lógica de *Hover Pause* (pausa al posar el cursor sobre Hero o carrusel y reanudación al salir) en `app/src/adapters/inbound/desktop_ui/hero_showcase.rs`
- [x] T017 [US2] Implementar la animación cinemática *Slide & Cross-fade* a 60 FPS (desplazamiento horizontal de elementos y disolución de fondo en ~400ms) en `app/src/adapters/inbound/desktop_ui/hero_showcase.rs`
- [x] T018 [US2] Implementar la selección interactiva por clic en las miniaturas del carrusel con salto instantáneo y reinicio del temporizador en `app/src/main.rs`

**Punto de Control**: User Story 2 añade dinamismo y navegación completa entre múltiples colecciones recientes.

---

## Phase 5: User Story 3 - Estado Vacío ("Crea tu primera colección") (Priority: P1)

**Objetivo**: Mostrar una pantalla negra elegante con el mensaje claro "Crea tu primera colección" cuando no existan colecciones en la base de datos.

**Prueba Independiente**: Ejecutar la app con una base de datos sin colecciones; verificar que se renderice el estado negro centrado y que el botón abra el modal de creación.

- [x] T019 [US3] Diseñar e implementar la vista de estado vacío con fondo negro puro y botón central "Crea tu primera colección" en `app/src/adapters/inbound/desktop_ui/hero_showcase.rs`
- [x] T020 [US3] Conectar la conmutación condicional entre estado vacío y Hero Showcase según `collections.is_empty()` en `app/src/main.rs`

**Punto de Control**: User Story 3 garantiza que la experiencia inicial de nuevos usuarios sea elegante y funcional.

---

## Phase 6: User Story 4 - Creación y Edición Extendida de Colección con Imágenes Hero (Priority: P2)

**Objetivo**: Permitir al usuario seleccionar opcionalmente imagen de fondo panorámica, imagen de personaje lateral y escribir la sinopsis al crear o editar colecciones.

**Prueba Independiente**: Abrir el diálogo de creación, seleccionar imágenes locales de fondo y lateral, escribir sinopsis y guardar; verificar que se copien a `data/media/collections/` y se visualicen en la vista Hero.

- [x] T021 [US4] Añadir selectores de archivo nativos para imagen de fondo panorámica e imagen de personaje lateral en `app/src/adapters/inbound/desktop_ui/collection_editor.rs`
- [x] T022 [US4] Añadir campo de texto para la sinopsis/descripción de la colección en `app/src/adapters/inbound/desktop_ui/collection_editor.rs`
- [x] T023 [US4] Implementar procesamiento asíncrono con `tokio::task::spawn_blocking` para optimizar y copiar imágenes a `app/data/media/collections/` en `app/src/adapters/inbound/desktop_ui/collection_editor.rs`
- [x] T024 [US4] Conectar el guardado de rutas relativas con el repositorio y refrescar la vista Hero en `app/src/main.rs`

**Punto de Control**: User Story 4 permite personalizar la apariencia visual de cualquier saga de cómics.

---

## Phase 7: User Story 5 - Barra Superior de Navegación Unificada con Menús Desplegables (Priority: P2)

**Objetivo**: Sustituir la barra lateral por una barra de menú superior fija con diseño integrado (`INICIO`, `COLECCIONES ▾`, `CÓMICS ▾`, `SERVIDOR / QR ▾`) e iniciar la ventana maximizada.

**Prueba Independiente**: Iniciar la app; verificar que arranque maximizada al monitor y que la barra superior permita desplegar submenús y navegar a cada sección.

- [x] T025 [US5] Crear el módulo `app/src/adapters/inbound/desktop_ui/top_bar.rs` con logotipo, menús temáticos y desplegables (`INICIO`, `COLECCIONES ▾`, `CÓMICS ▾`, `SERVIDOR / QR ▾`)
- [x] T026 [US5] Reemplazar la barra lateral por `top_bar` en el diseño general de la aplicación en `app/src/main.rs`
- [x] T027 [US5] Configurar `window::Settings { maximized: true, .. }` en la inicialización de la ventana en `app/src/main.rs`

**Punto de Control**: User Story 5 unifica la navegación global y asegura el arranque maximizado.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Propósito**: Verificación de rendimiento, pruebas unitarias y validación general de extremo a extremo.

- [x] T028 [P] Implementar pruebas unitarias de persistencia para `get_recent_collections` y migraciones en `app/src/adapters/outbound/persistence/sqlite_repo.rs`
- [x] T029 Medir y verificar la fluidez de animación a 60 FPS durante la transición y 0% de uso de CPU en reposo
- [x] T030 Ejecutar la guía de validación completa de extremo a extremo según `quickstart.md`
- [x] T031 [P] Auditoría final de código: verificar estricta ausencia de `unwrap()`, `panic!` y asegurar que el 100% de comentarios y textos estén en español

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Sin dependencias — inicio inmediato.
- **Foundational (Phase 2)**: Depende de Phase 1 — BLOQUEA todas las Historias de Usuario.
- **User Stories (Phases 3-7)**: Dependen de la finalización de la Fase 2 (Foundational).
  - Orden secuencial recomendado: US1 (MVP) → US2 → US3 → US4 → US5.
- **Polish (Phase 8)**: Depende de completar las historias de usuario deseadas.

---

## Implementation Strategy

### MVP Primero (User Story 1)
1. Completar Setup (Fase 1) y Foundational (Fase 2).
2. Implementar User Story 1 (Fase 3): Composición multicapa Hero Showcase.
3. Validar de forma independiente: la pantalla de inicio ya luce cinematográfica con 1 colección.

### Entrega Incremental
1. Añadir User Story 2: Carrusel de 5 recientes con rotación cada 6s y *Hover Pause*.
2. Añadir User Story 3: Manejo elegante del estado vacío.
3. Añadir User Story 4: Formulario de creación con selectores de imagen.
4. Añadir User Story 5: Barra superior fija con menús desplegables y arranque maximizado.
5. Ejecutar Fase 8 de pulido y verificación final.
