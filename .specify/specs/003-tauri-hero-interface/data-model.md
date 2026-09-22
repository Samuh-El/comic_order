# Data Model: Entidades y Esquema de Datos Comic Showcase

**Feature**: `003-tauri-hero-interface`
**Fecha**: 2026-09-16

---

## 1. Entidades del Dominio

### Entidad: `Collection`
Representa una colección lógica de cómics, universo temático o franquicia.

| Campo | Tipo | Nulo | Descripción | Validaciones / Invariantes |
|---|---|---|---|---|
| `id` | `i64` | No | Clave primaria autonumérica | `>= 0` (`0` para nuevas entidades no persistidas) |
| `name` | `String` | No | Nombre descriptivo de la colección | `1 <= len <= 100`, no vacío ni espacios en blanco |
| `protagonist` | `Option<String>` | Sí | Nombre del personaje principal para el bloque rojo | `len <= 100`. Si `None`, el sistema usa `name.to_uppercase()` |
| `description` | `Option<String>` | Sí | Sinopsis narrativa para la tarjeta translúcida | `len <= 1000` caracteres |
| `background_image_path` | `Option<String>` | Sí | Ruta local relativa al fondo panorámico | Ruta válida o `None` |
| `hero_image_path` | `Option<String>` | Sí | Ruta local del arte del personaje recortado | Ruta válida o `None` |
| `icon_data` | `Option<Vec<u8>>` | Sí | Imagen binaria 1:1 para miniaturas y Dock | Buffer JPEG/PNG procesado o `None` |
| `created_at` | `DateTime<Utc>` | No | Marca de tiempo de registro | Fecha válida UTC |

---

### Entidad: `Comic`
Representa un tomo o archivo digital individual asociado a una colección.

| Campo | Tipo | Nulo | Descripción |
|---|---|---|---|
| `id` | `i64` | No | Clave primaria |
| `collection_id` | `i64` | No | Clave foránea referenciando a `collections(id)` |
| `path` | `String` | No | Ruta física del archivo (`.cbz` / `.cbr`) |
| `title` | `String` | No | Título del cómic |
| `series` | `Option<String>` | Sí | Saga o serie |
| `number` | `Option<i32>` | Sí | Número de entrega |
| `year` | `Option<i32>` | Sí | Año de publicación |
| `page_count` | `i32` | No | Total de páginas |
| `cover_data` | `Option<Vec<u8>>` | Sí | Miniatura JPEG de portada |

---

## 2. Esquema de Base de Datos SQLite (Migración Idempotente)

```sql
-- Tabla collections con soporte para protagonist
CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    protagonist TEXT,
    description TEXT,
    background_image_path TEXT,
    hero_image_path TEXT,
    icon_data BLOB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Índices de consulta rápida
CREATE INDEX IF NOT EXISTS idx_collections_created_at ON collections(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_comics_collection_id ON comics(collection_id);
```

---

## 3. DTOs de Comunicación IPC (Tauri Frontend ↔ Backend Rust)

### `CollectionDto` (Frontend Representation)
```typescript
interface CollectionDto {
  id: number;
  name: string;
  protagonist: string; // fallback a name si está vacío
  description: string;
  backgroundImageUrl: string | null;
  heroImageUrl: string | null;
  iconDataUrl: string | null;
  comicCount: number;
  createdAt: string;
}
```

### `SaveCollectionPayload`
```typescript
interface SaveCollectionPayload {
  id?: number;
  name: string;
  protagonist?: string;
  description?: string;
  backgroundImagePath?: string;
  heroImagePath?: string;
  iconBytes?: number[];
}
```
