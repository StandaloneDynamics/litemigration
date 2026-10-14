use crate::database::{Database, Error};
use crate::migration::Migration;
use log;
use postgres;

pub struct PostgresDatabase<'a> {
    client: &'a mut postgres::Client,
}

impl<'a> PostgresDatabase<'a> {
    pub fn new(client: &'a mut postgres::Client) -> Self {
        Self { client }
    }
}

impl<'a> Database for PostgresDatabase<'a> {
    fn create_migration_table(&mut self) -> Result<(), Error> {
        self.client.execute(
            r#"CREATE TABLE litemigration(
id SERIAL PRIMARY KEY,
version INTEGER NOT NULL,
name VARCHAR NOT NULL

)"#,
            &[],
        )?;
        log::info!("migration table created");
        Ok(())
    }
    fn latest_migration(&mut self) -> Result<i32, Error> {
        let row = self.client.query_one(
            r#"SELECT COALESCE(MAX(version), 0) FROM litemigration"#,
            &[],
        )?;
        let max_version = row.get("coalesce");

        Ok(max_version)
    }

    fn apply_migrations(&mut self, migrations: &[Migration]) -> Result<(), Error> {
        for m in migrations {
            let max_version = self.latest_migration()?;
            if m.version > max_version {
                let mut tx = self.client.transaction()?;
                tx.execute(m.sql, &[]).map_err(|e| Error::MigrationError {
                    version: m.version,
                    source: Box::new(e),
                })?;
                tx.execute(
                    r#"INSERT INTO litemigration(version, name) VALUES($1,$2)"#,
                    &[&m.version, &m.name],
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
    use super::PostgresDatabase;
    use crate::{
        database::{Database, Error},
        migration::Migration,
    };
    use postgres::{Client, NoTls};
    use testcontainers_modules::{postgres as test_pg, testcontainers::runners::SyncRunner};

    #[test]
    fn test_create_migration() -> Result<(), Error> {
        let container = test_pg::Postgres::default()
            .start()
            .expect("Failed to start postgres container");
        let host_port = container
            .get_host_port_ipv4(5432)
            .expect("Failed to get postgres port");

        let database_url = format!(
            "host=127.0.0.1 port={host_port} user=postgres password=postgres dbname=postgres"
        );
        let mut client = Client::connect(&database_url, NoTls)?;
        let mut db = PostgresDatabase::new(&mut client);
        db.create_migration_table()?;

        let _ = client.close();

        Ok(())
    }

    #[test]
    fn test_apply_migration() -> Result<(), Error> {
        let container = test_pg::Postgres::default()
            .start()
            .expect("Failed to start postgres container");
        let host_port = container
            .get_host_port_ipv4(5432)
            .expect("Failed to get postgres port");

        let database_url = format!(
            "host=127.0.0.1 port={host_port} user=postgres password=postgres dbname=postgres"
        );
        let mut client = Client::connect(&database_url, NoTls)?;
        let mut db = PostgresDatabase::new(&mut client);

        let m_1 = Migration::new(1, "create users", "CREATE TABLE users(name TEXT)");
        let m_2 = Migration::new(
            2,
            "insert user",
            r#"INSERT into users(name) VALUES('test')"#,
        );
        let migrations = vec![m_1, m_2];

        db.create_migration_table()?;
        db.apply_migrations(&migrations)?;

        let max_version = db.latest_migration()?;
        assert_eq!(max_version, 2);

        let _ = client.close();

        Ok(())
    }
    #[test]
    fn test_fail_apply_migration() -> Result<(), Error> {
        let container = test_pg::Postgres::default()
            .start()
            .expect("Failed to start postgres container");
        let host_port = container
            .get_host_port_ipv4(5432)
            .expect("Failed to get postgres port");

        let database_url = format!(
            "host=127.0.0.1 port={host_port} user=postgres password=postgres dbname=postgres"
        );
        let mut client = Client::connect(&database_url, NoTls)?;
        let mut db = PostgresDatabase::new(&mut client);

        let m_1 = Migration::new(1, "create users", "CREATE TABLE users(name VARCHAR)");

        // failing migration - no table userssss
        let m_2 = Migration::new(
            2,
            "insert user",
            r#"INSERT into userssss(name) VALUES('test')"#,
        );
        let migrations = vec![m_1, m_2];

        db.create_migration_table()?;
        let result = db.apply_migrations(&migrations);
        match result {
            Err(Error::MigrationError { version, source: _ }) => {
                assert_eq!(version, 2);
            }
            _ => panic!("expected migrtions 2"),
        }

        let max_version = db.latest_migration()?;
        assert_eq!(max_version, 1);

        let _ = client.close();

        Ok(())
    }
}
