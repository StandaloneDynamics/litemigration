use std::fmt;

#[cfg(feature = "sqlite")]
use rusqlite;

#[cfg(feature = "postgres")]
use postgres;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "sqlite")]
    SqliteError(rusqlite::Error),

    #[cfg(feature = "postgres")]
    PostgresError(postgres::Error),

    MigrationError {
        version: i32,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl std::error::Error for Error {}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Error {
        Error::SqliteError(err)
    }
}

#[cfg(feature = "postgres")]
impl From<postgres::Error> for Error {
    fn from(err: postgres::Error) -> Error {
        Error::PostgresError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "sqlite")]
            Error::SqliteError(error) => {
                write!(f, "Sqlite error: {error}")
            }
            #[cfg(feature = "postgres")]
            Error::PostgresError(error) => {
                write!(f, "Postgres error: {error}")
            }
            Error::MigrationError { version, source } => {
                write!(f, "Migration error on version: {version} {source}")
            }
        }
    }
}
