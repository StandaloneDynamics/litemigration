use sqlx::{SqliteConnection, Connection, Sqlite, migrate::MigrateDatabase, PgConnection};
use litemigration::{sqlite::LiteSqlite, schema::Schema, Migration, postgres::LitePostgresql};
use std;


async fn sqlite_drop_tables(conn: &mut SqliteConnection){
    sqlx::query("DROP TABLE migration").execute(&mut *conn).await.unwrap();
    sqlx::query("DROP TABLE cats").execute(&mut *conn).await.unwrap();
    std::fs::remove_file("litemigration.db").unwrap();

}

async fn postgres_drop_tables(conn: &mut PgConnection){
    sqlx::query("DROP TABLE migration").execute(&mut *conn).await.unwrap();
    sqlx::query("DROP TABLE cats").execute(&mut *conn).await.unwrap();

}




#[tokio::test]
async fn sqlite_migration(){
    let path = "sqlite://litemigration.db";
    Sqlite::create_database(&path).await.unwrap();
    let mut conn = SqliteConnection::connect(&path).await.unwrap();

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
    assert!(result.is_ok());

    let max_result = lite.latest_migration().await;
    assert!(max_result.is_ok_and(|r| r == 4));

    let _ = sqlite_drop_tables(&mut conn).await;
}

#[tokio::test]
async fn postgres_migration(){
    let url = "postgresql://localhost/litemigration?sslmode=disable";
        let mut conn = PgConnection::connect(url).await.unwrap();


        let mut pg = LitePostgresql{conn: &mut conn};
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

        let _ = postgres_drop_tables(&mut conn).await;
}