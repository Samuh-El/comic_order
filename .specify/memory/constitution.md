# Constitution

El presente proyecto trabaja utilizando como base de trabajo **SDD (Spec-Driven Development)**. Por lo tanto, su estructura, directrices y gobernanza se basan rigurosamente en esta metodología.

## Finalidad del proyecto

**Comic** es una aplicación de alto rendimiento para escritorio y servidor local desarrollada en **Rust**, diseñada para la catalogación, organización, edición de metadatos y lectura de cómics digitales en formatos empaquetados (`.cbz`, `.cbr`, y soporte proyectado para `.pdf` y `.epub`).

Sus objetivos fundamentales son:
1. **Gestión de biblioteca local**: Permitir a los usuarios crear colecciones lógicas vinculadas a carpetas del sistema de archivos, indexando automáticamente cómics y extrayendo miniaturas de portadas y número de páginas en una base de datos local SQLite.
2. **Edición de metadatos**: Proporcionar herramientas interactivas para catalogar título, año de publicación, número de entrega y saga de cada volumen.
3. **Lectura en escritorio optimizada**: Ofrecer un visor gráfico acelerado por hardware con navegación secuencial rápida, atajos de teclado, zoom dinámico continuo (10% a 500%) y paneo interactivo mediante arrastre.
4. **Acceso remoto multidispositivo (Servidor Web + QR)**: Convertir la aplicación en un servidor web local (puerto `8080`) accesible desde teléfonos móviles, tabletas u ordenadores dentro de la misma red local (Wi-Fi/LAN), mediante sincronización instantánea por código QR y autenticación mediante tokens de sesión y dispositivos de confianza (*trusted devices*).
5. **Privacidad y autonomía (*Offline-First*)**: Funcionar de manera 100% autónoma y local, sin telemetría, sin depender de servicios en la nube ni de conexiones a internet externas.

---

# Arquitectura del Sistema

El proyecto implementa un **Monolito Modular** estructurado bajo **Arquitectura Hexagonal (Puertos y Adaptadores / Clean Architecture)** y un **Modelo Cliente-Servidor** para red local:

## 1. Estilo Arquitectónico Principal

- **Monolito Modular**: Todo el sistema se empaqueta y distribuye en una única unidad ejecutable autónoma, organizado internamente en módulos estrictamente delimitados e independientes, evitando la complejidad innecesaria de microservicios en entornos locales.
- **Arquitectura Hexagonal**:
  - **Núcleo del Dominio**: Contiene las entidades puras y la lógica de negocio (catálogo, colecciones, metadatos, listas de lectura y progreso) totalmente aisladas de frameworks, bases de datos o detalles de UI.
  - **Puertos de Entrada (Primary / Inbound)**:
    - API REST (Axum) para clientes web locales y lectores remotos.
    - Protocolo estándar **OPDS** (*Open Publication Distribution System*) para clientes y lectores de terceros en tablets/móviles.
    - WebSockets para sincronización de progreso de lectura en tiempo real.
    - Adaptador de GUI de Escritorio (Iced) para interacción del usuario host.
  - **Puertos de Salida (Secondary / Outbound)**:
    - Adaptador del sistema de archivos local para lectura de volúmenes.
    - Adaptador de base de datos relacional para metadatos y configuración.
    - Adaptador de almacenamiento en caché para miniaturas y páginas renderizadas.
- **Modelo Cliente-Servidor LAN**: El programa actúa como servidor central en la máquina host donde residen los archivos físicos de cómics y transmite páginas e información a clientes web o aplicaciones móviles dentro de la red doméstica.

## 2. Capas y Módulos Clave

1. **Capa de Presentación y API**: Expone endpoints HTTP/REST, feeds OPDS y la interfaz de usuario de escritorio.
2. **Módulo de Escaneo e Ingesta (*File Watcher & Ingestion Worker*)**: Procesa el sistema de archivos de forma asíncrona en segundo plano, detecta adiciones o cambios en cómics, extrae metadatos embebidos (`ComicInfo.xml`) y genera miniaturas sin bloquear la interfaz.
3. **Módulo del Dominio (*Catálogo y Colecciones*)**: Encapsula las reglas de negocio de series, tomos, editoriales, etiquetas, colecciones personalizadas y estado de lectura por usuario/dispositivo.
4. **Capa de Persistencia y Caché**:
   - **Base de datos relacional ligera** (SQLite local con SQLx en modo WAL).
   - **Caché de imágenes y miniaturas** bajo demanda para evitar descompresiones repetitivas de archivos pesados.

## 3. Patrones de Diseño Obligatorios

- **Patrón Adaptador (*Adapter Pattern*)**: Todos los formatos de cómic (`.cbz`, `.cbr`, `.pdf`, `.epub`) se encapsulan tras la abstracción común `ComicFileReader`, permitiendo incorporar nuevos formatos sin alterar el dominio ni la presentación.
- **Patrón Fachada (*Facade Pattern*)**: `ComicFacade` unifica operaciones compuestas (ej. "obtener página X del cómic Y redimensionada"), ocultando los pasos internos de localización, descompresión en memoria, escalado y caché.
- **Patrón Observador (*Observer Pattern / Event-Driven*)**: El *File Watcher* monitorea el disco y emite eventos hacia el trabajador de ingesta para sincronizar la base de datos de manera reactiva.
- **Patrón Estrategia (*Strategy Pattern*)**: Permite alternar dinámicamente modos de lectura (página individual, doble página, vertical tipo Webtoon) y políticas de compresión de imágenes según las capacidades del dispositivo cliente.
- **Patrón Proxy / Caché (*Cache Proxy*)**: Intercepta las solicitudes de imágenes y páginas: entrega de inmediato las instancias cacheadas o delega la descompresión al motor del cómic si no existen.

## 4. Características de Red Local y Escalabilidad

- **Streaming e imágenes bajo demanda**: La API envía páginas individuales redimensionadas al vuelo según la resolución solicitada por el dispositivo cliente.
- **Descubrimiento de Servicios (mDNS / Zeroconf)**: Anuncio automático en la red local para que dispositivos móviles descubran el servidor sin requerir la introducción manual de la dirección IP.
- **Control de Concurrencia**: Despacho asíncrono con Tokio para permitir lecturas simultáneas de múltiples dispositivos en paralelo sin bloqueos mutuos.

---

# Estructuras

## Estructura de archivos

1. Todo el software, módulos, dependencias y archivos de desarrollo de la aplicación DEBEN ubicarse exclusivamente en la carpeta `app/` en la raíz del proyecto.
2. Dentro de `app/`, el código fuente reside en `app/src/`, los recursos gráficos en `app/assets/`, la configuración del paquete en `app/Cargo.toml`, los scripts de ejecución en `app/scripts/build.cmd` y `app/scripts/genera_exe.cmd`, y los datos de persistencia en `app/data/`.
3. Los artefactos generados de compilación (`app/target/`, `app/release/`), la base de datos local (`app/data/comic.db*`), los logs (`app/log/`), el directorio de caché (`app/.cache/`) y archivos temporales están expresamente excluidos del control de versiones mediante `.gitignore`.
4. Queda estrictamente PROHIBIDO colocar código fuente o archivos de la aplicación fuera del directorio `app/`.

## Estructura de `.specify/`

La carpeta `.specify/` se reserva **EXCLUSIVAMENTE** para la infraestructura del flujo spec-kit y DEBE contener únicamente:

- `memory/` — constitución y memoria persistente del proyecto.
- `scripts/` — scripts de soporte y automatización de spec-kit.
- `templates/` — plantillas canónicas de spec, plan, tasks y checklist.
- `extensions.yml` — configuración de hooks de spec-kit.
- `feature.json` — estado de la feature activa (apuntando a `specs/<NNN>-...`).

Las especificaciones de características se gestionan canónicamente en `.specify/specs/<numero>-<nombre>/` (según las herramientas upstream de spec-kit).

---

# Forma de trabajar

## Método de Trabajo

Cada iniciativa o nueva funcionalidad DEBE contener exactamente tres archivos en su directorio de feature:

- `spec.md` — objetivo de negocio, alcance, requisitos funcionales, criterios de aceptación y dependencias.
- `plan.md` — decisiones técnicas, arquitectura, módulos afectados y orden de implementación.
- `tasks.md` — lista de pasos secuenciales, accionables y verificables, marcables como completados (`[X]`).

El flujo obligatorio mínimo es:
`speckit.specify` → `speckit.plan` → `speckit.tasks` → `speckit.implement`.

Para specs fundacionales o de alto impacto se recomienda añadir `speckit.clarify` (entre specify y plan) y `speckit.analyze` (entre plan y tasks). Ninguna fase obligatoria puede saltarse. Cada tarea completada DEBE marcarse como `[X]` en `tasks.md`.

## Modo Interactivo de Preguntas

Los comandos `speckit.specify` y `speckit.clarify` operan en **modo interactivo obligatorio**: presentan sus preguntas de una en una y esperan respuesta antes de continuar. El comando `speckit.plan` opera en **modo interactivo condicional**: solo lanza preguntas si existen decisiones estructurales que afecten specs futuras y que no estén resueltas en la constitución ni en la spec vigente.

## Spec-Driven Development

- NO implementar nada que no esté descrito en un `spec.md` aprobado.
- El orden de implementación lo define el prefijo numérico de cada spec.
- Toda tarea implementada debe rastrearse en `tasks.md`.

## Spec activa

<!-- SPECKIT START -->
Plan vigente: `.specify/specs/004-collection-comics-view`
<!-- SPECKIT END -->

---

# Base de datos (SQLite Local con SQLx)

1. **Motor y Driver**: Se utiliza exclusivamente **SQLite** local a través del driver asíncrono `sqlx` (con `runtime-tokio`, `sqlite` y `macros`).
2. **Ubicación**: El archivo de base de datos es `comic.db`, ubicado en `app/data/comic.db` (ignorado en git).
3. **Concurrencia y WAL**: Toda conexión a la base de datos DEBE habilitar inmediatamente el modo WAL (`PRAGMA journal_mode=WAL;`) para asegurar lecturas y escrituras concurrentes sin bloqueos de archivo.
4. **Pool de Conexiones**: Las operaciones de base de datos se canalizan a través de `SqlitePool` con un límite controlado de conexiones (máximo 5 concurrentes).
5. **Integridad Referencial**:
   - Las claves foráneas están activas y configuradas con `ON DELETE CASCADE` en las tablas hijas (`collection_paths` y `comics` referenciando a `collections`).
6. **Caché Binaria de Portadas e Iconos**:
   - Las portadas procesadas (JPEG 300x450) y los iconos de colección se almacenan en columnas `BLOB` para garantizar respuesta instantánea tanto en la UI de escritorio como en la API web sin sobrecargar el disco con descompresiones continuas.
7. **Seguridad en Consultas**:
   - Queda estrictamente PROHIBIDO concatenar cadenas SQL directas. Todas las consultas DEBEN utilizar parámetros vinculados (`.bind()`) para prevenir inyecciones SQL y garantizar tipado estricto.
8. **Evolución del Esquema**:
   - Las modificaciones o migraciones de tablas DEBEN ser idempotentes (`CREATE TABLE IF NOT EXISTS`, `ALTER TABLE ... ADD COLUMN ...` controlados) y ejecutarse durante el arranque en `Database::initialize_schema`.

---

# Gobierno

Esta constitución es el documento rector del proyecto y tiene precedencia sobre cualquier otra práctica, convención o preferencia personal. Las enmiendas DEBEN documentarse con nueva versión semántica:

- **MAJOR**: eliminación o redefinición incompatible de un principio o del stack obligatorio.
- **MINOR**: nuevo principio o sección añadida; expansión material de un principio existente.
- **PATCH**: aclaraciones, redacción o correcciones no semánticas.

---

# Sistema de instrucciones

- Para todo trabajo frontend (Iced GUI y SPA Web), la definición canónica de tokens visuales y directrices vive en `.github/instructions/frontend.instructions.md`.
- Para todo trabajo backend (Rust, Tokio, Axum, Comic Reader), la definición canónica de directrices vive en `.github/instructions/backend.instructions.md`.
- Para todo trabajo con base de datos y persistencia, la definición canónica de directrices vive en `.github/instructions/database.instructions.md`.

---

# Historial de versiones

| Número de versión | Descripción de cambios | Fecha |
|:---|:---|:---|
| v0.1.0 | Redacción y formalización inicial de la constitución del proyecto Comic. | 2026-09-15 |
| v0.2.0 | Adopción formal de Arquitectura Hexagonal (Puertos y Adaptadores), Monolito Modular, patrones de diseño (Adaptador, Fachada, Observador, Estrategia, Proxy/Caché), protocolos OPDS, mDNS y activación de feature `001-reorganize-app-modular`. | 2026-09-15 |