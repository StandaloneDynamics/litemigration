use thiserror::Error;

mod schema;

//#[cfg(feature = "postgres")]
pub mod postgres;

//#[cfg(feature = "sqlite")]
mod sqlite;


#[derive(Error, Debug)]
pub enum LiteMigrationError{
    #[error("Unable to create postgres database")]
    PostgresError,
    #[error("unable to create sqlite database")]
    SqiliteError,
    #[error("Unable to migrate schema")]
    MigrationError,

}

trait Migration{
    async fn create_table(&mut self) -> Result<(), LiteMigrationError>;
    async fn exists(&mut self) -> bool;
    async fn add_schema(&mut self, changes: &Vec<schema::Schema>) -> Result<(), LiteMigrationError>;
    async fn latest_migration(&mut self) -> Result<i32, LiteMigrationError>;
}