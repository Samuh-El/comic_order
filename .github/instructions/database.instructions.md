# Instrucciones Canónicas de Base de Datos y Persistencia (Adaptador Hexagonal)

Este documento establece las directrices obligatorias para el diseño, modelado, consultas, migraciones y rendimiento de la capa de datos en el proyecto **Comic** bajo la Arquitectura Hexagonal.

---

## 1. Rol en la Arquitectura Hexagonal

La capa de base de datos opera como un **Adaptador de Salida Secundario (*Secondary / Outbound Adapter*)** ubicado en `app/src/adapters/outbound/persistence/`.

- **Desacoplamiento**: Implementa los puertos definidos por el dominio (`ComicRepository`, `CollectionRepository`, `TrustedDeviceRepository`, `ProgressRepository`).
- **Aislamiento**: Las entidades del dominio no contienen anotaciones de base de datos ni dependencias de `sqlx`; el adaptador mapea entre los modelos relacionales de la BD y las entidades del dominio.

---

## 2. Motor y Configuración de Persistencia

- **Motor**: **SQLite 3** local embebido.
- **Ubicación del Archivo**: `app/comic.db` en el directorio de la aplicación (excluido de git).
- **Driver Asíncrono**: `sqlx` (v0.8) sobre runtime Tokio.
- **Pool de Conexiones**: `SqlitePool` con un límite de **5 conexiones concurrentes** (`max_connections(5)`).
- **Modo WAL Obligatorio**: Ejecutar `PRAGMA journal_mode=WAL;` inmediatamente tras conectar para permitir lecturas masivas paralelas durante la ingesta de archivos.

---

## 3. Esquema Relacional Canónico

### A. Tabla `collections`
```sql
CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    icon_data BLOB NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### B. Tabla `collection_paths`
```sql
CREATE TABLE IF NOT EXISTS collection_paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
);
```

### C. Tabla `comics`
```sql
CREATE TABLE IF NOT EXISTS comics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_type TEXT NOT NULL,
    year INTEGER NULL,
    issue_number INTEGER NULL,
    saga TEXT NULL,
    cover_data BLOB NULL,
    page_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
);
```

### D. Tabla `trusted_devices`
```sql
CREATE TABLE IF NOT EXISTS trusted_devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token TEXT NOT NULL UNIQUE,
    device_name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### E. Tabla `reading_progress` (Progreso de Lectura)
```sql
CREATE TABLE IF NOT EXISTS reading_progress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    comic_id INTEGER NOT NULL,
    device_id TEXT NOT NULL,
    current_page INTEGER NOT NULL DEFAULT 0,
    completed BOOLEAN NOT NULL DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (comic_id) REFERENCES comics(id) ON DELETE CASCADE,
    UNIQUE(comic_id, device_id)
);
```

---

## 4. Reglas de Integridad y Consultas

1. **Eliminación en Cascada (`ON DELETE CASCADE`)**:
   - Todas las tablas hijas (`collection_paths`, `comics`, `reading_progress`) deben mantener claves foráneas con eliminación en cascada.
2. **Consultas Parametrizadas Estrictas**:
   - Prohibido concatenar strings SQL. Toda variable DEBE vincularse mediante `.bind()` de `sqlx`.
3. **Caché en Columnas BLOB**:
   - `cover_data` e `icon_data` almacenan miniaturas binarias preprocesadas (JPEG 300x450) para alimentar al proxy de caché sin lecturas de disco adicionales.
4. **Ordenamiento Canónico**:
   - Cómics: `ORDER BY COALESCE(saga, ''), COALESCE(issue_number, 0), title`.
   - Colecciones: `ORDER BY name ASC`.

---

## 5. Migraciones Idempotentes

- Todas las tablas e índices deben crearse con `CREATE TABLE IF NOT EXISTS` y `CREATE INDEX IF NOT EXISTS` en el arranque del repositorio.
- Las migraciones graduales de columnas se efectúan mediante sentencias controladas de tipo `ALTER TABLE ... ADD COLUMN` capturando el error si la columna ya existe.
