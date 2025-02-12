use sqlx::{Acquire, SqliteConnection, Row};

use crate::{schema::Schema, Migration, LiteMigrationError};

pub struct LiteSqlite<'a >{
    pub conn: &'a mut SqliteConnection
}


impl Migration for LiteSqlite<'_>{
    async fn create_table(&mut self) -> Result<(), LiteMigrationError>{
        println!("Creating migration table and initial migration");
        let mut trxn = self.conn.begin().await.unwrap();
        sqlx::query("
            CREATE TABLE migration(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version INTEGER UNIQUE NOT NULL)"
        ).execute(&mut *trxn).await.unwrap();

        sqlx::query("INSERT INTO migration(version) values($1)").bind(1).execute(&mut *trxn).await.unwrap();
        trxn.commit().await.unwrap();
        Ok(())
    }
     // sanity check as to whether a db exists.
     async fn exists(&mut self) -> bool{
        match sqlx::query("SELECT * from migration").fetch_one(&mut *self.conn).await{
            Ok(_) => return true,
            Err(_) => return false
        };
    }

    async fn latest_migration(&mut self) -> Result<i32, LiteMigrationError>{
        let row = match sqlx::query("SELECT max(version) from migration").fetch_one(&mut *self.conn).await{
            Ok(row) => row,
            Err(err) => return Err(LiteMigrationError::DatabaseError(err))
        };
        let max_id = row.get("max(version)");
        Ok(max_id)
    }
    async fn add_schema(&mut self, changes: &Vec<Schema<'_>>) -> Result<(), LiteMigrationError>{
        for change in changes.iter(){
            let max_id = match self.latest_migration().await{
                Ok(max) => max,
                Err(err) => return Err(err)
            };
            if change.id > max_id{
                let mut trxn = match self.conn.begin().await{
                    Ok(trn) => trn,
                    Err(err) => return Err(LiteMigrationError::MigrationError{value: change.id, error: err})
                };
                if let Err(err) = sqlx::query(&change.query).execute(&mut *trxn).await{
                    return Err(LiteMigrationError::MigrationError{value: change.id, error: err})
                } 
                
                if let Err(err) = sqlx::query("INSERT INTO migration(version) VALUES($1)").bind(change.id).execute(&mut *trxn).await{
                    return Err(LiteMigrationError::MigrationError{value: change.id, error: err})
                } 
                
                if let Err(err) = trxn.commit().await{
                    return Err(LiteMigrationError::MigrationError{value: change.id, error: err})
                }
            }

        }
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use sqlx::{migrate::MigrateDatabase, Connection, Sqlite};
    use std;

    async fn drop_tables(conn: &mut SqliteConnection){
        sqlx::query("DROP TABLE migration").execute(&mut *conn).await.unwrap();
        sqlx::query("DROP TABLE cats").execute(&mut *conn).await.unwrap();
        std::fs::remove_file("litemigration.db").unwrap();

    }

    #[tokio::test]
    async fn test_sqlite_connection(){
        let path = "sqlite://litemigration.db";
        Sqlite::create_database(&path).await.unwrap();
        let mut conn = SqliteConnection::connect(path).await.unwrap();


        let mut lite = LiteSqlite{conn: &mut conn};
        let result = lite.create_table().await;
        assert!(result.is_ok());

        let max_result = lite.latest_migration().await;
        assert!(max_result.is_ok_and(|r| r == 1));

        let changes = vec![
            Schema{id: 2, query: "CREATE TABLE cats(id INTEGER PRIMARY KEY AUTOINCREMENT, name VARCHAR)"},
            Schema{id: 3, query: "INSERT INTO cats(name) VALUES('John')"},
            Schema{id: 4, query: "INSERT INTO cats(name) VALUES('Peter')"},
        ];

        let result = lite.add_schema(&changes).await;
        //println!("{:?}", result.err());
        assert!(result.is_ok());

        let max_result = lite.latest_migration().await;
        assert!(max_result.is_ok_and(|r| r == 4));

        let _ = drop_tables(&mut conn).await;

}

#[tokio::test]
async fn test_migrate_error(){
    let path = "sqlite://litemigration.db";
    let _ = Sqlite::create_database(&path).await.unwrap();
    let mut conn = SqliteConnection::connect(&path).await.unwrap();

    let mut pg = LiteSqlite{conn: &mut conn};
    let result = pg.create_table().await;
    assert!(result.is_ok());

    let changes = vec![
            Schema{id: 2, query: "CREATE TABLE cats(id INTERGER PRIMARY KEY NOT NULL, name VARCHAR)"},
            Schema{id: 3, query: "INSERTS INTO cats(name) VALUES('John')"},
            Schema{id: 4, query: "INSERT INTO cats(name) VALUES('Peter')"},
        ];

    let result = pg.add_schema(&changes).await;
    assert!(result.is_err());


    let _ = drop_tables(&mut conn).await;
}
}