use dotenv;

pub static APPLICATION_NAME: &str = "firetrack";

pub fn initialize_logger() {
    import_env_vars();
    env_logger::init();
}

// Imports environment variables by reading the .env files.
fn import_env_vars() {
    // Populate environment variables from the local `.env` file.
    dotenv::dotenv().ok();

    // Populate environment variables from the `.env.dist` file. This file contains sane defaults
    // as a fallback.
    dotenv::from_filename(".env.dist").ok();
}
