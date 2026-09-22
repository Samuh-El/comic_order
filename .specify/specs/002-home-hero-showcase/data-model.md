# Phase 1: Data Model - 002-home-hero-showcase

**Feature**: `002-home-hero-showcase`  
**Date**: 2026-09-15  
**Status**: Completed  

---

## 1. Entidades del Dominio

### 1.1 `Collection` (Entidad Extendida)

Representa una saga o grupo lógico de cómics indexados, ahora enriquecida con metadatos para la presentación cinematográfica Hero.

```rust
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub background_image_path: Option<String>,
    pub hero_image_path: Option<String>,
    pub icon_data: Option<Vec<u8>>,
    pub created_at: String, // Formato ISO 8601 UTC
}
```

- **Invariantes y Reglas de Negocio**:
  - `id`: Clave primaria autoincremental única.
  - `name`: No puede ser vacío ni contener solo espacios en blanco (mínimo 1 carácter, máximo 100 caracteres).
  - `description`: Opcional. Texto descriptivo o sinopsis de la serie/colección (máximo 1.000 caracteres).
  - `background_image_path`: Opcional. Ruta relativa interna (`data/media/collections/{id}_bg.jpg`).
  - `hero_image_path`: Opcional. Ruta relativa interna (`data/media/collections/{id}_hero.png`).
  - `created_at`: Generado automáticamente al insertar la colección; utilizado como criterio canónico para ordenar las últimas 5 colecciones de más reciente a más antigua.

---

### 1.2 `HeroShowcaseState` (Estado de UI en Memoria)

Encapsula el ciclo de vida, animación y controles de la pantalla Home Hero Showcase.

```rust
#[derive(Debug, Clone)]
pub struct HeroShowcaseState {
    pub recent_collections: Vec<Collection>, // Máximo 5 colecciones (más reciente en índice 0)
    pub active_index: usize,                 // Índice actual (0..recent_collections.len())
    pub is_hovered: bool,                    // Indicador para Hover Pause
    pub elapsed_secs: f32,                   // Contador acumulador de segundos (0.0..6.0)
    pub transition: Option<SlideTransition>, // Animación activa a 60 FPS
}

#[derive(Debug, Clone)]
pub struct SlideTransition {
    pub from_index: usize,
    pub to_index: usize,
    pub progress: f32,       // De 0.0 (inicio) a 1.0 (finalizado)
    pub duration_ms: f32,    // 400.0 ms
}
```

- **Máquina de Estados del Carrusel**:
  - `Empty`: `recent_collections.is_empty()` -> Muestra pantalla negra con 'Crea tu primera colección'.
  - `Single`: `recent_collections.len() == 1` -> Muestra la colección fija; temporizador de 6s inactivo.
  - `Playing`: `recent_collections.len() >= 2` y `!is_hovered` y `transition.is_none()` -> El temporizador acumula tiempo hasta 6.0s.
  - `Paused`: `is_hovered == true` -> `elapsed_secs` se congela.
  - `Transitioning`: `transition.is_some()` -> Suscripción activa a 60 FPS (~16ms por tick) actualizando `progress` hasta 1.0. Al alcanzar 1.0, `active_index` pasa a `to_index`, `transition` se destruye y `elapsed_secs` se reinicia a 0.0s.

---

### 1.3 `TopBarState` (Navegación Superior)

```rust
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TopBarMenu {
    #[default]
    None,
    Collections,
    Comics,
    Server,
}

#[derive(Debug, Clone)]
pub struct TopBarState {
    pub open_menu: TopBarMenu,
    pub current_view: AppView,
}
```

---

## 2. Esquema de Persistencia SQLite (`app/data/comic.db`)

### 2.1 Tabla `collections` Actualizada

```sql
CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    icon_data BLOB,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    description TEXT,
    background_image_path TEXT,
    hero_image_path TEXT
);
```

### 2.2 Migración Idempotente en `Database::initialize_schema`

```rust
let _ = sqlx::query("ALTER TABLE collections ADD COLUMN description TEXT;")
    .execute(&self.pool)
    .await;

let _ = sqlx::query("ALTER TABLE collections ADD COLUMN background_image_path TEXT;")
    .execute(&self.pool)
    .await;

let _ = sqlx::query("ALTER TABLE collections ADD COLUMN hero_image_path TEXT;")
    .execute(&self.pool)
    .await;
```

---

## 3. Almacenamiento Local de Medios (`app/data/media/collections/`)

- `data/media/collections/{collection_id}_bg.jpg`: Imagen panorámica optimizada (JPEG 1920x1080 máx, calidad 85%).
- `data/media/collections/{collection_id}_hero.png`: Arte lateral con canal alfa preservado (PNG recortado máx 1200px de altura).