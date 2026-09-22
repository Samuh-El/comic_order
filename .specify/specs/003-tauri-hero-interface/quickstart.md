# Quickstart: Guía de Validación y Ejecución

**Feature**: `003-tauri-hero-interface`
**Fecha**: 2026-09-16

---

## 1. Requisitos Previos

- Rust (versión 1.80+ recomendada) con `cargo`.
- Node.js (v18+) y gestor de paquetes (npm / pnpm / cargo-tauri).
- Windows 10/11 con Microsoft Edge WebView2 runtime (preinstalado por defecto en Windows).

---

## 2. Compilación y Ejecución en Desarrollo

Para compilar y ejecutar la aplicación de escritorio en modo desarrollo con recarga interactiva:

```bash
# Desde la carpeta app/
cd c:\Workspace\comic_order\app

# Ejecución del entorno Tauri
cargo tauri dev
```

---

## 3. Escenarios de Validación End-to-End

### Escenario 1: Arranque Limpio (Estado Inicial sin Colecciones)
1. Iniciar la aplicación sin base de datos o con una base de datos vacía.
2. **Resultado Esperado**: La ventana principal abre maximizada en menos de 1 segundo, mostrando el estado de bienvenida "Crea tu primera colección", con fondo cómic oscuro y 0 errores en consola.

### Escenario 2: Creación de Colección con Protagonista
1. Abrir el menú `COLECCIONES ▾` en el Navbar y hacer clic en `Nueva Colección`.
2. Completar:
   - Nombre: `Spider-Man: Spider-Verse`
   - Protagonista: `SPIDER-MAN`
   - Sinopsis: `Miles Morales se convierte en el nuevo Spider-Man...`
   - Imagen de Héroe: `assets/miles_hero.png`
   - Fondo: `assets/city_bg.jpg`
3. Presionar **Guardar Colección**.
4. **Resultado Esperado**: La pantalla Home se actualiza al instante; el cartel rojo muestra "SPIDER-MAN" en relieve, el escenario proyecta a Miles Morales y la miniatura aparece iluminada en el Dock inferior.

### Escenario 3: Navegación e Interacción con el Dock
1. Crear una segunda colección con protagonista `BATMAN`.
2. Hacer clic entre la tarjeta de Spider-Man y la tarjeta de Batman en el Dock inferior.
3. **Resultado Esperado**: Conmutación suave instantánea (<50ms) con actualización de temas de color, protagonista y sinopsis.

### Escenario 4: Apertura de Dossier
1. Hacer clic en el botón principal **READ MORE**.
2. **Resultado Esperado**: Apertura fluida del modal tipo Dossier con la ficha completa y lista de cómics.
