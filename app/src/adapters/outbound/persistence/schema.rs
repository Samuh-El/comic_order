//! Inicialización y evolución del esquema de la base de datos SQLite.

use sqlx::SqlitePool;
use tracing::info;

/// Inicializa el esquema de tablas e índices en la base de datos de forma idempotente.
pub async fn initialize_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Configuración de concurrencia y restricciones
    sqlx::query("PRAGMA journal_mode = WAL;").execute(pool).await?;
    sqlx::query("PRAGMA foreign_keys = ON;").execute(pool).await?;

    // Tabla de colecciones
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS collections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            protagonist TEXT NULL,
            icon_data BLOB NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            description TEXT NULL,
            background_image_path TEXT NULL,
            hero_image_path TEXT NULL
        );"
    )
    .execute(pool)
    .await?;

    // Migraciones idempotentes para columnas extendidas
    let _ = sqlx::query("ALTER TABLE collections ADD COLUMN protagonist TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE collections ADD COLUMN icon_data BLOB")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE collections ADD COLUMN description TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE collections ADD COLUMN background_image_path TEXT")
        .execute(pool)
        .await;
    let _ = sqlx::query("ALTER TABLE collections ADD COLUMN hero_image_path TEXT")
        .execute(pool)
        .await;

    // Tabla de carpetas físicas asociadas a colecciones
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS collection_paths (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            collection_id INTEGER NOT NULL,
            path TEXT NOT NULL UNIQUE,
            FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
        );"
    )
    .execute(pool)
    .await?;

    // Tabla de volúmenes de cómics
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS comics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            collection_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            file_path TEXT NOT NULL UNIQUE,
            file_type TEXT NOT NULL,
            year INTEGER NULL,
            issue_number INTEGER NULL,
            saga TEXT NULL,
            cover_data BLOB NULL,
            page_count INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
        );"
    )
    .execute(pool)
    .await?;

    // Índices de optimización de consulta
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_comics_collection ON comics(collection_id);"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_comics_order ON comics(collection_id, saga, issue_number, title);"
    )
    .execute(pool)
    .await?;

    // Tabla de dispositivos autorizados de confianza
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS trusted_devices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token TEXT NOT NULL UNIQUE,
            device_name TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    )
    .execute(pool)
    .await?;

    // Tabla de progreso de lectura por dispositivo
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS reading_progress (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            comic_id INTEGER NOT NULL,
            device_id TEXT NOT NULL,
            current_page INTEGER NOT NULL DEFAULT 0,
            completed BOOLEAN NOT NULL DEFAULT 0,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (comic_id) REFERENCES comics(id) ON DELETE CASCADE,
            UNIQUE(comic_id, device_id)
        );"
    )
    .execute(pool)
    .await?;

    info!("[DB] Esquema e índices SQLite inicializados correctamente.");
    Ok(())
}
