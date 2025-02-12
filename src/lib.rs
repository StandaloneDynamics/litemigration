/*!
This crate allows for the creation of embedded migrations.
Provides supoort for Sqlite and Posgresql.
Only forward migrations are performed.
# Usage
```
[dependencies]
litemigrations = {version = "1", features=["postgres", "sqlite"]}
```


# Example
```rust
let path = "sqlite://litemigration.db";
Sqlite::create_database(&path).await.unwrap();
let mut conn = SqliteConnection::connect(path).await.unwrap();


// Create a struct that which contains an existing db connection
let mut lite = LiteSqlite{conn: &mut conn};

// Create a migration table to keep track of migrations
lite.create_table().await;

// Add the database changes as a list of schemas
let changes = vec![
    Schema{id: 2, query: "CREATE TABLE cats(id INTEGER PRIMARY KEY AUTOINCREMENT, name VARCHAR)"},
    Schema{id: 3, query: "INSERT INTO cats(name) VALUES('John')"},
    Schema{id: 4, query: "INSERT INTO cats(name) VALUES('Peter')"},
];

// This loops through each of the changes and applies them.
lite.add_schema(&changes).await;
```
 */


use thiserror::Error;

/// Struct containing a database migration and its version
pub mod schema;

/// Module supporting Postgres migrations
#[cfg(feature = "postgres")]
pub mod postgres;

/// Module supporting Sqlite migrations
#[cfg(feature = "sqlite")]
pub mod sqlite;


/// Enum of possible error messages
#[derive(Error, Debug)]
pub enum LiteMigrationError{
    #[error("Unable to create migration table")]
    DatabaseError(sqlx::Error),
    #[error("Unable to add migration {value:?}: {error}")]
    MigrationError{
        value: i32,
        error: sqlx::Error
    },

}

/// Trait implementing methods to perform migrations.
pub trait Migration{
    /// Create initial migration table to keep track of migrations
    async fn create_table(&mut self) -> Result<(), LiteMigrationError>;

    /// Check of the database and migrations table exists
    async fn exists(&mut self) -> bool;
    
    /// apply a schema list of migrations
    async fn add_schema(&mut self, changes: &Vec<schema::Schema>) -> Result<(), LiteMigrationError>;
    
    /// Obtain the latest migration version
    async fn latest_migration(&mut self) -> Result<i32, LiteMigrationError>;
}