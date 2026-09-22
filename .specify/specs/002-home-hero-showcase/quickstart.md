# Quickstart & Validation Guide: 002-home-hero-showcase

**Feature**: `002-home-hero-showcase`  
**Date**: 2026-09-15  
**Status**: Completed  

---

## 1. Preparación y Compilación

```powershell
cd c:\Workspace\comic_order\app
cargo check
cargo test
```

---

## 2. Escenarios de Validación de Extremo a Extremo

### Escenario 1: Arranque Maximizado y Estado Vacío
1. Asegurarse de que no existan colecciones registradas (o iniciar con base de datos limpia de pruebas).
2. Ejecutar la aplicación:
   ```powershell
   cargo run
   ```
3. **Verificación Esperada**:
   - La ventana se abre maximizada ocupando toda la pantalla del monitor (`window::Settings { maximized: true, .. }`).
   - La pantalla se presenta con fondo negro absoluto y el texto centrado *"Crea tu primera colección"*.
   - La barra superior muestra el logotipo y las opciones limpias organizadas en menús temáticos.

### Escenario 2: Creación de Colección con Fondo y Personaje Lateral
1. En la pantalla de inicio, hacer clic en *"Crea tu primera colección"* o en `COLECCIONES ▾ -> Nueva colección`.
2. Completar:
   - Nombre: `Spider-Verse`
   - Descripción: `Historias y aventuras del multiverso arácnido.`
   - Imagen de fondo: seleccionar un wallpaper panorámico (opcional).
   - Imagen lateral: seleccionar una imagen PNG con transparencia (opcional).
   - Guardar.
3. **Verificación Esperada**:
   - Los archivos se copian optimizados a `data/media/collections/`.
   - La vista Hero se actualiza inmediatamente mostrando el fondo, el arte lateral a la derecha, el badge *"Hello my name is Spider-Verse"*, el título en relieve y el botón *"READ NOW"*.
   - En la franja inferior aparece la primera tarjeta con la miniatura.

### Escenario 3: Carrusel de Múltiples Colecciones, Rotación de 6s y Hover Pause
1. Crear 2 o más colecciones adicionales (`Batman`, `X-Men`, etc.).
2. Observar la pantalla Home en reposo sin mover el ratón.
3. **Verificación Esperada**:
   - Cada 6 segundos la vista realiza una transición suave *Slide & Cross-fade* a 60 FPS hacia la siguiente colección.
   - La franja inferior muestra hasta 5 colecciones ordenadas de más reciente a más antigua (de izquierda a derecha).
4. Mover el cursor del ratón sobre la vista Hero o sobre las tarjetas inferiores.
5. **Verificación Esperada**:
   - La rotación de 6 segundos se pausa inmediatamente (*Hover Pause*).
   - Al retirar el cursor, la rotación se reanuda.
6. Hacer clic en una tarjeta específica de la franja inferior:
   - Salta de inmediato a la colección seleccionada con la animación *Slide & Cross-fade*.
7. Pulsar el botón *"READ NOW"*:
   - Emite el evento para navegar a la organización de la colección activa.