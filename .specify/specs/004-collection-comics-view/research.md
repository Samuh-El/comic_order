# Phase 0 Research: Vista de Cómics de Colección y Lectura Integrada (004-collection-comics-view)

## Decisiones Técnicas y Arquitectónicas

### 1. Ordenamiento Alfabético de Cómics (A-Z)
- **Decisión**: El repositorio SQLite (`sqlite_repo.rs`) ordenará los cómics de una colección por `ORDER BY title COLLATE NOCASE ASC`. En el frontend (`script.js`), se aplicará una ordenación adicional con `localeCompare(..., { sensitivity: 'base', numeric: true })` para garantizar que los números en títulos (ej. *"Tomo 1"*, *"Tomo 2"*, *"Tomo 10"*) se ordenen de manera natural.
- **Razón**: Proporciona coherencia total entre backend y frontend, asegurando una experiencia predecible e intuitiva sin importar la combinación de mayúsculas/minúsculas o numeración.
- **Alternativas consideradas**: Ordenar únicamente por número de edición (`issue_number`), pero muchos cómics empaquetados solo poseen nombre de archivo o título sin metadatos de entrega.

### 2. Estructura y Transición de la Vista Integrada (SPA)
- **Decisión**: Implementar una sección `#collection-comics-view` en `index.html` que conviva con `#hero-main-container`. Al activarse mediante la función `showCollectionComicsView(collectionId)`, el Hero Showcase se oculta suavemente (`display: none` con fade-in en la vista de cómics) y se despliega la cabecera cinemática de la saga con su grilla de portadas, un botón *"← Volver al Inicio"* y el Navbar superior persistente.
- **Razón**: Elimina la limitación de espacio de los modales flotantes tradicionales y maximiza el área visual para exhibir múltiples tomos con portadas de alta calidad, alineado a la decisión de la fase de clarificación.
- **Alternativas consideradas**: Modal clásico (`dossier-modal`) o panel inferior deslizable; descartados por resultar estrechos para colecciones de decenas de tomos.

### 3. Renderizado de Tarjetas de Cómic con Chips Superpuestos (Manga Reader UI)
- **Decisión**: Cada tarjeta de cómic utilizará un contenedor con relación de aspecto `2:3` (`aspect-ratio: 2 / 3`), borde redondeado (`border-radius: 8px`), sombra suave y elevación interactiva al hover (`transform: translateY(-4px)`).
- **Badges/Chips superpuestos**:
  - Chip inferior izquierdo con badge verde/cian para número de tomo (ej. `#01` o `T. 1`).
  - Chip inferior derecho para el conteo de páginas (ej. `32 págs`).
  - Título formateado en la base de la tarjeta truncado a un máximo de 40 caracteres con puntos suspensivos.
- **Razón**: Réplica fiel del lenguaje visual moderno observado en la imagen de referencia provista por el usuario.

### 4. Flujo de Lectura y Panel de Continuidad
- **Decisión**: Al hacer clic en cualquier tarjeta de la grilla, se invoca `openReader(comicId, title)`. Si el usuario navega hasta la última página del cómic, en lugar de bloquear el avance, el visor mostrará un panel estilizado de finalización con dos acciones claras: *"Siguiente Tomo: [Nombre]"* y *"Volver a la Colección"*.
- **Razón**: Fomenta la inmersión y la continuidad en maratones de lectura sin forzar al usuario a salir manualmente para abrir el siguiente volumen.
