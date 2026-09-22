# Feature Specification: Adopción de Tauri y Nueva Interfaz Gráfica Comic Showcase

**Feature Branch**: `003-tauri-hero-interface`

**Created**: 2026-09-16

**Status**: Draft

**Input**: User description: "la UI no quedó como esperaba, vamos a hacer un cambio, usaremos TAURI, la interfaz visual debe quedar como en el ejemplo de "docs\\example" (recordando que en principio no hay coleccion por lo tanto NO tiene imagenes inicialmente), la pagina y sus opciones en el navbar y conserva todas sus características actuales, solo cambia el título, por ahora esta sería la imagen de home. Al añadir la colección se debe agregar el nombre de protagonista (batman, spider-man, etc) eso se vera en la parte roja de la interfaz (que actualmente en el ejemplo dice "SPIDER-MAN")."

## Clarifications

### Session 2026-09-16
- Q: Comportamiento del Dock Inferior con Múltiples Colecciones → A: Dock horizontal con scroll continuo interactivo (rueda del ratón / arrastre táctil) que permite navegar por todas las colecciones de la biblioteca sin truncamiento.
- Q: Acción del Botón Principal ("READ MORE" / "LEER MÁS") en el Hero Showcase → A: Abrir el modal interactivo estilo Dossier clasificado con la ficha detallada de la colección, estadísticas de tomos y acceso directo a la lectura de cómics.
- Q: Efectos Visuales Dinámicos y Efectos de Sonido Estilo Cómic → A: Efectos visuales y animaciones activas por defecto, incorporando un conmutador de audio (Silencio / Activado) en el Navbar para efectos de sonido de cómic opcionales.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Experiencia Visual Home Hero Showcase estilo Cómic Dinámico (Priority: P1)

Como lector y coleccionista de cómics, quiero que la pantalla principal de bienvenida presente un diseño cinematográfico inmersivo de estilo cómic (idéntico a la maqueta de referencia `docs/example`), con el arte del personaje, fondo temático, título estilizado del protagonista en relieve rojo y sinopsis, para disfrutar de una presentación moderna y atractiva de mi biblioteca.

**Why this priority**: Es el núcleo visual del cambio solicitado por el usuario y define la primera impresión y estética general de la aplicación de escritorio mediante la nueva arquitectura de interfaz webview de alto rendimiento con Tauri.

**Independent Test**: Puede probarse de manera independiente abriendo la aplicación con colecciones existentes y verificando que el escenario central muestre el arte del personaje activo, el título del protagonista en el cartel rojo, la sinopsis en la tarjeta translúcida y el fondo visual correspondiente.

**Acceptance Scenarios**:

1. **Given** una colección seleccionada con nombre de protagonista "SPIDER-MAN", imagen de héroe y sinopsis, **When** el usuario ingresa a la vista Home, **Then** la interfaz renderiza el título "SPIDER-MAN" sobre el fondo rojo con relieve tridimensional, el texto de la sinopsis en el contenedor translúcido, el arte recortado del personaje en el escenario derecho con efectos de aura cómic, y el fondo ambiental.
2. **Given** una colección que no tiene imágenes asignadas, **When** se visualiza en el Home, **Then** el sistema presenta un marco y avatar estilizado por defecto sin mostrar iconos de imagen rota ni errores gráficos.
3. **Given** que no existe ninguna colección en la base de datos (arranque limpio), **When** se inicia la aplicación, **Then** la vista Home muestra un estado de bienvenida inicial limpio y elegante que invita a crear la primera colección, sin errores ni referencias a imágenes inexistentes.

---

### User Story 2 - Dock Inferior de Selección Rápida y Transición de Colecciones (Priority: P2)

Como usuario con múltiples colecciones, quiero una barra inferior flotante (Dock) que exhiba tarjetas con miniaturas y nombres de mis colecciones más recientes, permitiendo alternar instantáneamente entre ellas con animaciones fluidas y retroalimentación visual al hacer clic.

**Why this priority**: Facilita la navegación rápida entre universos/colecciones directamente desde la pantalla principal sin necesidad de abrir menús secundarios.

**Independent Test**: Puede probarse creando al menos 2 colecciones, verificando que ambas aparezcan en el Dock inferior y que al hacer clic sobre una tarjeta inactiva, la interfaz cambie suavemente el protagonista, la sinopsis, el fondo y la iluminación activa en el Dock.

**Acceptance Scenarios**:

1. **Given** varias colecciones registradas, **When** el usuario hace clic en una tarjeta del Dock inferior, **Then** dicha tarjeta se marca con el resplandor activo y el escenario principal actualiza inmediatamente el protagonista, arte y sinopsis de esa colección.
2. **Given** el cursor del usuario posándose sobre una tarjeta del Dock, **When** se produce el hover, **Then** la tarjeta se eleva ligeramente y muestra un efecto de resalte interactivo.

---

### User Story 3 - Gestión de Colecciones con Campo de Protagonista / Héroe (Priority: P3)

Como usuario organizador de mi biblioteca, quiero que al crear o editar una colección pueda especificar el nombre del protagonista principal (ej. "BATMAN", "SPIDER-MAN", "LARA CROFT") junto con su nombre general, sinopsis, imagen panorámica y arte de personaje, para que se refleje fielmente en la tipografía y bloque rojo del Hero Showcase.

**Why this priority**: Proporciona los datos requeridos por la nueva interfaz visual para personalizar el título de impacto en cada colección.

**Independent Test**: Puede probarse abriendo el modal de creación de colección, completando el nombre de la colección ("Batman: Detective Comics"), el protagonista ("BATMAN") y la sinopsis, guardando la colección y verificando que el bloque rojo del Home muestre "BATMAN".

**Acceptance Scenarios**:

1. **Given** el formulario de creación/edición de colección abierto, **When** el usuario ingresa el nombre de la colección y el nombre del protagonista y presiona "Guardar", **Then** los datos se persisten en la base de datos local y el Hero Showcase refleja el nuevo protagonista.
2. **Given** una colección existente sin protagonista configurado previamente, **When** se carga en el Home, **Then** el sistema utiliza como fallback el nombre general de la colección en mayúsculas para el bloque rojo.

---

### User Story 4 - Barra Superior Unificada con Funcionalidades del Ecosistema Comic (Priority: P4)

Como usuario, quiero una barra de navegación superior fija (Navbar) que mantenga el estilo visual de `docs/example` e integre todas las herramientas actuales del sistema: acceso al Home, listado y creación de Colecciones, re-escaneo de Cómics, y control del Servidor Local / QR de dispositivos remotos.

**Why this priority**: Garantiza la paridad total de funcionalidades con la versión previa, asegurando que ninguna capacidad de red, catalogación o escaneo se pierda con la nueva interfaz gráfica.

**Independent Test**: Puede probarse navegando por cada menú desplegable del Navbar (`COLECCIONES`, `CÓMICS`, `SERVIDOR / QR`) y verificando que ejecuten las acciones de escaneo, conexión de servidor, apertura de QR y selección de colecciones.

**Acceptance Scenarios**:

1. **Given** el Navbar superior, **When** el usuario despliega el menú `COLECCIONES`, **Then** visualiza las colecciones existentes con acceso rápido y la opción de "Nueva Colección".
2. **Given** el Navbar superior, **When** el usuario interactúa con `SERVIDOR / QR`, **Then** puede encender/apagar el servidor web local, abrir el modal con el código QR para lectura remota y gestionar dispositivos de confianza.
3. **Given** el Navbar superior, **When** el usuario hace clic en `HOME`, **Then** regresa a la vista principal Hero Showcase.

---

### User Story 5 - Dossier Detallado y Acceso a Lectura de Cómics (Priority: P5)

Como lector, quiero presionar el botón de acción principal ("LEER MÁS" / "READ MORE" o "LEER AHORA") en el Hero Showcase para abrir un modal de Dossier detallado con información de la colección, lista de tomos disponibles y acceso directo al visor de lectura.

**Why this priority**: Conecta la experiencia de presentación estética con el consumo real de los cómics y la exploración de tomos de la colección.

**Independent Test**: Puede probarse haciendo clic en el botón principal de llamada a la acción (CTA) de la colección activa y comprobando la apertura del modal con la información extendida y los accesos a los cómics asociados.

**Acceptance Scenarios**:

1. **Given** una colección seleccionada en el Home, **When** el usuario hace clic en el botón "READ MORE" / "LEER MÁS", **Then** se abre una ventana modal de Dossier con la ficha completa de la colección, estadísticas de tomos y opciones de lectura.
2. **Given** el modal de Dossier abierto, **When** el usuario presiona el botón de cierre (✕) o hace clic fuera del diálogo, **Then** el modal se cierra fluidamente retornando al Home.

---

### Edge Cases

- **Biblioteca totalmente vacía**: Al iniciar la aplicación por primera vez en una base de datos nueva, no hay colecciones ni cómics. El Home debe mostrar un estado inicial decorativo neutral que indique "No hay colecciones configuradas" con un botón central para crear la primera colección, sin elementos rotos.
- **Colección sin imágenes de fondo ni de personaje**: Si el usuario crea una colección sin adjuntar imagen panorámica ni personaje transparente, el sistema debe utilizar una paleta de fondo basada en degradados CSS temáticos y un icono/silueta vectorial de reemplazo.
- **Nombre de protagonista extremadamente largo**: Si el usuario introduce un protagonista de más de 30 caracteres, la tipografía del cartel rojo debe escalar o truncar limpiamente sin desbordar los límites del contenedor ni romper la alineación con la caja de sinopsis.
- **Redimensión de ventana o pantallas ultra-anchas**: La cuadrícula principal y el Dock inferior deben adaptarse elásticamente mediante CSS Grid / Flexbox garantizando que el personaje lateral y el cartel permanezcan legibles tanto en resoluciones 1080p como en monitores 4K o ventanas compactas.
- **Pérdida de conexión o reinicio de servidor local**: Los indicadores de estado en el Navbar deben reflejar fielmente si el servidor Axum local está activo, inactivo o reportando errores.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El sistema DEBE implementar la interfaz gráfica de usuario de escritorio mediante **Tauri**, integrando la capa webview con el backend nativo en Rust.
- **FR-002**: La pantalla de inicio DEBE replicar la estructura visual, jerarquía y animaciones del prototipo `docs/example`, incluyendo:
  - Fondo ambiental con viñeteado oscuro y trama tipo cómic (*halftone*).
  - Escenario lateral con el arte del personaje de la colección activa.
  - Bloque superior de título del protagonista con relieve tridimensional y fondo rojo sesgado.
  - Caja de sinopsis translúcida (*Glassmorphism*) con tipografía legible.
  - Botón de llamada a la acción (*CTA*) con diseño diagonal dinámico.
  - Dock inferior interactivo con tarjetas de las colecciones registradas.
- **FR-003**: El modelo de datos de Colección DEBE incorporar el campo `protagonist` (nombre del protagonista / héroe, ej. "SPIDER-MAN", "BATMAN"), persistido en SQLite y editable desde el formulario de colección.
- **FR-004**: Si una colección no tiene definido el campo `protagonist`, el sistema DEBE utilizar el valor de `name` de la colección como texto por defecto en la caja roja.
- **FR-005**: Cuando la base de datos no contenga ninguna colección, el sistema DEBE mostrar un estado inicial de bienvenida sin requerir archivos de imagen previos.
- **FR-006**: La barra de navegación superior (Navbar) DEBE conservar todas las capacidades funcionales preexistentes e incorporar controles interactivos de la nueva UI:
  - Navegación al Home.
  - Menú desplegable de Colecciones (listado, selección y disparador de creación/edición).
  - Menú desplegable de Cómics (re-escaneo de carpetas y listado).
  - Menú desplegable de Servidor / Red (estado de ejecución, encendido/apagado, código QR y gestión de dispositivos de confianza).
  - Conmutador de efectos de audio de cómic (Mute / Unmute).
- **FR-007**: El sistema DEBE permitir conmutar entre colecciones haciendo clic en las tarjetas del Dock inferior, ofreciendo desplazamiento horizontal suave (*smooth scroll*) con soporte para rueda del ratón y arrastre interactivo cuando existan múltiples colecciones, actualizando el protagonista, fondo, arte y sinopsis en tiempo real.
- **FR-008**: El sistema DEBE proporcionar una ventana modal tipo Dossier al presionar el botón de llamada a la acción, permitiendo consultar los metadatos completos y acceder a los cómics de la colección.
- **FR-009**: Toda la comunicación entre la interfaz de usuario en Tauri y los servicios del backend en Rust DEBE realizarse mediante comandos invocables (*Tauri commands*) y eventos asíncronos sin bloquear el hilo de ejecución principal.
- **FR-010**: El sistema DEBE preservar el funcionamiento offline-first sin requerir conexión a internet ni CDNs externos, incluyendo fuentes y recursos gráficos empaquetados localmente.

### Key Entities

- **Colección (`Collection`)**: Agrupación lógica de cómics que representa una serie o universo.
  - `id`: Identificador numérico único.
  - `name`: Nombre descriptivo de la colección (ej. "The Amazing Spider-Man").
  - `protagonist`: Nombre del personaje / protagonista que se luce en el bloque rojo (ej. "SPIDER-MAN").
  - `description`: Sinopsis o resumen narrativo que se presenta en la caja de bio.
  - `background_image_path`: Ruta local de la imagen de fondo panorámica.
  - `hero_image_path`: Ruta local del arte del personaje recortado con transparencia.
  - `icon_data`: Imagen miniatura binaria (1:1) para el selector y Dock.
  - `created_at`: Fecha y hora de registro.

- **Dispositivo de Confianza (`TrustedDevice`)**: Dispositivo móvil o remoto autorizado para lectura en red local.
  - `id`: Identificador único.
  - `device_name`: Nombre descriptivo del cliente.
  - `token`: Token criptográfico de acceso.
  - `created_at`: Fecha de vinculación.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: El tiempo de arranque y renderizado del primer frame interactivo de la interfaz Tauri debe completarse en menos de 1.0 segundo en entornos de escritorio estándar.
- **SC-002**: La conmutación entre colecciones desde el Dock inferior debe responder en menos de 50 milisegundos sin parpadeos perceptibles.
- **SC-003**: El 100% de las funcionalidades del Navbar anterior (escaneo de biblioteca, creación de colecciones, servidor HTTP y códigos QR) deben ser accesibles y operativas en la nueva barra superior.
- **SC-004**: En un entorno recién instalado sin cómics ni colecciones previas, la aplicación debe iniciar con 0 errores de consola y mostrar el estado inicial de bienvenida sin imágenes rotas.
- **SC-005**: El consumo de recursos en reposo (pantalla de inicio estática) no debe superar el 1% de uso de CPU.

---

## Assumptions

- **A-001**: La interfaz se construirá sobre Tauri (v2) aprovechando el motor de renderizado nativo del sistema operativo (Microsoft Edge WebView2 en Windows).
- **A-002**: Los recursos tipográficos requeridos (Bebas Neue, Montserrat, Inter) se empaquetarán como fuentes locales en el directorio de assets para cumplir estrictamente con el principio offline-first.
- **A-003**: La lógica de negocio existente en Rust (servicios de catálogo, lector de archivos CBZ/CBR, servidor REST Axum, SQLite con SQLx) se mantiene y se reutiliza como núcleo del backend de la aplicación.
- **A-004**: Los assets visuales base provistos en `docs/example` (logo, texturas de fondo y efectos cómic) servirán de plantilla predeterminada para el estilo global de la aplicación.
