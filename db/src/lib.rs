#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use std::fmt;
use std::process::exit;

mod schema;
pub mod user;

// Type alias to make it easier to refer to the connection pool.
pub type ConnectionPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Possible errors being thrown when working with the database.
#[derive(Debug, PartialEq)]
pub enum DatabaseError {
    // The connection pool could not be created.
    ConnectionPoolNotCreated(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DatabaseError::ConnectionPoolNotCreated(ref err) => {
                write!(f, "Connection pool could not be created: {}", err)
            }
        }
    }
}

// Creates a connection pool.
pub fn create_connection_pool(database_url: &str) -> Result<ConnectionPool, DatabaseError> {
    r2d2::Pool::builder()
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .map_err(|err| DatabaseError::ConnectionPoolNotCreated(format!("{}", err)))
}

// Establishes a non-pooled database connection.
// Todo: return a `Result<PgConnection, DatabaseError>`.
pub fn establish_connection(database_url: &str) -> PgConnection {
    match PgConnection::establish(&database_url) {
        Ok(value) => value,
        Err(e) => {
            error!("Could not connect to PostgreSQL.");
            error!("Error connecting to {}", database_url);
            error!("{}", e.to_string());
            exit(1);
        }
    }
}

// Imports environment variables by reading the .env files.
#[cfg(test)]
fn import_env_vars() {
    // Populate environment variables from the local `.env` file.
    dotenv::dotenv().ok();

    // Populate environment variables from the `.env.dist` file. This file contains sane defaults
    // as a fallback.
    dotenv::from_filename(".env.dist").ok();
}

// Retrieves the database URL from the environment variables.
#[cfg(test)]
fn get_database_url() -> String {
    import_env_vars();
    std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set.")
}
