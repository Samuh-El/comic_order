# Quickstart & Verification: 004-collection-comics-view

## Escenarios de Validación Rápida

### Escenario 1: Navegación desde Hero Showcase ("LEER MÁS")
1. Iniciar la aplicación (`cargo tauri dev` o ejecutar el binario).
2. Crear o seleccionar una colección con cómics indexados.
3. Hacer clic en el botón principal **"LEER MÁS"**.
4. **Resultado esperado**: La pantalla principal transiciona a la vista `#collection-comics-view`, mostrando la cabecera de la saga y la grilla de cómics ordenados de la A a la Z.

---

### Escenario 2: Navegación desde Menú "COLECCIONES" del Navbar
1. Desplegar el menú "COLECCIONES" en la barra superior.
2. Hacer clic en cualquiera de las colecciones listadas.
3. **Resultado esperado**: La vista de cómics de esa colección se carga y renderiza inmediatamente.

---

### Escenario 3: Apertura y Lectura de Cómic
1. Dentro de la grilla de cómics, hacer clic en cualquier tarjeta de portada.
2. **Resultado esperado**: Se abre el visor a pantalla completa en la primera página del tomo.
3. Presionar la flecha derecha para avanzar de página.
4. Presionar `Escape` para cerrar el visor y confirmar que regresa a la grilla de la colección.

---

### Escenario 4: Botón "Volver al Inicio"
1. En la vista de cómics, hacer clic en el botón *"← Volver al Inicio"* o en el botón *"INICIO"* del Navbar.
2. **Resultado esperado**: Se oculta la grilla de cómics y se vuelve a mostrar el Hero Showcase cinematográfico.
