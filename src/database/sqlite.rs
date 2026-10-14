use log;
use rusqlite;

use crate::database::{Database, Error};
use crate::migration::Migration;

pub struct SqliteDatabase<'a> {
    pub connection: &'a mut rusqlite::Connection,
}

impl<'a> SqliteDatabase<'a> {
    pub fn new(connection: &'a mut rusqlite::Connection) -> Self {
        Self { connection }
    }
}

impl<'a> Database for SqliteDatabase<'a> {
    fn create_migration_table(&mut self) -> Result<(), Error> {
        self.connection.execute(
            r#"CREATE TABLE litemigration(
id INTEGER PRIMARY KEY,
version INTEGER NOT NULL,
name VARCHAR NOT NULL


)"#,
            (),
        )?;
        log::info!("migration table created");
        Ok(())
    }
    fn latest_migration(&mut self) -> Result<i32, Error> {
        let max_version: i32 = self.connection.query_row(
            r#"SELECT COALESCE(MAX(version), 0) FROM litemigration"#,
            (),
            |row| row.get(0),
        )?;

        Ok(max_version as i32)
    }
    fn apply_migrations(&mut self, migrations: &[Migration]) -> Result<(), Error> {
        for m in migrations {
            let max_version = self.latest_migration()?;
            if m.version > max_version {
                let tx = self.connection.transaction()?;
                tx.execute(&m.sql, ()).map_err(|e| Error::MigrationError {
                    version: m.version,
                    source: Box::new(e),
                })?;
                tx.execute(
                    r#"INSERT INTO litemigration(version, name) VALUES(?,?)"#,
                    (&m.version, &m.name),
                )?;
                tx.commit()?;
                log::info!("migration: {} applied", &m.version);
            } else {
                log::warn!("Migration: {} skipped", &m.version);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SqliteDatabase;
    use crate::{
        database::{Database, Error},
        migration::Migration,
    };
    use std::fs;
    use tempfile;

    #[test]
    fn test_create_migration() -> Result<(), Error> {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut connection = rusqlite::Connection::open(&db_path)?;

        let mut sqlite = SqliteDatabase::new(&mut connection);
        sqlite.create_migration_table()?;

        let _ = connection.close();
        let _ = fs::remove_file(db_path);
        Ok(())
    }

    #[test]
    fn test_apply_migration() -> Result<(), Error> {
        let m_1 = Migration::new(1, "create users", "CREATE TABLE users(name VARCHAR)");
        let m_2 = Migration::new(
            2,
            "insert user",
            r#"INSERT into users(name) VALUES("test")"#,
        );
        let migrations = vec![m_1, m_2];

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut connection = rusqlite::Connection::open(&db_path)?;

        let mut sqlite = SqliteDatabase::new(&mut connection);

        sqlite.create_migration_table()?;
        sqlite.apply_migrations(&migrations)?;

        let max_version = sqlite.latest_migration()?;
        assert_eq!(max_version, 2);

        let _ = connection.close();

        let _ = fs::remove_file(db_path);

        Ok(())
    }
    #[test]
    fn test_fail_apply_migration() -> Result<(), Error> {
        let m_1 = Migration::new(1, "create users", "CREATE TABLE users(name VARCHAR)");
        let m_2 = Migration::new(
            2,
            "insert user",
            r#"INSERT into userssss(name) VALUES("test")"#,
        );
        let migrations = vec![m_1, m_2];

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut connection = rusqlite::Connection::open(&db_path)?;

        let mut sqlite = SqliteDatabase::new(&mut connection);

        sqlite.create_migration_table()?;
        let result = sqlite.apply_migrations(&migrations);
        match result {
            Err(Error::MigrationError { version, source: _ }) => {
                assert_eq!(version, 2);
            }
            _ => panic!("expected migrtions 2"),
        }

        let max_version = sqlite.latest_migration()?;
        assert_eq!(max_version, 1);

        let _ = connection.close();

        let _ = fs::remove_file(db_path);

        Ok(())
    }
}
