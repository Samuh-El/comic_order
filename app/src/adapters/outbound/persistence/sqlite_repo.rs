//! Implementación de repositorios de persistencia sobre SQLite utilizando SQLx.

use std::path::PathBuf;
use std::str::FromStr;
use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use chrono::{DateTime, Utc};
use crate::domain::comic::{Comic, ComicFormat};
use crate::domain::collection::{Collection, CollectionPath};
use crate::domain::device::TrustedDevice;
use crate::domain::progress::ReadingProgress;
use crate::domain::errors::RepositoryError;
use crate::domain::ports::{
    ComicRepository,
    CollectionRepository,
    DeviceRepository,
    ProgressRepository,
};

/// Repositorio unificado sobre SQLite para el catálogo de cómics y colecciones.
#[derive(Debug, Clone)]
pub struct SqliteComicRepository {
    pool: SqlitePool,
}

impl SqliteComicRepository {
    /// Crea una nueva instancia del repositorio a partir de un pool de conexiones existente.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Retorna una referencia al pool de conexiones subyacente.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl ComicRepository for SqliteComicRepository {
    async fn get_by_id(&self, id: i64) -> Result<Option<Comic>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, collection_id, title, file_path, file_type, year, issue_number, saga, cover_data, page_count, created_at, updated_at
             FROM comics WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| {
            let file_type_str: String = r.get("file_type");
            let format = ComicFormat::from_extension(&file_type_str).unwrap_or(ComicFormat::Cbz);
            let created_at_str: Option<String> = r.try_get("created_at").ok();
            let updated_at_str: Option<String> = r.try_get("updated_at").ok();
            
            let created_at = created_at_str
                .and_then(|s| DateTime::from_str(&s).ok())
                .unwrap_or_else(Utc::now);
            let updated_at = updated_at_str
                .and_then(|s| DateTime::from_str(&s).ok())
                .unwrap_or_else(Utc::now);

            Comic {
                id: r.get("id"),
                collection_id: r.get("collection_id"),
                title: r.get("title"),
                file_path: PathBuf::from(r.get::<String, _>("file_path")),
                format,
                year: r.get("year"),
                issue_number: r.get("issue_number"),
                saga: r.get("saga"),
                cover_data: r.get("cover_data"),
                page_count: r.get::<i32, _>("page_count") as usize,
                created_at,
                updated_at,
            }
        }))
    }

    async fn get_by_collection(&self, collection_id: i64) -> Result<Vec<Comic>, RepositoryError> {
        let rows = sqlx::query(
            "SELECT id, collection_id, title, file_path, file_type, year, issue_number, saga, cover_data, page_count, created_at, updated_at
             FROM comics WHERE collection_id = ?
             ORDER BY title COLLATE NOCASE ASC, COALESCE(issue_number, 0) ASC"
        )
        .bind(collection_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let file_type_str: String = r.get("file_type");
                let format = ComicFormat::from_extension(&file_type_str).unwrap_or(ComicFormat::Cbz);
                Comic {
                    id: r.get("id"),
                    collection_id: r.get("collection_id"),
                    title: r.get("title"),
                    file_path: PathBuf::from(r.get::<String, _>("file_path")),
                    format,
                    year: r.get("year"),
                    issue_number: r.get("issue_number"),
                    saga: r.get("saga"),
                    cover_data: r.get("cover_data"),
                    page_count: r.get::<i32, _>("page_count") as usize,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            })
            .collect())
    }

    async fn exists_by_path(&self, file_path: &str) -> Result<bool, RepositoryError> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM comics WHERE file_path = ?")
            .bind(file_path)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let count: i64 = row.get("cnt");
        Ok(count > 0)
    }

    async fn upsert(&self, comic: &Comic) -> Result<i64, RepositoryError> {
        let file_path_str = comic.file_path.to_string_lossy().to_string();
        let file_type_str = comic.format.as_str().to_string();
        let page_count_i32 = comic.page_count as i32;

        if comic.id > 0 {
            sqlx::query(
                "UPDATE comics SET title = ?, year = ?, issue_number = ?, saga = ?, cover_data = ?, page_count = ?, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?"
            )
            .bind(&comic.title)
            .bind(comic.year)
            .bind(comic.issue_number)
            .bind(&comic.saga)
            .bind(&comic.cover_data)
            .bind(page_count_i32)
            .bind(comic.id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

            Ok(comic.id)
        } else {
            let result = sqlx::query(
                "INSERT INTO comics (collection_id, title, file_path, file_type, year, issue_number, saga, cover_data, page_count)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(comic.collection_id)
            .bind(&comic.title)
            .bind(&file_path_str)
            .bind(&file_type_str)
            .bind(comic.year)
            .bind(comic.issue_number)
            .bind(&comic.saga)
            .bind(&comic.cover_data)
            .bind(page_count_i32)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

            Ok(result.last_insert_rowid() as i64)
        }
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM comics WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_metadata(
        &self,
        id: i64,
        title: &str,
        year: Option<i32>,
        issue_number: Option<i32>,
        saga: Option<&str>,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE comics SET title = ?, year = ?, issue_number = ?, saga = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?"
        )
        .bind(title)
        .bind(year)
        .bind(issue_number)
        .bind(saga)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl CollectionRepository for SqliteComicRepository {
    async fn get_all(&self) -> Result<Vec<Collection>, RepositoryError> {
        let rows = sqlx::query("SELECT id, name, protagonist, icon_data, created_at, description, background_image_path, hero_image_path FROM collections ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Collection {
                id: r.get("id"),
                name: r.get("name"),
                protagonist: r.get("protagonist"),
                description: r.get("description"),
                background_image_path: r.get("background_image_path"),
                hero_image_path: r.get("hero_image_path"),
                icon_data: r.get("icon_data"),
                created_at: Utc::now(),
            })
            .collect())
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Collection>, RepositoryError> {
        let row = sqlx::query("SELECT id, name, protagonist, icon_data, created_at, description, background_image_path, hero_image_path FROM collections WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Collection {
            id: r.get("id"),
            name: r.get("name"),
            protagonist: r.get("protagonist"),
            description: r.get("description"),
            background_image_path: r.get("background_image_path"),
            hero_image_path: r.get("hero_image_path"),
            icon_data: r.get("icon_data"),
            created_at: Utc::now(),
        }))
    }

    async fn get_recent(&self, limit: usize) -> Result<Vec<Collection>, RepositoryError> {
        let rows = sqlx::query(
            "SELECT id, name, protagonist, icon_data, created_at, description, background_image_path, hero_image_path 
             FROM collections 
             ORDER BY datetime(created_at) DESC, id DESC 
             LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Collection {
                id: r.get("id"),
                name: r.get("name"),
                protagonist: r.get("protagonist"),
                description: r.get("description"),
                background_image_path: r.get("background_image_path"),
                hero_image_path: r.get("hero_image_path"),
                icon_data: r.get("icon_data"),
                created_at: Utc::now(),
            })
            .collect())
    }

    async fn create(&self, name: &str) -> Result<i64, RepositoryError> {
        let result = sqlx::query("INSERT INTO collections (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(result.last_insert_rowid() as i64)
    }

    async fn create_with_details(
        &self,
        name: &str,
        protagonist: Option<&str>,
        description: Option<&str>,
        background_image_path: Option<&str>,
        hero_image_path: Option<&str>,
    ) -> Result<i64, RepositoryError> {
        let result = sqlx::query(
            "INSERT INTO collections (name, protagonist, description, background_image_path, hero_image_path) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(name)
        .bind(protagonist)
        .bind(description)
        .bind(background_image_path)
        .bind(hero_image_path)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(result.last_insert_rowid() as i64)
    }

    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE collections SET name = ?, protagonist = ?, icon_data = ?, description = ?, background_image_path = ?, hero_image_path = ? WHERE id = ?"
        )
        .bind(&collection.name)
        .bind(&collection.protagonist)
        .bind(&collection.icon_data)
        .bind(&collection.description)
        .bind(&collection.background_image_path)
        .bind(&collection.hero_image_path)
        .bind(collection.id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM collections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_paths(&self, collection_id: i64) -> Result<Vec<CollectionPath>, RepositoryError> {
        let rows = sqlx::query("SELECT id, collection_id, path FROM collection_paths WHERE collection_id = ?")
            .bind(collection_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| CollectionPath {
                id: r.get("id"),
                collection_id: r.get("collection_id"),
                path: PathBuf::from(r.get::<String, _>("path")),
            })
            .collect())
    }

    async fn add_path(&self, collection_id: i64, path: &str) -> Result<i64, RepositoryError> {
        let result = sqlx::query("INSERT INTO collection_paths (collection_id, path) VALUES (?, ?) ON CONFLICT(path) DO UPDATE SET collection_id = excluded.collection_id")
            .bind(collection_id)
            .bind(path)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(result.last_insert_rowid() as i64)
    }

    async fn remove_path(&self, path_id: i64) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM collection_paths WHERE id = ?")
            .bind(path_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn set_icon(&self, collection_id: i64, icon_data: &[u8]) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE collections SET icon_data = ? WHERE id = ?")
            .bind(icon_data)
            .bind(collection_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl DeviceRepository for SqliteComicRepository {
    async fn get_all_trusted_devices(&self) -> Result<Vec<TrustedDevice>, RepositoryError> {
        let rows = sqlx::query("SELECT id, token, device_name, created_at FROM trusted_devices ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| TrustedDevice {
                id: r.get("id"),
                token: r.get("token"),
                device_name: r.get("device_name"),
                created_at: Utc::now(),
            })
            .collect())
    }

    async fn create_trusted_device(&self, device_name: &str) -> Result<TrustedDevice, RepositoryError> {
        let token = TrustedDevice::generate_token();
        let result = sqlx::query("INSERT INTO trusted_devices (token, device_name) VALUES (?, ?)")
            .bind(&token)
            .bind(device_name)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let id = result.last_insert_rowid() as i64;
        Ok(TrustedDevice {
            id,
            token,
            device_name: device_name.to_string(),
            created_at: Utc::now(),
        })
    }

    async fn validate_token(&self, token: &str) -> Result<bool, RepositoryError> {
        let row = sqlx::query("SELECT COUNT(*) as cnt FROM trusted_devices WHERE token = ?")
            .bind(token)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let count: i64 = row.get("cnt");
        Ok(count > 0)
    }

    async fn delete_trusted_device(&self, id: i64) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM trusted_devices WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl ProgressRepository for SqliteComicRepository {
    async fn get_progress(&self, comic_id: i64, device_id: &str) -> Result<Option<ReadingProgress>, RepositoryError> {
        let row = sqlx::query(
            "SELECT comic_id, device_id, current_page, completed, updated_at
             FROM reading_progress WHERE comic_id = ? AND device_id = ?"
        )
        .bind(comic_id)
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| ReadingProgress {
            comic_id: r.get("comic_id"),
            device_id: r.get("device_id"),
            current_page: r.get::<i32, _>("current_page") as usize,
            completed: r.get("completed"),
            updated_at: Utc::now(),
        }))
    }

    async fn save_progress(&self, progress: &ReadingProgress) -> Result<(), RepositoryError> {
        let page_i32 = progress.current_page as i32;
        sqlx::query(
            "INSERT INTO reading_progress (comic_id, device_id, current_page, completed, updated_at)
             VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)
             ON CONFLICT(comic_id, device_id) DO UPDATE SET
                current_page = excluded.current_page,
                completed = excluded.completed,
                updated_at = CURRENT_TIMESTAMP"
        )
        .bind(progress.comic_id)
        .bind(&progress.device_id)
        .bind(page_i32)
        .bind(progress.completed)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::outbound::persistence::schema::initialize_schema;

    #[tokio::test]
    async fn test_create_and_get_recent_collections() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Pool en memoria");

        initialize_schema(&pool).await.expect("Schema inicializado");

        let repo = SqliteComicRepository::new(pool);

        let id1 = repo
            .create_with_details("Coleccion Alfa", Some("ALFA"), Some("Sinopsis Alfa"), Some("bg_alfa.png"), Some("hero_alfa.png"))
            .await
            .expect("Creado 1");

        let id2 = repo
            .create_with_details("Coleccion Beta", Some("BETA"), Some("Sinopsis Beta"), None, None)
            .await
            .expect("Creado 2");

        let recent = repo.get_recent(5).await.expect("Recientes");
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, id2); // Más reciente primero
        assert_eq!(recent[0].name, "Coleccion Beta");
        assert_eq!(recent[0].protagonist.as_deref(), Some("BETA"));
        assert_eq!(recent[1].id, id1);
        assert_eq!(recent[1].name, "Coleccion Alfa");
        assert_eq!(recent[1].protagonist.as_deref(), Some("ALFA"));
        assert_eq!(recent[1].background_image_path.as_deref(), Some("bg_alfa.png"));
        assert_eq!(recent[1].hero_image_path.as_deref(), Some("hero_alfa.png"));

        // Probar update
        let mut col = recent[0].clone();
        col.protagonist = Some("BETA-HERO".to_string());
        col.description = Some("Sinopsis Beta Actualizada".to_string());
        col.background_image_path = Some("bg_beta.png".to_string());
        repo.update(&col).await.expect("Update exitoso");

        let updated = crate::domain::ports::CollectionRepository::get_by_id(&repo, id2)
            .await
            .expect("Obtenido")
            .expect("Existe");
        assert_eq!(updated.protagonist.as_deref(), Some("BETA-HERO"));
        assert_eq!(updated.description.as_deref(), Some("Sinopsis Beta Actualizada"));
        assert_eq!(updated.description.as_deref(), Some("Sinopsis Beta Actualizada"));
        assert_eq!(updated.background_image_path.as_deref(), Some("bg_beta.png"));
    }
}

