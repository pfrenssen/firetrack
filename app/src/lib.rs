use dotenv;
use std::env::var;

pub static APPLICATION_NAME: &str = "firetrack";

/// Contains the configuration options for the application. These values are typically coming from
/// the environment variables and are read only.
pub struct AppConfig {
    // The host IP address.
    // Todo: turn this into a `std::net::IpAddr` value.
    host: String,

    // The host port.
    // Todo: integrate this into the host IP address.
    port: u16,

    // The database URL.
    database_url: String,
}

impl AppConfig {
    /// Configures the application by supplying the values directly. Intended for testing.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let host = "127.0.0.1";
    /// let port = 8888;
    /// let database_url = "postgres://username:password@localhost/firetrack";
    ///
    /// let config = AppConfig::from(host, port, database_url);
    ///
    /// # assert_eq!(config.host(), host);
    /// # assert_eq!(config.port(), port);
    /// # assert_eq!(config.database_url(), database_url);
    /// ```
    pub fn from(host: &str, port: u16, database_url: &str) -> AppConfig {
        AppConfig {
            host: host.to_string(),
            port,
            database_url: database_url.to_string(),
        }
    }

    /// Configures the application by using environment variables.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    /// # use std::env;
    ///
    /// # let host = "127.0.0.1";
    /// # let port = 8888;
    /// # let database_url = "postgres://username:password@localhost/firetrack";
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    ///
    /// let config = AppConfig::from_environment();
    ///
    /// # assert_eq!(config.host(), host);
    /// # assert_eq!(config.port(), port);
    /// # assert_eq!(config.database_url(), database_url);
    /// ```
    pub fn from_environment() -> AppConfig {
        import_env_vars();
        AppConfig {
            host: var("HOST").expect("HOST environment variable is not set."),
            port: var("PORT")
                .expect("PORT environment variable is not set.")
                .parse()
                .unwrap(),
            database_url: var("DATABASE_URL")
                .expect("DATABASE_URL environment variable is not set."),
        }
    }

    /// Returns the host IP address.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    /// # use std::env;
    ///
    /// # let host = "127.0.0.1";
    /// # let port = 8888;
    /// # let database_url = "postgres://username:password@localhost/firetrack";
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    ///
    /// let config = AppConfig::from_environment();
    /// assert_eq!(config.host(), "127.0.0.1");
    /// ```
    pub fn host(&self) -> &str {
        self.host.as_str()
    }

    /// Returns the host port number.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    /// # use std::env;
    ///
    /// # let host = "127.0.0.1";
    /// # let port = 8888;
    /// # let database_url = "postgres://username:password@localhost/firetrack";
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    ///
    /// let config = AppConfig::from_environment();
    /// assert_eq!(config.port(), 8888);
    /// ```
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the host port number.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    /// # use std::env;
    ///
    /// # let host = "127.0.0.1";
    /// # let port = 8888;
    /// # let database_url = "postgres://username:password@localhost/firetrack";
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    ///
    /// let config = AppConfig::from_environment();
    /// assert_eq!(config.database_url(), "postgres://username:password@localhost/firetrack");
    /// ```
    pub fn database_url(&self) -> &str {
        self.database_url.as_str()
    }
}

/// Configures log output levels as defined in the `RUST_LOG` environment variable.
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
