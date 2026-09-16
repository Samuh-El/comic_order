# Instrucciones Canónicas de Frontend

Este documento define la arquitectura visual, tokens de diseño, estándares de componentes y directrices de implementación para las interfaces de usuario del proyecto **Comic**: la aplicación de escritorio nativa (**Iced GUI**) y el lector web remoto (**SPA Embebida**).

---

## 1. Arquitectura de Frontend

El proyecto posee dos canales de presentación visual unificados bajo una misma identidad gráfica:

1. **Frontend Nativo de Escritorio (Iced v0.13)**:
   - Basado en **The Elm Architecture (TEA)**: Modelo central (`ComicApp`), Actualización pura (`update`), Renderizado funcional declarativo (`view`) y Suscripciones asíncronas (`subscription`).
   - Ubicación de componentes: `app/src/ui/`.
2. **Lector Web Remoto (SPA Embebida en `app/src/server.rs`)**:
   - Desarrollado en **HTML5 / CSS3 / JavaScript Vanilla (ES6+)** autocontenido en una constante de cadena en el servidor.
   - Diseñado para visualización *mobile-first* en teléfonos y tabletas conectadas por red local, sin dependencias externas ni CDNs.

---

## 2. Sistema de Diseño y Tokens Visuales

Ambas interfaces DEBEN mantener coherencia estricta respetando la siguiente paleta de colores y tipografía:

### Paleta de Colores Canónica:

| Token | Código HEX | Valor RGB / RGBA | Uso específico |
| :--- | :--- | :--- | :--- |
| **`--bg`** (Fondo principal) | `#0a0a0c` | `rgb(0.08, 0.08, 0.10)` | Fondo general de la ventana y el canvas. |
| **`--surface`** (Superficie/Paneles) | `#16161a` | `rgb(0.12, 0.12, 0.16)` | Contenedores, tarjetas, cabecera y sidebar. |
| **`--surface-hover`** | `#25252d` | `rgb(0.20, 0.20, 0.28)` | Estado hover de tarjetas de colección y botones. |
| **`--primary`** (Acento / Marca) | `#ff3e5e` | `rgb(0.91, 0.27, 0.37)` | Botones de acción principal, bordes activos, títulos destacados. |
| **`--primary-hover`** | `#ff5e7a` | `rgb(1.0, 0.37, 0.48)` | Hover sobre elementos primarios. |
| **`--success`** (Servidor activo) | `#218c54` | `rgb(0.13, 0.55, 0.33)` | Indicador de estado del servidor en ejecución. |
| **`--text`** (Texto principal) | `#ffffff` | `rgb(1.0, 1.0, 1.0)` | Títulos, nombres de colecciones y lectura principal. |
| **`--text-dim`** (Texto secundario) | `#a1a1aa` | `rgb(0.70, 0.70, 0.70)` | Metadatos (año, saga, número), conteo de cómics. |
| **`--overlay-bg`** | `#000000b3`| `rgba(0.0, 0.0, 0.0, 0.70)` | Fondo oscurecido de modales y visor a pantalla completa. |

### Tipografía:
- **Familia tipográfica**: Segoe UI, -apple-system, BlinkMacSystemFont, 'Inter', Roboto, sans-serif.
- **Variantes de peso**: Regular (texto corriente), Semibold (títulos de sección), Bold (encabezados de modales).

### Radios de Borde (Border Radius):
- Pequeño: `4.0px` - `6.0px` (botones secundarios, menús contextuales).
- Estándar: `8.0px` - `10.0px` (tarjetas de cómic, botones principales, campos de texto).
- Amplio: `12.0px` - `16.0px` (contenedores modales flotantes).

---

## 3. Guía de Componentes de Escritorio (Iced)

### A. Barra Lateral (`app/src/ui/sidebar.rs`)
- Contenedor con ancho fijo (`200px`) y scroll vertical independiente.
- Lista de colecciones con iconos representativos.
- Menú contextual desplegable mediante clic derecho (`mouse_area(btn).on_right_press(...)`) con opciones para **Editar** (nombre e icono) y **Eliminar**.
- Botón dinámico de **Servidor Activo / Compartir QR** con cambio de color semántico (rojo cuando está inactivo, verde cuando está activo).
- Botón de **Dispositivos Recurrentes** para acceder a la gestión de accesos permanentes.

### B. Cuadrícula de Cómics (`app/src/ui/comic_grid.rs`)
- Renderizado de fondo mediante Canvas personalizado (`MeshGradient`) con gradientes lineales superpuestos para un aspecto moderno y elegante.
- Barra superior con nombre de la colección, total de cómics y botón **"Añadir Carpeta"**.
- Cuadrícula responsiva (5 tarjetas por fila en escritorio estándar).
- Cada tarjeta contiene:
  - Portada de 160x240 px con radio de 8px (o contenedor de reemplazo si no hay imagen).
  - Título recortado con elipsis a 20 caracteres para evitar desbordamientos.
  - Metadatos formateados (Año, Número de entrega o tipo de archivo).
  - Botón de edición de metadatos accesible debajo de la portada.

### C. Visor de Cómics (`app/src/ui/reader.rs`)
- Modo inmersivo con fondo negro puro (`#000000`).
- Canvas interactivo (`PageViewer`):
  - Renderiza la página aplicando una matriz de transformación afín: `translate(center + pan)` y `scale(zoom)`.
  - Zoom dinámico mediante rueda del ratón entre `0.1x` y `5.0x`.
  - Paneo continuo arrastrando el cursor del ratón (`mouse_area` con interacción `Grabbing`).
- Barra de control superior e inferior ocultable/mostrable mediante un solo clic en la página.
- Navegación bidireccional por botones en pantalla y teclas de flecha (`←` / `→`), con escape (`Esc`) para salir.

### D. Modales de Edición y Superposiciones
- Los modales (`metadata_editor.rs`, `collection_editor.rs`, `trusted_devices.rs`, `qr_overlay`) se implementan como capas superpuestas con el macro `stack!`.
- Anchura fija centrada (450px) con fondo oscuro contrastado (`#10101c`) y borde primario sutil.
- Cierre mediante botón de cruz (SVG) o tecla de escape.

---

## 4. Guía del Lector Web Remoto (SPA Embebida)

1. **Autonomía Total**:
   - Todo el código HTML, CSS y JS reside en `WEB_PAGE` dentro de `app/src/server.rs`.
   - PROHIBIDO incluir enlaces a hojas de estilo remotas, fuentes de Google Fonts o librerías de terceros (React, Tailwind, Axios).
2. **Experiencia Táctil (*Mobile Gestures*)**:
   - Escucha los eventos `touchstart` y `touchend` en el visor para detectar gestos de deslizamiento horizontal (*swipe*):
     - Deslizamiento hacia la derecha (> 60px): página anterior.
     - Deslizamiento hacia la izquierda (< -60px): página siguiente.
3. **Persistencia de Sesión Local**:
   - Al cargar la página con el parámetro `?token=...`, se almacena inmediatamente en el `localStorage` del dispositivo bajo la clave `comic_token`.
   - Toda petición asíncrona (`apiFetch`) inyecta la cabecera `Authorization: Bearer <token>`.
4. **Modo de Diagnóstico Rápido**:
   - La SPA incluye un botón flotante `ID` que despliega una consola superpuesta en pantalla (`debugInfo`) mostrando IP, fragmento del token, User-Agent y estado de conectividad para facilitar el soporte al usuario sin necesidad de herramientas de desarrollador en dispositivos móviles.

---

## 5. Reglas de Idioma y Estilo

- Todas las etiquetas de interfaz, mensajes de confirmación, diálogos y descripciones DEBEN estar redactados exclusivamente en español.
- No se permiten alteraciones de color no documentadas o que rompan la armonía de los tokens declarados en este documento.
