use sqlx::{Acquire, PgConnection, Row};

use crate::{schema::Schema, LiteMigrationError, Migration};

pub struct LitePostgresql<'a>{
    pub conn: &'a mut PgConnection
}

impl Migration for LitePostgresql<'_>{
    // When first ran, a migration table should be created.
    // This table will keep track of migrations.
    async fn create_table(&mut self) -> Result<(), LiteMigrationError>{
        println!("Creating migration table and initial migration");
        let mut trxn = self.conn.begin().await.unwrap();
        sqlx::query("
            CREATE TABLE migration(
            id SERIAL PRIMARY KEY NOT NULL,
            version INTEGER UNIQUE NOT NULL)"
        ).execute(&mut *trxn).await.unwrap();

        sqlx::query("INSERT INTO migration(version) values($1)").bind(1).execute(&mut *trxn).await.unwrap();
        trxn.commit().await.unwrap();
        Ok(())
    }

    // sanity check as to whether a db exists.
    async fn exists(&mut self) -> bool{
        sqlx::query("SELECT * from migration").fetch_one(&mut self.conn).await.unwrap();
        true
    }

    async fn latest_migration(&mut self) -> Result<i32, LiteMigrationError>{
        let row = sqlx::query("SELECT max(id) from migration").fetch_one(&mut self.conn).await.unwrap();
        let max_id = row.get("max");
        Ok(max_id)
    }
    // A vector of schema changes is looped through to check which ones need to be applied.
    async fn add_schema(&mut self, changes: &Vec<Schema<'_>>) -> Result<(), LiteMigrationError>{
        for change in changes.iter(){
            let max_id = self.latest_migration().await.unwrap();
            if max_id > change.id{
                println!("Change already applied")
            }else{
                let mut trxn = self.conn.begin().await.unwrap();
                sqlx::query(&change.query).execute(&mut *trxn).await.unwrap();
                sqlx::query("INSERT INTO migration(version) VALUES($1)").bind(change.id).execute(&mut *trxn).await.unwrap();
                trxn.commit().await.unwrap();
            }


        }
        Ok(())
    }

}



#[cfg(test)]
mod tests{
    use super::*;
    use sqlx::Connection;

    async fn drop_tables(conn: &mut PgConnection){
        sqlx::query("DROP TABLE litemigration").execute(&mut *conn).await.unwrap();
        sqlx::query("DROP TABLE cats").execute(&mut *conn).await.unwrap();

    }

    #[tokio::test]
    async fn test_postgres_connection(){
        let url = "postgresql://localhost/litemigration?sslmode=disable";
        let conn = PgConnection::connect(url).await.unwrap();


        let mut pg = LitePostgresql{conn};
        let result = pg.create_table().await;
        assert!(result.is_ok());

        let max_result = pg.latest_migration().await;
        assert!(max_result.is_ok_and(|r| r == 1));

        let changes = vec![
            Schema{id: 2, query: "CREATE TABLE cats(id SERIAL PRIMARY KEY NOT NULL, name VARCHAR)"},
            Schema{id: 3, query: "INSERT INTO cats(name) VALUES('John')"},
            Schema{id: 4, query: "INSERT INTO cats(name) VALUES('Peter')"},


        ];

        let result = pg.add_schema(&changes).await;
        assert!(result.is_ok());

        let max_result = pg.latest_migration().await;
        assert!(max_result.is_ok_and(|r| r == 4));

        let _ = drop_tables(&mut conn).await;



}
}

