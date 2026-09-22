# Research: Arquitectura Tauri v2 y Nueva Interfaz Comic Showcase

**Feature**: `003-tauri-hero-interface`
**Fecha**: 2026-09-16

---

## 1. Decisiones de Arquitectura Frontend y Tauri v2

### Decisión 1: Framework de UI y Gestión del Webview
- **Decisión**: Utilizar **Tauri v2** con una aplicación frontend ligera basada en HTML5 semántico, CSS3 moderno (Custom Properties, CSS Grid, Glassmorphism, animaciones aceleradas por GPU) y JavaScript modular (ES6+), sin sobrecargar el proyecto con frameworks SPA pesados (React/Vue/Angular) ni dependencias de CDN externas.
- **Razón**: 
  1. Cumple al 100% con la prohibición constitucional de CDNs y librerías externas en tiempo de ejecución (*Offline-First*).
  2. Permite portar y adaptar directamente la arquitectura visual de alto impacto ya validada en `docs/example/` (`index.html`, `styles.css`, `script.js`).
  3. Tamaño de binario mínimo, arranque ultra rápido (<1 segundo) y uso insignificante de memoria RAM y CPU en reposo (<1%).
- **Alternativas descartadas**:
  - *Iced TEA*: Descartada a petición expresa del usuario debido a las limitaciones de composición gráfica compleja y flexibilidad de layout requerida para la estética cómic de `docs/example`.
  - *Electron*: Descartada por consumo excesivo de memoria (150MB+ en reposo), tamaño de paquete voluminoso (80MB+) y sobrecarga en sistemas locales.

---

## 2. Comunicación IPC (Inter-Process Communication) en Tauri

### Decisión 2: Patrón Inbound Adapter para Comandos Tauri
- **Decisión**: Implementar un módulo adaptador de entrada `adapters/inbound/tauri_bridge/` que expone funciones anotadas con `#[tauri::command]`. Cada comando interactúa exclusivamente con `ComicFacade`, preservando el aislamiento estricto de la Arquitectura Hexagonal.
- **Comandos Principales**:
  - `get_collections()`: Lista todas las colecciones para el Dock y Navbar.
  - `get_collection(id)`: Obtiene detalles de una colección.
  - `create_collection(payload)`: Crea una nueva colección con nombre, protagonista, sinopsis e imágenes.
  - `update_collection(payload)`: Actualiza colección existente.
  - `delete_collection(id)`: Elimina colección.
  - `get_comics_by_collection(id)`: Obtiene lista de cómics de una colección.
  - `get_comic_cover(comic_id)`: Obtiene miniatura base64 o blob local.
  - `get_comic_page(comic_id, page_index)`: Obtiene imagen procesada de página.
  - `scan_library()`: Dispara escaneo asíncrono de carpetas.
  - `get_server_status()` / `toggle_server()`: Control de servidor Axum local.
  - `get_trusted_devices()` / `add_trusted_device()` / `remove_trusted_device()`: Gestión de dispositivos remotos.
  - `select_file_dialog(filter)`: Abre selector de archivos nativo de Windows para imágenes.
- **Razón**: El frontend webview nunca accede directamente a la base de datos ni al sistema de archivos; toda operación pasa por las reglas de validación del dominio y la fachada.

---

## 3. Entrega de Imágenes y Streaming Local

### Decisión 3: Protocolo Custom / Base64 Data URLs para Portadas y Arte
- **Decisión**: Las imágenes procesadas (portadas de cómic, avatares de personajes, fondos panorámicos) se entregarán mediante un protocolo custom de Tauri (`comic://media/...` o streaming asset protocol) y Data URLs en Base64 para miniaturas inmediatas cacheadas en SQLite.
- **Razón**: Permite renderizado instantáneo en el DOM sin latencia de disco, con soporte nativo para caché y compatibilidad total con `img` y `background-image`.

---

## 4. Tipografías Locales y Recursos Gráficos Offline

### Decisión 4: Empaquetado Local de Fuentes Bebas Neue, Montserrat e Inter
- **Decisión**: Descargar e incluir los archivos de fuentes `.woff2` / `.ttf` en el directorio de assets locales del frontend (`app/ui/assets/fonts/`), referenciándolos con `@font-face` en `styles.css`.
- **Razón**: Garantiza funcionamiento 100% offline sin peticiones a `fonts.googleapis.com` ni `fonts.gstatic.com`.

---

## 5. Modelo de Datos y Campo Protagonista

### Decisión 5: Extensión Idempotente del Esquema SQLite
- **Decisión**: Añadir la columna `protagonist TEXT` en la tabla `collections` de forma idempotente:
  ```sql
  -- Migración idempotente en schema.rs
  ALTER TABLE collections ADD COLUMN protagonist TEXT;
  ```
- **Razón**: Preserva la compatibilidad retrospectiva con bases de datos existentes `comic.db`. Si `protagonist` es `NULL` o vacío, el frontend usará `name` en mayúsculas como valor de sustitución (*fallback*).
