use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CollectionPath {
    pub id: i64,
    pub collection_id: i64,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Comic {
    pub id: i64,
    pub collection_id: i64,
    pub title: String,
    pub file_path: String,
    pub file_type: String,
    pub year: Option<i32>,
    pub issue_number: Option<i32>,
    pub saga: Option<String>,
    pub cover_data: Option<Vec<u8>>,
    pub page_count: i32,
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;
        
        info!("[DB] Conectado exitosamente a: {}", url);
        if let Ok(path) = std::env::current_dir() {
            info!("[DB] Directorio de trabajo actual: {:?}", path);
        }

        let db = Self { pool };
        sqlx::query("PRAGMA journal_mode=WAL;").execute(&db.pool).await?;
        db.initialize_schema().await?;
        Ok(db)
    }

    async fn initialize_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS collection_paths (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                collection_id INTEGER NOT NULL,
                path TEXT NOT NULL,
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS comics (
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
            )"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // === Collections ===

    pub async fn get_collections(&self) -> Result<Vec<Collection>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, name FROM collections ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .iter()
            .map(|row| Collection {
                id: row.get("id"),
                name: row.get("name"),
            })
            .collect())
    }

    pub async fn create_collection(&self, name: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO collections (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid() as i64)
    }

    pub async fn delete_collection(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM collections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn rename_collection(&self, id: i64, new_name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE collections SET name = ? WHERE id = ?")
            .bind(new_name)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // === Collection Paths ===

    pub async fn get_collection_paths(&self, collection_id: i64) -> Result<Vec<CollectionPath>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, collection_id, path FROM collection_paths WHERE collection_id = ?")
            .bind(collection_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .iter()
            .map(|row| CollectionPath {
                id: row.get("id"),
                collection_id: row.get("collection_id"),
                path: row.get("path"),
            })
            .collect())
    }

    pub async fn add_collection_path(&self, collection_id: i64, path: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO collection_paths (collection_id, path) VALUES (?, ?)")
            .bind(collection_id)
            .bind(path)
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid() as i64)
    }

    // === Comics ===

    pub async fn get_comics_by_collection(&self, collection_id: i64) -> Result<Vec<Comic>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, collection_id, title, file_path, file_type, year, issue_number, saga, cover_data, page_count 
             FROM comics WHERE collection_id = ? ORDER BY COALESCE(saga, ''), COALESCE(issue_number, 0), title"
        )
        .bind(collection_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| Comic {
                id: row.get("id"),
                collection_id: row.get("collection_id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                file_type: row.get("file_type"),
                year: row.get("year"),
                issue_number: row.get("issue_number"),
                saga: row.get("saga"),
                cover_data: row.get("cover_data"),
                page_count: row.get("page_count"),
            })
            .collect())
    }

    pub async fn upsert_comic(&self, comic: &Comic) -> Result<i64, sqlx::Error> {
        if comic.id > 0 {
            sqlx::query(
                "UPDATE comics SET title=?, year=?, issue_number=?, saga=?, cover_data=?, page_count=? WHERE id=?"
            )
            .bind(&comic.title)
            .bind(comic.year)
            .bind(comic.issue_number)
            .bind(&comic.saga)
            .bind(&comic.cover_data)
            .bind(comic.page_count)
            .bind(comic.id)
            .execute(&self.pool)
            .await?;
            Ok(comic.id)
        } else {
            let result = sqlx::query(
                "INSERT INTO comics (collection_id, title, file_path, file_type, year, issue_number, saga, cover_data, page_count) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(comic.collection_id)
            .bind(&comic.title)
            .bind(&comic.file_path)
            .bind(&comic.file_type)
            .bind(comic.year)
            .bind(comic.issue_number)
            .bind(&comic.saga)
            .bind(&comic.cover_data)
            .bind(comic.page_count)
            .execute(&self.pool)
            .await?;
            Ok(result.last_insert_rowid() as i64)
        }
    }

    pub async fn comic_exists_by_path(&self, file_path: &str) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM comics WHERE file_path = ?")
            .bind(file_path)
            .fetch_one(&self.pool)
            .await?;
        let count: i64 = row.get("cnt");
        Ok(count > 0)
    }

    pub async fn get_comic_by_id(&self, id: i64) -> Result<Option<Comic>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, collection_id, title, file_path, file_type, year, issue_number, saga, cover_data, page_count 
             FROM comics WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Comic {
            id: row.get("id"),
            collection_id: row.get("collection_id"),
            title: row.get("title"),
            file_path: row.get("file_path"),
            file_type: row.get("file_type"),
            year: row.get("year"),
            issue_number: row.get("issue_number"),
            saga: row.get("saga"),
            cover_data: row.get("cover_data"),
            page_count: row.get("page_count"),
        }))
    }

    pub async fn delete_comic(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM comics WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
