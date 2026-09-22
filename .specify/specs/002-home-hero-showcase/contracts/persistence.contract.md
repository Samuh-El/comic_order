# Contract: Persistence & Catalog Ports - 002-home-hero-showcase

**Feature**: `002-home-hero-showcase`  
**Date**: 2026-09-15  
**Status**: Completed  

---

## 1. Puerto Secundario (Outbound Port): `ComicRepository`

```rust
#[async_trait]
pub trait ComicRepository: Send + Sync {
    // Métodos existentes
    async fn get_all_collections(&self) -> Result<Vec<Collection>, DomainError>;
    async fn get_collection_by_id(&self, id: i64) -> Result<Option<Collection>, DomainError>;
    
    // Método especializado para Home Hero Showcase
    async fn get_recent_collections(&self, limit: usize) -> Result<Vec<Collection>, DomainError>;
    
    // Guardado y actualización extendidos
    async fn save_collection(
        &self, 
        collection: &Collection, 
        paths: &[String]
    ) -> Result<i64, DomainError>;
    
    async fn update_collection(&self, collection: &Collection) -> Result<(), DomainError>;
}
```

- **Contrato de Consulta de Colecciones Recientes (`get_recent_collections`)**:
  - Sentencia SQL parametrizada obligatoria:
    ```sql
    SELECT id, name, description, background_image_path, hero_image_path, icon_data, created_at 
    FROM collections 
    ORDER BY datetime(created_at) DESC 
    LIMIT ?;
    ```
  - Parámetro vinculado obligatoriamente con `.bind(limit as i64)`.
  - Retorna como máximo `limit` colecciones ordenadas estrictamente de la más reciente a la más antigua (índice 0 = más reciente).
  - Nunca emite pánicos en tiempo de ejecución; errores de I/O o base de datos se propagan como `Result<_, DomainError>`.