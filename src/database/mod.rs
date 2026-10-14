pub mod pg;
pub mod sqlite;

use crate::error::Error;
use crate::migration::Migration;

/// Provides database operations required by the library

/// Implement this trait to add support for another database backend.
/// Implementations should ensure that migrations happen within database transactions.
pub trait Database {
    /// Create a database table containing columns (version and name)
    fn create_migration_table(&mut self) -> Result<(), Error>;
    /// Applies each migration and updates the migration table with the latest
    /// migration version.
    fn apply_migrations(&mut self, migrations: &[Migration]) -> Result<(), Error>;
    /// Returns tha latest migration that was applied
    fn latest_migration(&mut self) -> Result<i32, Error>;
}
