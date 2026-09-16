# Instrucciones Canónicas de Frontend (Adaptadores de Entrada)

Este documento define la arquitectura visual, tokens de diseño, estándares de componentes y directrices de implementación para las interfaces de usuario del proyecto **Comic**: la aplicación de escritorio nativa (**Iced GUI**) y el lector web remoto (**SPA Embebida**).

---

## 1. Rol en la Arquitectura Hexagonal

Tanto la interfaz gráfica de escritorio como el lector web remoto operan como **Adaptadores de Entrada Primarios (*Primary / Inbound Adapters*)**:

1. **GUI Nativa de Escritorio (Iced v0.13)**:
   - Se ubica en `app/src/adapters/inbound/desktop_ui/` (o modularizado en `app/src/ui/`).
   - Implementa **The Elm Architecture (TEA)** y consume directamente la Fachada de Aplicación (`ComicFacade`) para cargar colecciones y páginas sin interactuar con I/O directo de disco.
2. **Lector Web Remoto (SPA Embebida)**:
   - Se sirve desde el adaptador de API REST (`app/src/adapters/inbound/rest_api/`) y consume los endpoints HTTP del servidor Axum.
   - 100% autocontenida en HTML5 / CSS3 / JavaScript Vanilla (ES6+), diseñada para visualización *mobile-first* en teléfonos y tabletas conectadas por red local, sin dependencias externas ni CDNs.

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
- **Familia**: Segoe UI, -apple-system, BlinkMacSystemFont, 'Inter', Roboto, sans-serif.
- **Pesos**: Regular (400), Semibold (600), Bold (700).

### Radios de Borde (Border Radius):
- Pequeño: `4.0px` - `6.0px`
- Estándar: `8.0px` - `10.0px`
- Amplio: `12.0px` - `16.0px`

---

## 3. Modos de Lectura (Patrón Estrategia en UI)

Las interfaces deben permitir la selección de diferentes estrategias de renderizado de lectura:
1. **Página Simple**: Modo predeterminado para escritorio y smartphones en vertical.
2. **Doble Página**: Modo optimizado para pantallas panorámicas y tabletas en orientación horizontal.
3. **Lectura Vertical Continua (*Webtoon*)**: Desplazamiento vertical fluido de páginas en cascada.

---

## 4. Guía de Componentes de Escritorio (Iced)

- **Barra Lateral (`sidebar.rs`)**: Navegación con scroll independiente, menú contextual con clic derecho para colecciones, alternador de servidor HTTP y gestión de dispositivos de confianza.
- **Cuadrícula de Cómics (`comic_grid.rs`)**: Fondo decorativo dinámico con `MeshGradient` sobre Canvas, tarjetas con portadas (160x240 px, radio 8px), títulos truncados y botón de edición.
- **Visor de Cómics (`reader.rs`)**: Pantalla inmersiva con Canvas interactivo (`PageViewer`), traslación continua (*pan*) y escalado continuo (*zoom*) entre `0.1x` y `5.0x`.
- **Modales flotantes**: Implementados con `stack!` centrados en pantalla (ancho 450px) para metadatos, edición de colección y dispositivos de confianza.

---

## 5. Guía del Lector Web Remoto (SPA Embebida)

- **Streaming Bajo Demanda**: Consume páginas individuales a través de la API REST sin descargar el cómic completo.
- **Gestos Táctiles**: Soporte nativo para eventos táctiles (`touchstart`, `touchend`) con cambio de página al superar un umbral de 60px de deslizamiento horizontal (*swipe*).
- **Persistencia**: Token de sesión almacenado en `localStorage` (`comic_token`).
- **Autonomía**: Prohibido el uso de CDNs, frameworks externos o librerías de internet.

---

## 6. Idioma

Todas las etiquetas, textos y mensajes de la interfaz de usuario DEBEN estar escritos exclusivamente en español.
