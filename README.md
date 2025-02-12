# litemigration
Simple embedded migrations for small rust applications.

Supports both `Sqlite` and `Postgresql`

Only forward migrations are applied.


## Example

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