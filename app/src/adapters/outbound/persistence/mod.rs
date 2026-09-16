//! Adaptador de persistencia relacional SQLite.

pub mod schema;
pub mod sqlite_repo;

pub use schema::initialize_schema;
pub use sqlite_repo::SqliteComicRepository;
