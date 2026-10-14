/// A single database migration

/// Migrations are executed in numerial order
/// and are forward only.
/// Once a migration is applied, its SQL cannot be modified.
/// A new migration will have to be created.
pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

impl Migration {
    /// Creates a new migration instance
    pub fn new(version: i32, name: &'static str, sql: &'static str) -> Self {
        Self { version, name, sql }
    }
}
