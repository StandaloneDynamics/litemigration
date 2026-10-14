# Litemigration

litemigrtion is a simple forward-only database migration library for small rust applications that require a database.

The library provides a common migrations API supporting Sqlite and Postgreql.


## Features
* Forward-only migrations.
* Migrations versioning and tracking within the database.
* Migrations names for ease of reference.
* Database implementation behind a Database trait, can be expanded to add more databases.



## Installation

Add the library to Cargo.toml.
Either the `sqlite` or the `postgres` feature has to be specified.


For SQLite:
library uses `rusqlite` crate.

[dependencies]
litemigration = { version = "0.1", features = ["sqlite"] }


For Postgresql:
library user the `postgres` sql drriver crate.

[dependencies] 
litemigration = { version = "0.1", features = ["postgres"] }


## Usage

### Creating a migration

```rust
use litemigration::Migration;

const MIGRATAIONS: &[MIGRATION] = &[
	Migration::new(
		1, 
		"create user", 
		r#"create user(id INTEGER PRIMARY KEY, name TEXT NOT NULL)"#
	),
	Migration::new(
		2, 
		"insert john user", 
		r#"INSERT INTO user(name) VALUES('john')"#
	)

]

```

### Migrating

```rust

fn main() -> Result<()> {
	
	// Sqlite Connection
    let mut connection = Connection::open("application.db")?;
    let database = SqliteDatabase::new(&mut connection);
	
	// Postgres Connection
	let mut client = Client::connect(&database_url)?;
    let mut database = PostgresDatabase::new(&mut client)?;

	// migrations
    let mut migrator = Migrator::new(database);

    migrator.migrate(MIGRATIONS)?;

    Ok(())
}


```
