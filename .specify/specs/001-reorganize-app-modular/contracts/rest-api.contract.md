# Contrato de la API REST

**Feature**: `001-reorganize-app-modular`  
**Fecha**: 2026-09-15  
**Estado**: Completado  

Este documento define la especificación de endpoints, autenticación, códigos de respuesta y esquemas JSON del servidor HTTP Axum.

---

## 1. Autenticación y Seguridad

- **Mecanismo**: Validación obligatoria de token para todos los endpoints bajo `/api/`.
- **Formas de envío**:
  1. Cabecera HTTP: `Authorization: Bearer <token>`
  2. Query Parameter: `?token=<token>` o `?t=<token>`
- **Códigos de error de autenticación**:
  - `401 Unauthorized`: Token faltante, no coincidente con la sesión actual ni registrado en `trusted_devices`.

---

## 2. Catálogo de Endpoints

### A. Rutas Públicas

#### `GET /`
- **Descripción**: Sirve la SPA HTML5/CSS3/JS del lector remoto.
- **Respuesta**: `200 OK`, `Content-Type: text/html; charset=utf-8`.

#### `GET /ping`
- **Descripción**: Verificación de conectividad.
- **Respuesta**: `200 OK`, `Content-Type: text/plain`, body: `"pong"`.

---

### B. Rutas de Colecciones

#### `GET /api/collections`
- **Descripción**: Retorna el listado de colecciones activas.
- **Respuesta**: `200 OK`, `Content-Type: application/json`.
```json
[
  {
    "id": 1,
    "name": "DC Comics",
    "has_icon": true
  }
]
```

#### `GET /api/collections/:id/icon`
- **Descripción**: Retorna la imagen del icono de la colección.
- **Respuesta**:
  - `200 OK`, `Content-Type: image/png` o `image/jpeg`, `Cache-Control: public, max-age=3600`.
  - `404 Not Found` si la colección no posee icono.

#### `GET /api/collections/:id/comics`
- **Descripción**: Lista los cómics indexados en una colección dada.
- **Respuesta**: `200 OK`, `Content-Type: application/json`.
```json
[
  {
    "id": 10,
    "title": "Batman #01",
    "file_type": "cbz",
    "year": 1940,
    "issue_number": 1,
    "saga": "Golden Age",
    "page_count": 64,
    "has_cover": true
  }
]
```

---

### C. Rutas de Cómics y Streaming de Páginas

#### `GET /api/comics/:id/cover`
- **Descripción**: Retorna la portada del cómic en miniatura (300x450 JPEG).
- **Respuesta**: `200 OK`, `Content-Type: image/jpeg`, `Cache-Control: public, max-age=3600`.

#### `GET /api/comics/:id/page/:page?max_width=800`
- **Descripción**: Obtiene y transmite la página solicitada (índice 0-based), opcionalmente redimensionada según el ancho solicitado.
- **Query Params**:
  - `max_width`: (Opcional) Ancho máximo deseado para streaming responsivo en clientes móviles.
- **Respuesta**:
  - `200 OK`, `Content-Type: image/jpeg` o `image/png`, `Cache-Control: public, max-age=86400`.
  - `404 Not Found` si el índice de página excede el total o el cómic no existe.

---

### D. Rutas de Progreso de Lectura

#### `GET /api/comics/:id/progress?device_id=xyz`
- **Descripción**: Consulta la última página leída para un dispositivo.
- **Respuesta**: `200 OK`, `Content-Type: application/json`.
```json
{
  "comic_id": 10,
  "current_page": 24,
  "completed": false
}
```

#### `POST /api/comics/:id/progress`
- **Descripción**: Actualiza el progreso de lectura para un dispositivo.
- **Body**:
```json
{
  "device_id": "tablet-samsung-01",
  "current_page": 25
}
```
- **Respuesta**: `200 OK`, `{"status": "ok"}`.
