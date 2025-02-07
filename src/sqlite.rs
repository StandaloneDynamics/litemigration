use sqlx::SqliteConnection;

use crate::{Migration, LiteMigrationError};

struct LiteSqlite{
    conn: SqliteConnection
}


// impl Migration for LiteSqlite{
//     async fn create_table(&mut self) -> Result<(), LiteMigrationError>{
//         println!("Creating migration table and initial migration");
//         let mut trxn = self.conn.begin().await.unwrap();
//         sqlx::query("
//             CREATE TABLE migration(
//             id SERIAL PRIMARY KEY NOT NULL,
//             version INTEGER UNIQUE NOT NULL"
//         ).execute(&mut *trxn).await.unwrap();

//         sqlx::query("INSERT INTO migration(version) values($1)").bind(1).execute(&mut *trxn).await.unwrap();
//         trxn.commit().await.unwrap();
//         Ok(())
//     }
// }