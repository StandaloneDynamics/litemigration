mod database;
mod error;
mod migration;

use database::Database;

pub use error::Error;
pub use migration::Migration;

#[cfg(feature = "sqlite")]
pub use database::sqlite::SqliteDatabase;

#[cfg(feature = "postgres")]
pub use database::pg::PostgresDatabase;

/// Run database migrations against a database backend

/// Struct that is generic over [`Database`] which allows
/// for any database engine to work with it
pub struct Migrator<D>
where
    D: Database,
{
    database: D,
}

impl<D> Migrator<D>
where
    D: Database,
{
    /// Create a new migrator instance containing database connection
    pub fn new(database: D) -> Self {
        Self { database }
    }
    /// Applies pending migrations or
    /// creates migration table and applies all migrations.
    pub fn migrate(&mut self, migrations: &[Migration]) -> Result<(), Error> {
        self.database.create_migration_table()?;
        self.database.apply_migrations(migrations)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::database::sqlite::SqliteDatabase;
    use super::error::Error;
    use super::migration::Migration;
    use super::Migrator;
    use std::fs;
    use tempfile;

    #[test]
    fn test_sqlite() -> Result<(), Error> {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut connection = rusqlite::Connection::open(&db_path)?;

        let m_1 = Migration::new(1, "create users", "CREATE TABLE users(name VARCHAR)");
        let m_2 = Migration::new(
            2,
            "insert user",
            r#"INSERT into users(name) VALUES("test")"#,
        );
        let migrations = vec![m_1, m_2];

        let sqlite = SqliteDatabase::new(&mut connection);
        let mut migrator = Migrator::new(sqlite);
        migrator.migrate(&migrations)?;

        let _ = connection.close();
        let _ = fs::remove_file(db_path);

        Ok(())
    }
}
