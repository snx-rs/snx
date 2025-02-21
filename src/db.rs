#[cfg(feature = "sqlite")]
pub type DatabaseConnection = diesel::SqliteConnection;
#[cfg(feature = "postgres")]
pub type DatabaseConnection = diesel::PgConnection;
#[cfg(feature = "mysql")]
pub type DatabaseConnection = diesel::MySqlConnection;
