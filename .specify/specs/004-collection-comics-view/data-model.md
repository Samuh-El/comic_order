# Data Model: Vista de Cómics de Colección y Lectura Integrada (004-collection-comics-view)

## Entidades y Estructuras de Datos

### 1. Entidad Cómic (`Comic`)
Entidad del dominio que representa un archivo físico indexado de cómic (`.cbz` / `.cbr`).

| Campo | Tipo | Restricción / Descripción |
| :--- | :--- | :--- |
| `id` | `i64` | Identificador único autoincremental |
| `collection_id` | `i64` | Clave foránea a la colección propietaria |
| `title` | `String` | Título del cómic (máx. 40 caracteres) |
| `file_path` | `PathBuf` | Ruta absoluta al archivo físico (`UNIQUE`) |
| `format` | `ComicFormat` | Formato (`Cbz`, `Cbr`, etc.) |
| `year` | `Option<i32>` | Año de publicación |
| `issue_number` | `Option<i32>` | Número de entrega |
| `saga` | `Option<String>` | Saga o arco argumental |
| `cover_data` | `Option<Vec<u8>>` | Miniatura JPEG de la portada en Base64/BLOB |
| `page_count` | `usize` | Total de páginas del volumen |

---

### 2. DTO de Resumen de Cómic (`ComicSummaryDto`)
Estructura transferida vía IPC hacia la interfaz de usuario para el renderizado de tarjetas en la grilla.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComicSummaryDto {
    pub id: i64,
    pub collection_id: i64,
    pub title: String,
    pub file_path: String,
    pub series: Option<String>,
    pub number: Option<i32>,
    pub year: Option<i32>,
    pub page_count: i32,
    pub has_cover: bool,
    pub cover_base64: Option<String>,
}
```

---

### 3. Modelo de Estado de la Vista de Colección en el Frontend (JavaScript)

```javascript
// Estado en memoria de la vista activa de cómics
let activeCollectionComics = []; // Array de ComicSummaryDto ordenados A-Z
let activeCollectionId = null;
let activeReadingComicIndex = -1; // Índice del cómic actual dentro de activeCollectionComics
```
