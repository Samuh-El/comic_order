# Contract: Tauri IPC Commands & Events

**Feature**: `003-tauri-hero-interface`
**Fecha**: 2026-09-16

---

## 1. Comandos Invocables (`#[tauri::command]`)

### Colecciones

#### `get_collections`
- **Invocación**: `invoke('get_collections')`
- **Retorno**: `Promise<CollectionDto[]>`
- **Descripción**: Devuelve la lista completa de colecciones con metadatos y URLs de recursos para el Dock y el Navbar.

#### `save_collection`
- **Invocación**: `invoke('save_collection', { payload: SaveCollectionPayload })`
- **Retorno**: `Promise<{ id: number }>`
- **Errores**: `string` con mensaje descriptivo si falla la validación o persistencia.

#### `delete_collection`
- **Invocación**: `invoke('delete_collection', { id: number })`
- **Retorno**: `Promise<void>`

---

### Cómics y Lectura

#### `get_comics_by_collection`
- **Invocación**: `invoke('get_comics_by_collection', { collectionId: number })`
- **Retorno**: `Promise<ComicSummaryDto[]>`

#### `get_comic_page`
- **Invocación**: `invoke('get_comic_page', { comicId: number, pageIndex: number })`
- **Retorno**: `Promise<{ dataUrl: string, totalPages: number, pageIndex: number }>`

#### `scan_monitored_paths`
- **Invocación**: `invoke('scan_monitored_paths')`
- **Retorno**: `Promise<{ comicsFound: number }>`

---

### Servidor Local y Dispositivos de Confianza

#### `get_server_status`
- **Invocación**: `invoke('get_server_status')`
- **Retorno**: `Promise<{ running: boolean, port: number, localIp: string, qrDataUrl: string }>`

#### `toggle_server`
- **Invocación**: `invoke('toggle_server', { enable: boolean })`
- **Retorno**: `Promise<{ running: boolean }>`

#### `get_trusted_devices`
- **Invocación**: `invoke('get_trusted_devices')`
- **Retorno**: `Promise<TrustedDeviceDto[]>`

#### `add_trusted_device`
- **Invocación**: `invoke('add_trusted_device', { deviceName: string })`
- **Retorno**: `Promise<{ token: string, qrDataUrl: string }>`

#### `remove_trusted_device`
- **Invocación**: `invoke('remove_trusted_device', { id: string })`
- **Retorno**: `Promise<void>`

---

### Diálogos del Sistema

#### `pick_image_file`
- **Invocación**: `invoke('pick_image_file', { title: string })`
- **Retorno**: `Promise<string | null>` (ruta absoluta seleccionada por el usuario).
