# Proyecto "Comic"

Comic es una app para gestionar y leer comics, escrito en Rust.

## Tecnologias

- Rust
- Iced (para la UI)
- SQLite (BD local)

## Características

- Permite crear colecciones de comics, agregando rutas a ellas.
- Permite leer desde otros dispositivos, usando esta app como servidor. Esta vinculación se hace a través de un código QR.
- Permite organizar las vista de los comics, agregar su año, número, saga, título, editar portada, etc.
- Funciona en Windows, Linux y Mac.

## Funcionamiento

- El usuario crea una colección de comics, agregando rutas a ellas.
- La app lee los comics (cbr, cbz, pdf, etc) y los organiza en la base de datos local.
- El usuario puede hacer click para leerlo.
- El usuario puede dar acceso a otros para leer desde otro dispositivo, usando esta app como servidor. Esta vinculación se hace a través de un código QR.

## Requisitos

- **Rust** (stable) — [https://rustup.rs](https://rustup.rs)
- **Visual Studio Build Tools 2022** con el componente "Desarrollo de escritorio con C++" (necesario para compilar en Windows)
- **Windows 11 SDK** (se instala junto con VS Build Tools)

## Base de Datos

La aplicación utiliza **SQLite**, por lo que no es necesario instalar un servidor de base de datos como MySQL. El archivo de base de datos (`comic.db`) se crea automáticamente en la raíz del proyecto al iniciar la aplicación.

## Compilar y Ejecutar

### Opción 1: Usando el script (recomendado en Windows)

```bash
.\build.cmd
```

Este script configura el entorno MSVC y ejecuta `cargo run`.

### Opción 2: Manualmente

```bash
cargo run
```

> Si `cargo run` falla con error de linker, asegúrate de ejecutar desde una **Developer Command Prompt** de Visual Studio o usar `build.cmd`.

## Configuración

La app utiliza por defecto el archivo `comic.db` en la raíz del proyecto. El servidor HTTP para acceso remoto escucha en el puerto `8080`.

Para cambiar el puerto del servidor, edita la constante en `src/main.rs`:

```rust
const SERVER_PORT: u16 = 8080;
```

## Uso

1. **Crear colección**: Clic en "Nueva Colección" en el panel izquierdo.
2. **Agregar carpeta**: Selecciona la colección y haz clic en "Añadir Carpeta". Elige un directorio con archivos `.cbz`.
3. **Leer comic**: Haz clic en la portada de un comic para abrirlo. Navega con los botones o las flechas del teclado.
4. **Editar metadatos**: Usa el botón ✏️ debajo de cada comic para editar título, año, número y saga.
5. **Compartir (QR)**: Haz clic en "📱 Compartir QR" para iniciar el servidor HTTP. Se mostrará un código QR en pantalla.

## Acceso Remoto (QR)

La app puede funcionar como servidor para que otros dispositivos lean los comics desde un navegador web.

### Pasos para conectarse desde otro dispositivo:

1. Abre la app y selecciona una colección que tenga comics.
2. Haz clic en el botón **"📱 Compartir QR"** en la barra superior de la colección.
3. Se abrirá un overlay con:
   - Un **código QR** que puedes escanear con la cámara de tu celular/tablet
   - La **URL del servidor** (ej: `http://192.168.1.100:8080`)
4. Desde el otro dispositivo:
   - **Opción 1**: Escanea el QR con la cámara → se abrirá el navegador automáticamente
   - **Opción 2**: Abre un navegador y escribe la URL manualmente
5. En el navegador verás todas las colecciones y podrás leer los comics.
6. Para detener el servidor, haz clic en **"Cerrar"** en el overlay del QR.

### Requisitos para el acceso remoto:

- Ambos dispositivos deben estar en la **misma red WiFi/LAN**
- El puerto `8080` debe estar permitido en el firewall de Windows
- La app debe estar ejecutándose en el PC

### Abrir el puerto en el firewall (si es necesario):

```powershell
New-NetFirewallRule -DisplayName "Comic Reader" -Direction Inbound -LocalPort 8080 -Protocol TCP -Action Allow
```
