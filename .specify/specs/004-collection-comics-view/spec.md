# Feature Specification: Vista de Cómics de Colección y Lectura Integrada (004-collection-comics-view)

**Feature Branch**: `004-collection-comics-view`  
**Created**: 2026-09-22  
**Status**: Draft  
**Input**: User description: "usa la imagen como referencia, al apretar 'read more' o apretar la coleccion en el navbar, debe ir a la vista en donde estan los comic de la coleccion, ordenados en orden alfabetico y al presionar un comic se debe poder leer. Ordenalo de manera moderna entretenida pero sobria"

---

## Clarifications

### Session 2026-09-22

- Q: ¿Cómo debe estructurarse la navegación entre el Hero Showcase y la Vista de Cómics? → A: **Vista completa integrada (Opción A)**: La vista del Hero Showcase se oculta suavemente para presentar la grilla de cómics a pantalla completa en el contenedor principal de la SPA, con botón de retorno "← Volver al Inicio" y Navbar superior persistente.
- Q: ¿Qué información deben mostrar los badges/chips superpuestos en las portadas de los cómics? → A: **Chip de Tomo + Chip de Páginas (Opción A)**: Se incluye un chip con el número de entrega (`#1`, `#2` o `Vol. 1`) y un chip con la cantidad de páginas (`32 págs`) superpuestos sobre la carátula, ubicando el título formateado (máx 40 caracteres) en la base de la tarjeta.
- Q: ¿Qué sucede al llegar a la última página de un tomo en el visor de lectura? → A: **Continuidad con Siguiente Tomo (Opción A)**: El visor muestra un panel de finalización elegante con opción para abrir inmediatamente el siguiente cómic de la colección en orden alfabético o regresar a la grilla de la colección.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Navegación a la Vista de Cómics de la Colección (Priority: P1)

Como lector de cómics, cuando hago clic en el botón principal **"LEER MÁS"** en el Hero Showcase o selecciono una colección específica en el menú desplegable del Navbar, deseo que la interfaz me lleve directamente a una vista dedicada donde se exhiban todos los tomos y cómics pertenecientes a dicha colección, para poder explorar fácilmente su contenido sin depender de un modal flotante restrictivo.

**Why this priority**: Es la ruta principal de navegación para explorar los volúmenes de una colección y conecta la portada cinematográfica con el catálogo de lectura.

**Independent Test**: Puede probarse haciendo clic en "LEER MÁS" o en cualquier elemento de la lista de colecciones del Navbar; la pantalla principal transiciona fluidamente hacia la vista de catálogo de cómics de esa colección.

**Acceptance Scenarios**:
1. **Given** que el usuario está en la pantalla de inicio con una colección activa, **When** presiona el botón "LEER MÁS", **Then** el Hero Showcase se oculta y se visualiza la vista completa de cómics de esa colección con su encabezado y grilla.
2. **Given** que el usuario está en cualquier vista, **When** abre el menú desplegable "COLECCIONES" del Navbar y selecciona una colección, **Then** la interfaz carga y muestra inmediatamente la vista de cómics de la colección seleccionada.
3. **Given** que el usuario está en la vista de cómics de una colección, **When** presiona el botón "INICIO" o el logo en el Navbar (o un botón de retorno "← Volver"), **Then** regresa a la vista de inicio del Hero Showcase.

---

### User Story 2 - Grilla Moderna y Sobria de Portadas Ordenadas Alfabéticamente (Priority: P1)

Como usuario, deseo ver todos los cómics de la colección organizados en una grilla moderna, sobria y entretenida inspirada en interfaces de lectores de manga/cómics profesionales, con portadas verticales de alta calidad, títulos claros y ordenados alfabéticamente de la A a la Z.

**Why this priority**: Proporciona el valor estético y funcional central: encontrar rápida y visualmente cualquier entrega de la saga mediante su portada y metadata concisa.

**Independent Test**: Puede probarse cargando una colección con múltiples archivos `.cbz`/`.cbr`; la grilla muestra las tarjetas alineadas con aspecto ~2:3, badges superiores (ej. número de tomo, cantidad de páginas), ordenadas por título de la A a la Z.

**Acceptance Scenarios**:
1. **Given** una colección con tomos indexados, **When** se presenta la grilla de cómics, **Then** los elementos se listan ordenados alfabéticamente por título de forma ascendente (A-Z).
2. **Given** una tarjeta de cómic en la grilla, **When** el usuario posa el cursor sobre ella (*hover*), **Then** la tarjeta experimenta una microinteracción sutil de elevación y resplandor sobrio sin romper la armonía visual.
3. **Given** un cómic con portada extraída en base de datos, **When** se renderiza la tarjeta, **Then** se muestra la imagen de portada nítida con esquinas redondeadas y badges superpuestos con información clave (número de tomo o páginas).

---

### User Story 3 - Lectura Inmediata al Seleccionar un Cómic (Priority: P1)

Como lector, cuando presiono cualquier tarjeta de cómic dentro de la grilla de la colección, deseo que se abra de forma instantánea el visor de lectura a pantalla completa en la primera página (o en el progreso guardado), con controles táctiles y de teclado rápidos y fluidos.

**Why this priority**: Es la acción final de valor del sistema: leer el contenido digital de forma cómoda y sin fricciones.

**Independent Test**: Puede probarse haciendo clic en cualquier tarjeta de la grilla; el visor de lectura se despliega de inmediato mostrando la página inicial del tomo y permitiendo avanzar con teclado o clics.

**Acceptance Scenarios**:
1. **Given** la grilla de cómics de una colección, **When** el usuario hace clic en una tarjeta de cómic, **Then** se abre el lector a pantalla completa cargando la página correspondiente.
2. **Given** el visor de lectura abierto, **When** el usuario pulsa las teclas Flecha Derecha / Espacio o el botón "Siguiente", **Then** avanza secuencialmente a la página siguiente.
3. **Given** el visor de lectura abierto, **When** el usuario presiona la tecla Escape o el botón de cierre "✕", **Then** el visor se cierra y regresa a la grilla de cómics de la colección.

---

### User Story 4 - Estado Vacío Informativo y Acción de Escaneo (Priority: P2)

Como usuario que acaba de asociar una carpeta o crear una colección sin cómics todavía procesados, deseo ver un estado amigable que me informe de la situación y me permita re-escanear la biblioteca o editar la carpeta con un solo clic.

**Why this priority**: Asegura que el usuario nunca quede frente a una pantalla vacía o confusa si la carpeta aún no ha terminado de indexar o no contiene cómics.

**Independent Test**: Puede probarse abriendo una colección con 0 cómics; la vista muestra un panel estilizado con botón para re-escanear o examinar carpeta.

**Acceptance Scenarios**:
1. **Given** una colección sin cómics indexados, **When** se abre su vista de cómics, **Then** se muestra un mensaje informativo sobrio con un botón de acción para "Re-escanear Carpeta" o "Editar Colección".

---

### Edge Cases

- **Colección con títulos numéricos o caracteres especiales**: El ordenamiento alfabético debe manejar números y tildes de forma natural (ej. *Batman #1, Batman #2, Batman: Año Uno*).
- **Cómics sin miniatura/portada procesada en base de datos**: Mostrar un placeholder gráfico estilizado con el icono del formato y el título visible en la tarjeta.
- **Títulos de cómics largos**: Los títulos deben truncarse elegantemente a un máximo de 40 caracteres con puntos suspensivos según las reglas de negocio del sistema.
- **Colección con gran volumen de cómics (100+ tomos)**: La grilla debe mantener un rendimiento fluido de renderizado y desplazamiento suave sin caídas de cuadros (*frame drops*).

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Al accionar el botón "LEER MÁS" del Hero Showcase o cualquier colección en el menú del Navbar, el sistema DEBE cambiar la vista activa de la aplicación a la **Vista de Cómics de la Colección** (`#collection-comics-view`).
- **FR-002**: La vista de cómics DEBE incluir una cabecera con el nombre de la colección, protagonista, sinopsis resumida, contador total de tomos y un botón de retorno rápido a la vista de Inicio.
- **FR-003**: La grilla de cómics DEBE presentar las tarjetas de cada tomo ordenadas alfabéticamente de la A a la Z según su título (`title`).
- **FR-004**: Cada tarjeta de cómic DEBE mostrar su portada vertical con relación de aspecto ~2:3, esquinas redondeadas, título truncado a un máximo de 40 caracteres y chip/badge informativo (número de tomo o total de páginas).
- **FR-005**: Al hacer clic en cualquier parte de la tarjeta de cómic, el sistema DEBE abrir el lector de cómics a pantalla completa cargando el volumen seleccionado.
- **FR-006**: El diseño visual de la vista DEBE adherirse a la estética moderna, sobria y oscura (*Dark Mode* / *Glassmorphism*) de la aplicación, inspirándose en las tarjetas y disposición de la imagen de referencia.
- **FR-007**: La navegación con tecla Escape o botón de cierre en el visor de lectura DEBE retornar limpiamente a la vista de cómics de la colección.

---

### Key Entities

- **Colección (`Collection`)**: Agrupación temática o de franquicia que contiene metadatos (nombre, protagonista, descripción, imágenes de fondo y héroe) y una lista asociada de cómics indexados.
- **Cómic (`ComicSummaryDto`)**: Representación de un volumen indexado con atributos de identificación (`id`, `collection_id`, `title`, `issue_number`, `page_count`, portada en miniatura y formato).

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: La transición entre el Hero Showcase y la Vista de Cómics de la Colección se completa en menos de **100 milisegundos** en la interfaz local.
- **SC-002**: El 100% de los cómics de la colección se presentan en estricto orden alfabético ascendente (A-Z).
- **SC-003**: La apertura del visor de lectura al hacer clic en cualquier tarjeta de cómic responde de manera instantánea (inferior a **200 milisegundos** para cargar la portada o página inicial).
- **SC-004**: Las imágenes de portadas y badges se adaptan fluidamente a distintas resoluciones de ventana manteniendo la proporción vertical y nitidez visual.

---

## Assumptions

- Los archivos `.cbz` y `.cbr` se encuentran indexados en la base de datos SQLite local de la aplicación.
- Las miniaturas de portada se obtienen directamente de los bytes almacenados en la base de datos o se generan en demanda a través de la API de Tauri.
- La aplicación opera 100% en modo local (*Offline-First*) sin dependencias de servicios externos ni fuentes externas en la red.
