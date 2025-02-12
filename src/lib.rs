use thiserror::Error;

pub mod schema;

//#[cfg(feature = "postgres")]
pub mod postgres;

//#[cfg(feature = "sqlite")]
pub mod sqlite;


#[derive(Error, Debug)]
pub enum LiteMigrationError{
    #[error("Unable to create migration table")]
    DatabaseError(sqlx::Error),
    #[error("Unable to add migration {value:?}: {error}")]
    MigrationError{
        value: i32,
        error: sqlx::Error},

}

pub trait Migration{
    async fn create_table(&mut self) -> Result<(), LiteMigrationError>;
    async fn exists(&mut self) -> bool;
    async fn add_schema(&mut self, changes: &Vec<schema::Schema>) -> Result<(), LiteMigrationError>;
    async fn latest_migration(&mut self) -> Result<i32, LiteMigrationError>;
}