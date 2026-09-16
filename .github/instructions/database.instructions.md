# Instrucciones Canónicas de Base de Datos y Persistencia

Este documento establece las directrices obligatorias para el diseño, modelado, consultas, migraciones y rendimiento de la capa de datos en el proyecto **Comic**.

---

## 1. Motor y Tecnologías de Persistencia

- **Motor de Base de Datos**: **SQLite 3** local embebido.
- **Ubicación del Archivo**: `comic.db` en el directorio de la aplicación (`app/comic.db`, ignorado en git).
- **Driver Asíncrono**: `sqlx` (v0.8) con soporte para `runtime-tokio`, `sqlite` y `macros`.
- **Pool de Conexiones**: Gestionado mediante `SqlitePool` con un límite estricto de **5 conexiones concurrentes** (`max_connections(5)`).
- **Modo de Operación**: Modo WAL obligatorio (`PRAGMA journal_mode=WAL;`), ejecutado inmediatamente tras establecer la conexión para permitir lecturas no bloqueantes durante escrituras.

---

## 2. Esquema Relacional de Datos

El modelo relacional consta de cuatro entidades principales gestionadas en `app/src/db.rs`:

### A. Tabla `collections`
Almacena las colecciones lógicas definidas por el usuario.
```sql
CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    icon_data BLOB NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### B. Tabla `collection_paths`
Almacena las rutas del sistema de archivos vinculadas a cada colección.
```sql
CREATE TABLE IF NOT EXISTS collection_paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
);
```

### C. Tabla `comics`
Almacena los volúmenes o cómics individuales indexados en el sistema.
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
Almacena los tokens persistentes asociados a dispositivos autorizados para acceso remoto permanente sin escanear QR en cada sesión.
```sql
CREATE TABLE IF NOT EXISTS trusted_devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token TEXT NOT NULL UNIQUE,
    device_name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 3. Reglas de Integridad y Diseño

1. **Borrado en Cascada Obligatorio (`ON DELETE CASCADE`)**:
   - Toda tabla hija (`collection_paths`, `comics`) DEBE declarar `FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE` para asegurar que al eliminar una colección se limpien automáticamente todas sus rutas y registros de cómics asociados.
2. **Caché en Columnas BLOB**:
   - La columna `cover_data` en `comics` y `icon_data` en `collections` almacenan la imagen binaria preprocesada (JPEG/PNG). Esto garantiza tiempos de respuesta instantáneos al mostrar cuadrículas y colecciones, eliminando la necesidad de leer y descomprimir archivos pesados en tiempo real.
3. **Criterios Canónicos de Ordenamiento**:
   - Las colecciones deben ordenarse alfabéticamente: `ORDER BY name`.
   - Los cómics dentro de una colección deben ordenarse dando prioridad a la saga y al número de entrega:
     ```sql
     ORDER BY COALESCE(saga, ''), COALESCE(issue_number, 0), title
     ```
   - Los dispositivos de confianza deben listarse en orden cronológico inverso: `ORDER BY created_at DESC`.

---

## 4. Reglas de Consultas y Seguridad

1. **PROHIBICIÓN de Inyecciones SQL y Concatenación Directa**:
   - Queda estrictamente PROHIBIDO interpolar variables o concatenar cadenas directamente en sentencias SQL.
   - Todas las consultas DEBEN utilizar parámetros posicionales (`?`) mediante el método `.bind()` de `sqlx`.
   - Ejemplo obligatorio:
     ```rust
     sqlx::query("SELECT id, name FROM collections WHERE id = ?")
         .bind(id)
         .fetch_optional(&self.pool)
         .await?;
     ```
2. **Operaciones de Upsert Controladas**:
   - Al persistir cómics (`upsert_comic`), si el `id > 0` se ejecuta un `UPDATE` de metadatos (`title`, `year`, `issue_number`, `saga`, `cover_data`, `page_count`). Si `id == 0`, se realiza un `INSERT` completo.

---

## 5. Ciclo de Vida y Migraciones del Esquema

1. **Idempotencia de Esquema**:
   - La inicialización del esquema en `Database::initialize_schema` debe ser totalmente idempotente. Toda creación de tablas debe usar `CREATE TABLE IF NOT EXISTS`.
2. **Migraciones Graduales**:
   - Cuando se añade una columna a una tabla existente (por ejemplo, `icon_data` en `collections`), debe ejecutarse una sentencia controlada (ej. `ALTER TABLE collections ADD COLUMN icon_data BLOB;`) capturando y descartando el error en caso de que la columna ya exista.
3. **Inicialización de Tablas Faltantes**:
   - Al iniciar la base de datos, DEBEN crearse todas las tablas requeridas por la aplicación, incluyendo `trusted_devices`.

---

## 6. Idioma y Documentación

- Toda la documentación, comentarios en código de base de datos y mensajes de error relacionados con la persistencia DEBEN estar escritos en español.
