# Contract: UI Components & TEA Messages - 002-home-hero-showcase

**Feature**: `002-home-hero-showcase`  
**Date**: 2026-09-15  
**Status**: Completed  

---

## 1. Mensajes de la Arquitectura Elm (TEA) para Iced

### 1.1 `TopBarMessage` (Barra Superior)

```rust
#[derive(Debug, Clone)]
pub enum TopBarMessage {
    NavigateTo(AppView),
    ToggleMenu(TopBarMenu),
    CloseMenu,
    NewCollectionClicked,
    RescanLibrariesClicked,
    OpenQrModal,
    OpenSettings,
}
```

- **Contrato de Interacción**:
  - `NavigateTo(AppView)`: Cambia la vista activa de la aplicación y cierra cualquier menú flotante abierto.
  - `ToggleMenu(TopBarMenu)`: Abre o alterna el menú desplegable indicado (`Collections`, `Comics`, `Server`).
  - `CloseMenu`: Cierra el menú desplegable abierto si el usuario hace clic fuera de él.

---

### 1.2 `HeroShowcaseMessage` (Carrusel y Showcase Central)

```rust
#[derive(Debug, Clone)]
pub enum HeroShowcaseMessage {
    AutoplayTick,                     // Disparado periódicamente cada 1s o 100ms
    HoverChanged(bool),               // Cursor entra (true) o sale (false) del área interactiva
    SelectCollection(usize),          // Clic manual en tarjeta inferior (0..5)
    TransitionTick(f32),              // Frame de animación a 60 FPS (delta en ms)
    ReadNowClicked(i64),              // Clic en botón "READ NOW" con el id de la colección activa
    CreateFirstCollectionClicked,     // Clic en estado vacío "Crea tu primera colección"
}
```

- **Contrato de Comportamiento**:
  - `AutoplayTick`: Si `is_hovered == false` y `transition.is_none()`, avanza `elapsed_secs`. Si alcanza 6.0s, inicia `SlideTransition` hacia `(active_index + 1) % recent_collections.len()`.
  - `HoverChanged(true)`: Congela inmediatamente el contador de 6 segundos (*Hover Pause*).
  - `HoverChanged(false)`: Reanuda el contador de 6 segundos.
  - `SelectCollection(target_idx)`: Inicia inmediatamente la transición hacia `target_idx` y reinicia `elapsed_secs = 0.0`.
  - `ReadNowClicked(collection_id)`: Emite el evento de navegación hacia la vista de organización de cómics de esa colección.
  - `CreateFirstCollectionClicked`: Abre el diálogo modal de creación de nueva colección.

---

### 1.3 `CollectionFormMessage` (Formulario Extendido)

```rust
#[derive(Debug, Clone)]
pub enum CollectionFormMessage {
    NameChanged(String),
    DescriptionChanged(String),
    SelectBackgroundImage,
    BackgroundImageSelected(Option<PathBuf>),
    SelectHeroImage,
    HeroImageSelected(Option<PathBuf>),
    Submit,
    Cancel,
}
```

- **Contrato de Procesamiento**:
  - `SelectBackgroundImage` / `SelectHeroImage`: Abre el diálogo nativo asíncrono con `rfd::AsyncFileDialog`.
  - `Submit`: Valida que el nombre no esté vacío, copia las imágenes a `data/media/collections/` y guarda el registro en SQLite.