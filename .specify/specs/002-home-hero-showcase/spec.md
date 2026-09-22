# Feature Specification: 002-home-hero-showcase

**Feature Branch**: `002-home-hero-showcase`  
**Created**: 2026-09-15  
**Status**: Draft  
**Input**: User description: "vamos a cambiar la UI por esta; todas las opciones aparecerán en el menuy de arriba, abajo, donde estan los nombres, son los nombres de las ultimas 5 colecciones; de izquierda a derecha se organizan de la mas reciente a la mas antigua (solo las ultimas 5), en medio se ve la imagen de fondo, imagen de la derecha, tituloy descripcion. EL boton 'READ NOW' permiet ir a la vista de organizacion de la coleccion (pero eso lo veremos mas adelante ahora concentrate en esta vista de home). SI NO HAY COELCCIONES, SE VE EN NEGRO Y DIRA CREA TU PRIMERA COLECCION. Al crearla se pide imagen de fondo e imagen lateral (ambas son opcionales) ademas de los demás datos que ya se pedían para crear la colección. Además, el software ahora se iniciará maximizado a tamaño de la pantalla actual. LA UI ES LA MISMA DE LA IMAGEN, COPIALA TAL CUAL. Cada 6 segundos, cuando hay mas de 1 coleccion, hace una transición al lado para mostrar la info de la siguiente colección. Las opciones actuales del sistema están en el menú superior."

## Clarifications

### Session 2026-09-15

- Q: ¿Cómo se organizan y estructuran las opciones de navegación en la barra superior? → A: Menús con desplegables temáticos: `INICIO`, `COLECCIONES ▾` (Ver todas, Nueva colección, Re-escanear), `CÓMICS ▾` (Biblioteca, Últimos leídos), `SERVIDOR / QR ▾` (Estado, Ver QR, Clientes conectados).
- Q: ¿Cómo se almacenan y persisten las imágenes de fondo y lateral? → A: Copia local optimizada en disco (`app/data/media/collections/`) con rutas relativas guardadas en SQLite.
- Q: ¿Cómo debe comportarse el temporizador de 6 segundos del carrusel ante la interacción del usuario? → A: Pausa automática por cursor (*Hover Pause*) sobre el área Hero o tarjetas inferiores, reanudándose al salir; reinicio inmediato del ciclo de 6 segundos ante clic manual.
- Q: ¿Qué apariencia adopta la vista Hero si una colección no tiene imagen de fondo o lateral? → A: *Fallback* temático inteligente: si falta fondo, se usa degradado cinematográfico oscuro o portada difuminada; si falta arte lateral, se muestra la portada del primer cómic con marco flotante en ángulo o se expande el bloque tipográfico con balance visual.
- Q: ¿Qué efecto visual debe tener la transición entre colecciones en el carrusel? → A: Desplazamiento lateral con fundido cruzado (*Slide & Cross-fade*) para los elementos en primer plano y fondo panorámico.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Experiencia Visual Hero Showcase en Pantalla de Inicio (Priority: P1)

Como lector de cómics, al abrir la aplicación maximizada deseo ver una pantalla principal impactante con diseño de póster cinematográfico (Home Hero Showcase), mostrando la colección destacada con su fondo panorámico, personaje o arte lateral recortado, etiqueta tipo credencial con el nombre, título en relieve estilizado, sinopsis y botón de lectura ("READ NOW" / "READ MORE"), para disfrutar de una presentación estética inmersiva de mis colecciones.

**Why this priority**: Es la pantalla de bienvenida y el núcleo visual de la nueva experiencia solicitada por el usuario, transformando radicalmente la interacción inicial con la biblioteca de cómics.

**Independent Test**: Puede probarse de forma independiente iniciando la aplicación con al menos una colección existente que disponga de imágenes y textos; se verifica que la ventana se abra maximizada y los elementos gráficos (fondo, lateral, títulos, botón) se rendericen exactamente en las posiciones y estilos de la referencia visual.

**Acceptance Scenarios**:
1. **Given** que existen colecciones registradas en la biblioteca, **When** el usuario inicia la aplicación, **Then** la ventana se abre maximizada al tamaño del monitor actual y presenta la colección más reciente en la vista Hero central con su fondo panorámico, arte lateral derecho, badge superior, título estilizado con relieve y botón de llamada a la acción ("READ NOW").
2. **Given** la vista Hero central mostrada, **When** el usuario hace clic en el botón principal "READ NOW" (o "READ MORE"), **Then** el sistema emite el evento de navegación hacia la vista de organización de la colección activa.

---

### User Story 2 - Carrusel Inferior de Últimas 5 Colecciones con Rotación Automática (Priority: P1)

Como usuario con múltiples colecciones, deseo ver en la parte inferior una franja horizontal con las últimas 5 colecciones ordenadas cronológicamente de la más reciente a la más antigua (de izquierda a derecha), con avance automático cada 6 segundos y opción de selección manual inmediata al pulsar sobre cualquiera de ellas, para explorar rápidamente mis series favoritas.

**Why this priority**: Proporciona dinamismo visual y navegación rápida entre las colecciones más recientes sin requerir abrir menús secundarios.

**Independent Test**: Con 3 o más colecciones en la base de datos, observar que la vista cambie de colección cada 6 segundos automáticamente, y al hacer clic sobre una tarjeta de la franja inferior se cambie de inmediato la vista Hero a la colección pulsada, reiniciando el temporizador.

**Acceptance Scenarios**:
1. **Given** que hay 2 o más colecciones en el sistema, **When** transcurren 6 segundos sin interacción del usuario, **Then** la vista realiza una transición cinemática de desplazamiento lateral con fundido cruzado (*Slide & Cross-fade*), desplazando los textos y el personaje lateral mientras el fondo panorámico se disuelve suavemente hacia la siguiente colección del carrusel.
2. **Given** la franja inferior con hasta 5 colecciones ordenadas de izquierda a derecha (más reciente a más antigua), **When** el usuario hace clic sobre una tarjeta de colección distinta a la activa, **Then** la vista Hero salta inmediatamente a la colección seleccionada y el ciclo de 6 segundos se reinicia.
3. **Given** que existe exactamente 1 colección en el sistema, **When** se visualiza la pantalla de inicio, **Then** se muestra la colección en la vista Hero y en la franja inferior, pero el temporizador de rotación de 6 segundos permanece inactivo (no cicla innecesariamente).
4. **Given** que el carrusel está en rotación automática, **When** el usuario sitúa el puntero del ratón sobre la vista Hero o sobre las tarjetas del carrusel, **Then** el temporizador de 6 segundos se pausa inmediatamente (*Hover Pause*) y se reanuda al retirar el cursor.

---

### User Story 3 - Estado Vacío ("Crea tu primera colección") (Priority: P1)

Como nuevo usuario que abre la aplicación por primera vez sin ninguna colección registrada, deseo ver una pantalla en negro elegante con un mensaje claro que indique "Crea tu primera colección", junto con una acción accesible para registrarla, evitando pantallas rotas o elementos vacíos desordenados.

**Why this priority**: Evita errores de renderizado ante bases de datos vacías y guía al usuario de inmediato hacia el primer paso del flujo de trabajo.

**Independent Test**: Ejecutar la aplicación con una base de datos vacía o recién creada; verificar que la interfaz muestre el fondo oscuro/negro, el mensaje de bienvenida y el mecanismo directo para crear la primera colección.

**Acceptance Scenarios**:
1. **Given** que la base de datos no contiene ninguna colección, **When** la aplicación se inicia o se accede a la pantalla de inicio, **Then** se muestra una pantalla con fondo negro y el texto centrado "Crea tu primera colección", con acceso visible para abrir el modal de creación.
2. **Given** la pantalla de estado vacío, **When** el usuario pulsa en la acción de creación y completa el formulario, **Then** la nueva colección se guarda y la interfaz cambia inmediatamente a la vista Home Hero Showcase mostrando la colección recién creada.

---

### User Story 4 - Creación y Edición Extendida de Colección con Imágenes Hero (Priority: P2)

Como organizador de cómics, deseo que el formulario de creación y edición de colecciones permita seleccionar opcionalmente una imagen de fondo panorámica, una imagen de personaje/arte lateral y una sinopsis/descripción, además del nombre, rutas e icono existentes, para personalizar completamente la apariencia Hero de cada saga.

**Why this priority**: Permite alimentar con datos y gráficos reales la experiencia cinematográfica de la pantalla principal.

**Independent Test**: Abrir el diálogo de creación de colección, ingresar nombre, seleccionar una imagen de fondo panorámica, una imagen de personaje lateral recortada, una descripción de texto y una carpeta de cómics; verificar que se guarden correctamente en SQLite y se visualicen en la vista Hero.

**Acceptance Scenarios**:
1. **Given** el formulario de creación de colección, **When** el usuario proporciona el nombre y opcionalmente selecciona una imagen de fondo, una imagen lateral y escribe una descripción, **Then** el sistema almacena estos campos en la base de datos junto con las rutas asociadas.
2. **Given** una colección creada sin imagen de fondo o sin imagen lateral, **When** se presenta en la vista Hero, **Then** el sistema aplica el *fallback* temático inteligente: si falta fondo panorámico, utiliza degradado cinematográfico oscuro o la portada del cómic más reciente difuminada; si falta imagen lateral de personaje, presenta la portada del primer cómic con marco flotante en perspectiva o balancea el bloque tipográfico central sin vacíos.

---

### User Story 5 - Barra Superior de Navegación Unificada con Menús Desplegables (Priority: P2)

Como usuario de la aplicación, deseo que todas las funciones principales del sistema estén accesibles desde una barra de menú superior fija con diseño integrado al tema oscuro cinematográfico, organizada en menús temáticos (`INICIO`, `COLECCIONES ▾`, `CÓMICS ▾`, `SERVIDOR / QR ▾`), para tener acceso universal a cualquier sección sin estorbar el contenido Hero.

**Why this priority**: Centraliza la navegación global de la aplicación sustituyendo la barra lateral anterior y respetando el diseño de la maqueta de referencia.

**Independent Test**: Verificar que en la parte superior se renderice la barra con el logotipo a la izquierda y los menús desplegables (`INICIO`, `COLECCIONES ▾` con Ver todas/Nueva/Re-escanear, `CÓMICS ▾` con Biblioteca/Últimos leídos, `SERVIDOR / QR ▾` con Estado/Ver QR/Clientes conectados), permitiendo desplegar y navegar a cada sección respectiva.

**Acceptance Scenarios**:
1. **Given** la barra superior fija, **When** el usuario hace clic en las distintas opciones de navegación o despliega sus submenús, **Then** la aplicación transiciona a la vista correspondiente manteniendo la coherencia visual.
2. **Given** que el usuario está en la vista Home, **When** observa la barra superior, **Then** la opción "INICIO" muestra un indicador visual activo (línea blanca inferior o resaltado sutil).

---

### Edge Cases

- **Colección con imágenes de gran resolución**: Si el usuario selecciona imágenes de fondo o lateral de 4K o superiores, el sistema debe redimensionarlas o cargarlas en memoria sin congelar el hilo de la interfaz de usuario ni provocar desbordamientos de memoria.
- **Formato de imagen lateral sin canal alfa (PNG transparente)**: Si la imagen lateral seleccionada no tiene fondo transparente, debe renderizarse con un encuadre estético o degradado suave que no desentone con el fondo panorámico.
- **Nombres de colección extremadamente largos**: Si el título de la colección supera los 30 caracteres, la tipografía del título debe ajustar su tamaño o truncarse con elipsis para no solaparse con el arte lateral ni desbordar la pantalla.
- **Eliminación de la colección activa**: Si el usuario elimina la colección que actualmente se encuentra visible en la pantalla Hero, el carrusel debe actualizarse de inmediato mostrando la siguiente colección más reciente, o pasar al estado vacío si no restan colecciones.
- **Redimensionamiento de ventana**: Aunque la aplicación inicia maximizada, si el usuario des-maximiza la ventana o cambia su resolución, la composición visual Hero (capas, tamaños de fuente y carrusel inferior) debe reajustarse responsivamente mediante contenedores elásticos.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El sistema DEBE iniciar la ventana principal de la aplicación en modo maximizado adaptado a la resolución del monitor primario activo.
- **FR-002**: El sistema DEBE proveer una barra de navegación superior fija que incluya el logotipo de la aplicación y menús temáticos con desplegables:
  - `INICIO`: Retorno a la pantalla principal Hero Showcase.
  - `COLECCIONES ▾`: Submenús para "Ver todas las colecciones", "Nueva colección" y "Re-escanear bibliotecas".
  - `CÓMICS ▾`: Submenús para "Explorador general / Biblioteca" y "Últimos cómics leídos".
  - `SERVIDOR / QR ▾`: Submenús para "Estado del servidor", "Mostrar código QR de conexión" y "Dispositivos conectados".
- **FR-003**: En la vista de Inicio, el sistema DEBE presentar un componente Hero central compuesto por:
  - Imagen de fondo panorámica de la colección (`background_image`).
  - Imagen de personaje o arte lateral derecho (`hero_image`).
  - Etiqueta o tarjeta badge superior con el nombre distintivo.
  - Título principal en mayúsculas con efecto visual de relieve y contraste (blanco con sombra de color acento).
  - Bloque de descripción o sinopsis de la colección en panel semitransparente.
  - Botón principal de acción ("READ NOW" / "READ MORE") con corte angular o estilo cómic.
- **FR-004**: Al pulsar el botón "READ NOW", el sistema DEBE disparar la acción de navegación hacia la vista de organización de la colección activa.
- **FR-005**: El sistema DEBE mostrar en la zona inferior de la pantalla una franja horizontal con las últimas 5 colecciones ordenadas de izquierda a derecha desde la más reciente a la más antigua.
- **FR-006**: Cada tarjeta de la franja inferior DEBE contener la miniatura o arte representativo y el nombre de la colección en mayúsculas, destacando visualmente la colección actualmente seleccionada.
- **FR-007**: Cuando existan 2 o más colecciones, el sistema DEBE rotar automáticamente la colección activa cada 6 segundos hacia la siguiente colección disponible en el carrusel, pausando el temporizador mientras el ratón permanezca sobre el área Hero o las tarjetas (*Hover Pause*).
- **FR-008**: Al hacer clic sobre cualquier tarjeta de la franja inferior de colecciones, el sistema DEBE cambiar de inmediato la colección activa de la vista Hero y reiniciar el contador de 6 segundos.
- **FR-009**: Si no existe ninguna colección en la base de datos, el sistema DEBE mostrar una pantalla en negro con el texto "Crea tu primera colección" y un botón para abrir el modal de creación.
- **FR-010**: El formulario de creación y edición de colecciones DEBE incorporar campos opcionales para:
  - Selección de imagen de fondo panorámica (`background_image`).
  - Selección de imagen lateral de personaje/arte (`hero_image`).
  - Texto de descripción o sinopsis (`description`).
- **FR-011**: El almacenamiento de imágenes de fondo y laterales DEBE realizarse copiando los archivos a `app/data/media/collections/` (optimizados/comprimidos según corresponda) y almacenando sus rutas relativas en SQLite, evitando inflar la base de datos con BLOBs de alta resolución.
- **FR-012**: El diseño visual general DEBE replicar con alta fidelidad la estética de la imagen de referencia: paleta oscura cinematográfica, contrastes en rojo vibrante (#E50914 o rojo cómic), tipografías condensadas impactantes y superficies con glassmorphism.
- **FR-013**: Cuando una colección carezca de imagen de fondo o de arte lateral, el sistema DEBE aplicar un *fallback* temático inteligente: si no hay fondo, utilizará un degradado cinematográfico oscuro o la portada del tomo más reciente difuminada; si no hay imagen lateral, utilizará la portada del primer cómic con marco flotante o adaptará el bloque tipográfico sin generar huecos antiestéticos.
- **FR-014**: La transición visual entre colecciones (tanto por el ciclo de 6 segundos como por clic en el carrusel) DEBE implementarse con una animación cinemática de desplazamiento lateral y fundido cruzado (*Slide & Cross-fade*), moviendo los elementos de primer plano y disolviendo el fondo de forma fluida.

---

### Key Entities *(include if feature involves data)*

- **Collection (Colección)**:
  - `id`: Identificador único (entero de 64 bits).
  - `name`: Nombre o título de la colección (cadena de texto obligatoria).
  - `description`: Sinopsis o descripción de la serie/saga (cadena de texto opcional).
  - `background_image_path`: Ruta relativa a la imagen de fondo panorámica en `app/data/media/collections/` (opcional).
  - `hero_image_path`: Ruta relativa a la imagen de arte lateral/personaje en `app/data/media/collections/` (opcional).
  - `icon_data`: Buffer binario de la miniatura/icono cuadrado existente (opcional).
  - `created_at`: Marca temporal de creación en formato ISO 8601 (utilizado para el ordenamiento cronológico de las 5 más recientes).
  - `paths`: Lista de rutas del sistema de archivos vinculadas para escaneo de cómics.

- **HeroCarouselState (Estado del Carrusel Hero)**:
  - `active_index`: Índice de la colección actualmente desplegada en la vista Hero.
  - `recent_collections`: Lista de hasta 5 colecciones ordenadas de más reciente a más antigua.
  - `autoplay_active`: Booleano que indica si la rotación de 6 segundos está habilitada (verdadero cuando hay >= 2 colecciones).
  - `elapsed_seconds`: Contador o disparador de sincronización de 6 segundos.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: El 100% de los lanzamientos de la aplicación abren la ventana maximizada ocupando el tamaño completo de la pantalla sin requerir intervención manual del usuario.
- **SC-002**: Con 2 o más colecciones existentes, la transición automática ocurre con una periodicidad exacta de 6 segundos sin parpadeos ni congelamiento de la interfaz.
- **SC-003**: La selección manual de cualquiera de las últimas 5 colecciones en la barra inferior actualiza la vista Hero en menos de 100 milisegundos.
- **SC-004**: En un entorno con cero colecciones, la pantalla negra de bienvenida con el mensaje "Crea tu primera colección" se renderiza instantáneamente y permite iniciar el formulario con 1 solo clic.
- **SC-005**: La interfaz replica la composición, jerarquía visual y paleta cromática de la imagen de referencia, manteniendo el 100% del código y documentación en español.
- **SC-006**: La animación de transición *Slide & Cross-fade* se ejecuta de forma suave a 60 cuadros por segundo sin congelar el hilo de renderizado ni causar desgarros gráficos.

---

## Assumptions

- Se asume que el backend de Iced v0.13 con las características de `image`, `canvas` y `tokio` soporta la composición por capas mediante `stack!`, `container` translúcidos y temporizadores asíncronos con `iced::time::every`.
- Se asume que las imágenes de fondo y lateral seleccionadas por el usuario se guardan en la carpeta interna `app/data/media/collections/` y sus rutas relativas en SQLite, preservando la ligereza y velocidad de la base de datos sin riesgo de enlaces rotos por cambios externos.
- Se asume que las opciones existentes en la interfaz previa (servidor local, código QR, lista de cómics, escaneo de carpetas) se trasladan organizadamente a la nueva barra superior sin perder ninguna de sus funcionalidades originales.
- Se asume que la navegación completa a la vista detallada de organización de cómics mediante el botón "READ NOW" se integrará de forma modular con el enrutador de vistas de la aplicación.
