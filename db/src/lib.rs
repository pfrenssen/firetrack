#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;
use std::process::exit;

mod schema;
pub mod user;

// Establishes a non-pooled database connection.
pub fn establish_connection() -> PgConnection {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set.");

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
