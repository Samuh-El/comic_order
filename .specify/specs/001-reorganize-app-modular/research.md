# Investigación Técnica y Decisiones de Diseño: Reorganización Modular Hexagonal

**Feature**: `001-reorganize-app-modular`  
**Fecha**: 2026-09-15  
**Estado**: Completado  

Este documento consolida las investigaciones, evaluaciones técnicas y justificaciones para la reorganización del código en `app/src/` bajo la Arquitectura Hexagonal y el Monolito Modular.

---

## 1. Estrategia de Migración y Desacoplamiento

### Decisión
Adoptar una **migración incremental por capas**, organizada en cuatro etapas estrictas:
1. **Etapa 1**: Creación del núcleo del dominio (`app/src/domain/`) y los contratos de puertos (traits).
2. **Etapa 2**: Implementación de los adaptadores de salida (`app/src/adapters/outbound/`), incluyendo el repositorio SQLite y los lectores de cómics (`ComicFileReader`).
3. **Etapa 3**: Construcción de la capa de aplicación y la fachada unificada (`ComicFacade`).
4. **Etapa 4**: Conexión de los adaptadores de entrada (`app/src/adapters/inbound/` para Iced Desktop y Axum REST/OPDS) y orquestador `main.rs`.

### Justificación
- Elimina el riesgo de rotura de compilación masiva en el workspace.
- Permite verificar unitariamente cada capa con `cargo check` y pruebas antes de proceder a la siguiente.
- Asegura que el dominio permanezca completamente puro y desacoplado de dependencias de GUI o red.

### Alternativas Consideradas
- *Reemplazo destructivo en un solo paso*: Descartado por el alto riesgo de inconsistencias de tipado y dependencias circulares complejas de depurar en Rust.
- *Coexistencia temporal con código duplicado*: Descartado porque aumentaría la complejidad y el mantenimiento de dos árboles paralelos de código.

---

## 2. Proxy de Caché de Imágenes y Páginas

### Decisión
Implementar un **`ImageCacheProxy` con almacenamiento persistente en disco en `app/.cache/`**, complementado con políticas de limpieza automática por tamaño máximo (ej. 500 MB) y tiempo de retención.

### Justificación
- Almacenar páginas redimensionadas para dispositivos móviles en disco local reduce drásticamente el consumo continuo de CPU en relecturas.
- Evita descompresiones repetitivas de archivos `.cbz` y `.cbr` pesados (de cientos de megabytes).
- Protege la memoria RAM en entornos donde múltiples dispositivos leen al mismo tiempo.
- El directorio `app/.cache/` está explícitamente excluido del repositorio en `.gitignore`.

### Alternativas Consideradas
- *Caché exclusivamente en RAM*: Descartada por elección del usuario (Opción B de la clarificación Q2), debido al riesgo de agotar la memoria del sistema anfitrión si la biblioteca o las resoluciones solicitadas son elevadas.
- *Sin caché*: Descartada porque degradaría el rendimiento de streaming hacia dispositivos móviles.

---

## 3. Mecanismo de File Watcher e Ingesta Asíncrona

### Decisión
Utilizar la biblioteca **`notify`** sobre el runtime asíncrono Tokio con un canal `mpsc` y una ventana de **debouncing de 500 ms**, combinada con un **escaneo de verificación al inicio de la aplicación**.

### Justificación
- `notify` se apoya en los mecanismos nativos del sistema operativo (`ReadDirectoryChangesW` en Windows, `inotify` en Linux, `FSEvents` en macOS), consumiendo 0% de CPU cuando el disco está en reposo.
- El *debouncing* de 500 ms evita disparar múltiples eventos cuando se copian archivos grandes en bloques sucesivos.
- El escaneo inicial al arrancar garantiza que cualquier cómic añadido mientras la app estuvo apagada sea indexado inmediatamente.

### Alternativas Consideradas
- *Polling periódico continuo*: Descartado por consumo innecesario de I/O de disco y latencia de detección fija.
- *Escaneo puramente manual*: Descartado porque no cumple el requisito de observabilidad reactiva para bibliotecas en crecimiento.

---

## 4. Puerto de Entrada OPDS y WebSockets

### Decisión
Implementar un **feed OPDS 1.2 funcional basado en XML Atom** con endpoints bajo `/opds`, y formalizar el contrato de puerto en el dominio para WebSockets de sincronización de progreso de lectura.

### Justificación
- OPDS es el estándar indiscutible de la industria para lectores móviles de cómics (Panels, Chunky, Kuro Reader). Al proveer `/opds`, la app se integra nativamente con el ecosistema existente sin necesidad de desarrollar apps móviles propias.
- La serialización XML Atom se genera directamente con plantillas o constructores ligeros sin librerías externas pesadas.
- Definir el puerto de WebSockets en el dominio deja la arquitectura preparada para sincronización bidireccional en tiempo real sin inflar el alcance de la reorganización inicial.

### Alternativas Consideradas
- *Implementar WebSockets bidireccionales completos de inmediato*: Descartado para mantener el alcance acotado y seguro en la primera fase.
- *No incluir OPDS*: Descartado porque limitaría la lectura remota exclusivamente al navegador web.

---

## 5. Descubrimiento de Servicios mDNS en Red Local

### Decisión
Incorporar el anuncio de servicio mDNS/Zeroconf anunciando el servicio HTTP/OPDS local en el puerto `8080` bajo el identificador `_comic._tcp.local`.

### Justificación
- Permite que aplicaciones OPDS y dispositivos en la red LAN localicen automáticamente el servidor sin que el usuario deba averiguar o escribir manualmente la dirección IP privada de la PC.

---

## 6. Patrones de Diseño Implementados

| Patrón | Módulo / Componente | Propósito |
| :--- | :--- | :--- |
| **Adaptador (*Adapter*)** | `ComicFileReader` (`CbzAdapter`, `CbrAdapter`) | Unifica la descompresión y lectura de formatos heterogéneos bajo una interfaz común. |
| **Fachada (*Facade*)** | `ComicFacade` | Expone métodos de alto nivel para la UI y la API (`get_page_resized`, `index_comic`). |
| **Observador (*Observer*)** | `FileWatcher` → `IngestionWorker` | Emite eventos reactivos de adición/borrado de archivos hacia el servicio de catálogo. |
| **Estrategia (*Strategy*)** | `ReadingStrategy` & `ResizePolicy` | Permite alternar algoritmos de lectura (página simple, doble, webtoon) y escalado según cliente. |
| **Proxy / Caché (*Cache Proxy*)** | `ImageCacheProxy` | Intercepta peticiones de imágenes sirviendo archivos desde `app/.cache/` o delegando al motor. |
| **Repositorio (*Repository*)** | `ComicRepository`, `CollectionRepository` | Abstrae las consultas SQL de SQLite detrás de traits puros del dominio. |
