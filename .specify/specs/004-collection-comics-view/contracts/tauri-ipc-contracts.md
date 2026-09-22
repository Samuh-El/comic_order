# Tauri IPC Contracts: Vista de Cómics de Colección (004-collection-comics-view)

## Comandos IPC Involucrados

### 1. `get_comics_by_collection`
Recupera todos los cómics indexados para una colección específica, ordenados alfabéticamente y listos para renderizarse en la grilla.

- **Firma Rust**:
  ```rust
  #[tauri::command]
  pub async fn get_comics_by_collection(
      collection_id: i64,
      state: State<'_, AppState>,
  ) -> Result<Vec<ComicSummaryDto>, String>
  ```
- **Invocación Frontend**:
  ```javascript
  const comics = await invokeBackend('get_comics_by_collection', { collectionId: id });
  ```
- **Respuesta JSON de éxito**:
  ```json
  [
    {
      "id": 1,
      "collection_id": 4,
      "title": "Batman #01 - The Court of Owls",
      "file_path": "C:/Comics/Batman/Batman 01.cbz",
      "series": "Batman (New 52)",
      "number": 1,
      "year": 2011,
      "page_count": 32,
      "has_cover": true,
      "cover_base64": "data:image/jpeg;base64,..."
    }
  ]
  ```

---

### 2. `get_comic_page`
Entrega una página específica de un cómic codificada en Data URL Base64 para el visor de lectura.

- **Firma Rust**:
  ```rust
  #[tauri::command]
  pub async fn get_comic_page(
      comic_id: i64,
      page_index: u32,
      max_width: Option<u32>,
      state: State<'_, AppState>,
  ) -> Result<PageResponseDto, String>
  ```
- **Invocación Frontend**:
  ```javascript
  const page = await invokeBackend('get_comic_page', { comicId, pageIndex: 0 });
  ```
