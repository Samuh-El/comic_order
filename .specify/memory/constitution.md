# Constitution

El presente proyecto trabaja utilizando como base de trabajo **SDD (Spec-Driven Development)**. Por lo tanto, su estructura, directrices y gobernanza se basan rigurosamente en esta metodología.

## Finalidad del proyecto

**Comic** es una aplicación de alto rendimiento para escritorio y servidor local desarrollada en **Rust**, diseñada para la catalogación, organización, edición de metadatos y lectura de cómics digitales en formatos empaquetados (`.cbz`, `.cbr` y soporte proyectado para `.pdf`).

Sus objetivos fundamentales son:
1. **Gestión de biblioteca local**: Permitir a los usuarios crear colecciones lógicas vinculadas a carpetas del sistema de archivos, indexando automáticamente cómics y extrayendo miniaturas de portadas y número de páginas en una base de datos local SQLite.
2. **Edición de metadatos**: Proporcionar herramientas interactivas para catalogar título, año de publicación, número de entrega y saga de cada volumen.
3. **Lectura en escritorio optimizada**: Ofrecer un visor gráfico acelerado por hardware con navegación secuencial rápida, atajos de teclado, zoom dinámico continuo (10% a 500%) y paneo interactivo mediante arrastre.
4. **Acceso remoto multidispositivo (Servidor Web + QR)**: Convertir la aplicación en un servidor web local (puerto `8080`) accesible desde teléfonos móviles, tabletas u ordenadores dentro de la misma red local (Wi-Fi/LAN), mediante sincronización instantánea por código QR y autenticación mediante tokens de sesión y dispositivos de confianza (*trusted devices*).
5. **Privacidad y autonomía (*Offline-First*)**: Funcionar de manera 100% autónoma y local, sin telemetría, sin depender de servicios en la nube ni de conexiones a internet externas.

---

# Estructuras

## Estructura de archivos

1. Todo el software, módulos, dependencias y archivos de desarrollo de la aplicación DEBEN ubicarse exclusivamente en la carpeta `app/` en la raíz del proyecto.
2. Dentro de `app/`, el código fuente reside en `app/src/`, los recursos gráficos en `app/assets/`, la configuración del paquete en `app/Cargo.toml`, y los scripts de ejecución en `app/build.cmd` y `app/genera_exe.cmd`.
3. Los artefactos generados de compilación (`app/target/`, `app/release/`), la base de datos local (`app/comic.db*`), los logs (`app/log/`) y archivos temporales están expresamente excluidos del control de versiones mediante `.gitignore`.
4. Queda estrictamente PROHIBIDO colocar código fuente o archivos de la aplicación fuera del directorio `app/`.

## Estructura de `.specify/`

La carpeta `.specify/` se reserva **EXCLUSIVAMENTE** para la infraestructura del flujo spec-kit y DEBE contener únicamente:

- `memory/` — constitución y memoria persistente del proyecto.
- `scripts/` — scripts de soporte y automatización de spec-kit.
- `templates/` — plantillas canónicas de spec, plan, tasks y checklist.
- `extensions.yml` — configuración de hooks de spec-kit.
- `feature.json` — estado de la feature activa (apuntando a `specs/<NNN>-...`).

Está **PROHIBIDO** crear especificaciones dentro de `.specify/`. Todas las especificaciones de características DEBEN crearse en el directorio raíz `specs/<numero>-<nombre>/`.

---

# Forma de trabajar

## Método de Trabajo

Cada iniciativa o nueva funcionalidad DEBE contener exactamente tres archivos en su directorio `specs/<feature>/`:

- `spec.md` — objetivo de negocio, alcance, requisitos funcionales, criterios de aceptación y dependencias.
- `plan.md` — decisiones técnicas, arquitectura, módulos afectados y orden de implementación.
- `tasks.md` — lista de pasos secuenciales, accionables y verificables, marcables como completados (`[X]`).

El flujo obligatorio mínimo es:
`speckit.specify` → `speckit.plan` → `speckit.tasks` → `speckit.implement`.

Para specs fundacionales o de alto impacto se recomienda añadir `speckit.clarify` (entre specify y plan) y `speckit.analyze` (entre plan y tasks). Ninguna fase obligatoria puede saltarse. Cada tarea completada DEBE marcarse como `[X]` en `tasks.md`.

## Modo Interactivo de Preguntas

Los comandos `speckit.specify` y `speckit.clarify` operan en **modo interactivo obligatorio**: presentan sus preguntas de una en una y esperan respuesta antes de continuar. El comando `speckit.plan` opera en **modo interactivo condicional**: solo lanza preguntas si existen decisiones estructurales que afecten specs futuras y que no estén resueltas en la constitución ni en la spec vigente.

Cuando un comando opera en modo interactivo, DEBE seguir este protocolo sin excepción:

**Formato de pregunta con opciones:**
```
Pregunta [N de TOTAL] — [tema corto]
─────────────────────────────────────
[Enunciado claro de la pregunta]

Por qué importa: [1 línea sobre el impacto de decidir mal]

A) [opción concreta con valor específico]
B) [opción concreta con valor específico]  ← Recomendado
C) [opción concreta con valor específico]
D) Otro — escribe tu respuesta

> Responde con la letra (A, B, C o D) o escribe tu respuesta libre.
```

**Formato de pregunta Sí/No:**
```
Pregunta [N de TOTAL] — [tema corto]
─────────────────────────────────────
[Enunciado de la pregunta]

Por qué importa: [1 línea]

S) Sí  ← Recomendado
N) No

> Responde S o N.
```

**Reglas de las opciones:**
- Cada opción (A, B, C) DEBE ser concreta y ejecutable, nunca genérica.
- Las opciones DEBEN ser mutuamente excluyentes: cada una lleva a un resultado de código distinto.
- La opción marcada con `← Recomendado` DEBE ser la más idónea y alineada con la arquitectura del proyecto.
- La opción `D) Otro` SIEMPRE debe estar presente como vía de escape para respuesta personalizada.

**Reglas de respuesta:**
- Si el usuario responde con una letra (A, B, C, S o N): confirmar la elección en una línea con el valor concreto elegido y pasar inmediatamente a la siguiente pregunta.
- Si el usuario responde con texto libre o elige D): aceptar la respuesta, confirmarla en una línea y pasar a la siguiente pregunta.
- Al terminar todas las preguntas: mostrar un resumen de las decisiones tomadas y generar el artefacto correspondiente.

## Spec-Driven Development

- NO implementar nada que no esté descrito en un `spec.md` aprobado bajo `specs/`.
- El orden de implementación lo define el prefijo numérico de cada spec.
- Toda tarea implementada debe rastrearse en `tasks.md`.
- PROHIBIDO crear specs bajo `.specify/specs/`: esa carpeta se reserva exclusivamente para infraestructura.

## Spec activa

<!-- SPECKIT START -->
Plan vigente: `specs/`
<!-- SPECKIT END -->

Si el valor de plan vigente dice `specs/` significa que aún no se ha generado una spec activa. Este valor debe actualizarse al comenzar a trabajar en una feature para asegurar trazabilidad.

## Flujo Speckit — hooks obligatorios

Antes de ejecutar cualquier comando `speckit.*`, el agente DEBE:
1. Leer `.specify/memory/constitution.md` completo para aplicar las reglas vigentes.
2. Revisar `.specify/extensions.yml` (si existe) y ejecutar todos los hooks `before_<comando>` con `optional: false`.
3. Para `before_specify`, ejecutar el hook de creación de branch (`speckit.git.feature`) ANTES de crear el directorio de la spec o `spec.md`.

PROHIBIDO crear archivos bajo `specs/<nueva-spec>/` sin haber ejecutado previamente los hooks obligatorios correspondientes.

---

# Base de datos (SQLite Local con SQLx)

1. **Motor y Driver**: Se utiliza exclusivamente **SQLite** local a través del driver asíncrono `sqlx` (con `runtime-tokio`, `sqlite` y `macros`).
2. **Ubicación**: El archivo de base de datos es `comic.db`, ubicado en la raíz del proyecto.
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

Toda PR o revisión DEBE verificar el cumplimiento de los principios declarados antes de aprobarse. Cualquier violación DEBE justificarse explícitamente en la sección **Complexity Tracking** del `plan.md` de la iniciativa correspondiente; en ausencia de justificación, los cambios DEBEN rechazarse.

## Precedencia ante conflictos con upstream spec-kit

Cuando un script, hook o plantilla de spec-kit entre en conflicto con esta constitución sobre rutas físicas de artefactos, el comportamiento upstream tiene autoridad y la constitución debe alinearse formalmente. Está prohibido al agente mover archivos o parchear scripts silenciosamente.

## Protocolo del agente ante divergencias

Si un agente detecta divergencia entre un script de spec-kit y esta constitución:
1. **Detenerse inmediatamente** y NO ejecutar el comando divergente.
2. **Reportar al usuario** la naturaleza exacta del conflicto, citando las secciones involucradas.
3. **Esperar instrucción explícita** antes de continuar.

---

# Sistema de instrucciones

- Para todo trabajo frontend (Iced GUI y SPA Web), la definición canónica de tokens visuales y directrices vive en `.github/instructions/frontend.instructions.md`.
- Para todo trabajo backend (Rust, Tokio, Axum, Comic Reader), la definición canónica de directrices vive en `.github/instructions/backend.instructions.md`.
- Para todo trabajo con base de datos y persistencia, la definición canónica de directrices vive en `.github/instructions/database.instructions.md`.

No se aceptan cambios arbitrarios de arquitectura o diseño visual sin trazabilidad en la correspondiente spec y actualización de las instrucciones canónicas.

---

# Historial de versiones

| Número de versión | Descripción de cambios | Fecha |
|:---|:---|:---|
| v0.1.0 | Redacción y formalización de la constitución del proyecto Comic: finalidad del producto, estructura en `src/`, persistencia en SQLite local (WAL, SQLx), stack Rust/Iced/Axum y directrices de gobierno SDD. | 2026-09-15 |