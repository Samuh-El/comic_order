# Guía de Inicio Rápido y Validación: Reorganización Modular Hexagonal

**Feature**: `001-reorganize-app-modular`  
**Fecha**: 2026-09-15  
**Estado**: Completado  

Esta guía proporciona los pasos y escenarios ejecutables para compilar, ejecutar y validar de extremo a extremo que la aplicación funciona correctamente tras la reorganización modular.

---

## 1. Requisitos Previos

1. **Rust y Cargo**: Versión estable instalada (edición 2021).
2. **Entorno de Compilación**:
   - En Windows: Visual Studio Build Tools 2022 con C++ Desktop Development y Windows 11 SDK.
3. **Navegador Web o Dispositivo Móvil**: Para probar el lector remoto y/o cliente OPDS en la misma red local.

---

## 2. Compilación y Ejecución

Desde la raíz del proyecto, ingresa a la carpeta `app/` y compila en modo desarrollo:

```powershell
cd C:\Workspace\comic_order\app
cargo check
cargo run
```

O utilizando el script automatizado para Windows:
```cmd
.\app\build.cmd
```

---

## 3. Escenarios de Validación de Extremo a Extremo

### Escenario 1: Navegación y Lectura en Escritorio (Desktop UI)
1. **Acción**: Iniciar la app de escritorio y hacer clic en una colección existente con cómics.
2. **Resultado Esperado**:
   - La cuadrícula carga de inmediato mostrando las portadas de los cómics.
   - Al hacer clic en un cómic, se abre el visor a pantalla completa en < 100 ms.
   - Las flechas del teclado `←` / `→` cambian de página de manera fluida.
   - La rueda del ratón realiza zoom dinámico y el arrastre permite desplazar la imagen.

### Escenario 2: Servidor Web Local y Lectura Móvil (Acceso Remoto QR)
1. **Acción**: En la app de escritorio, presionar el botón **"📱 Compartir QR"**.
2. **Resultado Esperado**:
   - El botón cambia de color a verde con el texto `[ON] Servidor Activo`.
   - Se despliega el modal con el código QR y la URL `http://<IP_LOCAL>:8080/?token=<TOKEN>`.
   - Al abrir la URL desde el navegador móvil (o navegador local), la SPA carga la biblioteca, permitiendo seleccionar cómics y pasar páginas con gestos táctiles (*swipe*).

### Escenario 3: File Watcher Reactivo (Patrón Observador)
1. **Acción**: Con la aplicación abierta, copiar un nuevo archivo `.cbz` en la carpeta física de una colección registrada.
2. **Resultado Esperado**:
   - En menos de 2 segundos, el *File Watcher* detecta el archivo nuevo, el trabajador de ingesta genera la miniatura y el nuevo cómic aparece en la cuadrícula sin necesidad de recargar manualmente.

### Escenario 4: Feed OPDS para Lectores de Terceros
1. **Acción**: Con el servidor activo, realizar una petición HTTP GET a `http://localhost:8080/opds?token=<TOKEN>` con cabecera `Accept: application/atom+xml`.
2. **Resultado Esperado**:
   - El servidor responde `200 OK` con un feed XML Atom válido que lista las colecciones disponibles.

### Escenario 5: Persistencia de Caché en Disco
1. **Acción**: Solicitar páginas de un cómic y verificar el directorio local `app/.cache/`.
2. **Resultado Esperado**:
   - Las imágenes procesadas se almacenan en `app/.cache/` y no son trackeadas por git gracias a la regla en `.gitignore`.
