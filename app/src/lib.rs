use dotenv;
use std::env::var;

pub static APPLICATION_NAME: &str = "firetrack";

/// Contains the configuration options for the application. These values are typically coming from
/// the environment variables and are read only.
#[derive(Clone, Debug)]
pub struct AppConfig {
    // The host IP address.
    // Todo: turn this into a `std::net::IpAddr` value.
    host: String,

    // The host port.
    // Todo: integrate this into the host IP address.
    port: u16,

    // The database URL.
    database_url: String,

    // The secret key used in password hashing.
    secret_key: String,

    // The amount of memory to use for password hashing, in kibibytes.
    hasher_memory_size: u32,

    // The number of password hashing iterations to perform.
    hasher_iterations: u32,
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
    /// let secret_key = "my_secret";
    /// let hasher_memory_size = 65536;
    /// let hasher_iterations = 512;
    ///
    /// let config = AppConfig::from(host, port, database_url, secret_key, hasher_memory_size, hasher_iterations);
    ///
    /// # assert_eq!(config.host(), host);
    /// # assert_eq!(config.port(), port);
    /// # assert_eq!(config.database_url(), database_url);
    /// # assert_eq!(config.secret_key(), secret_key);
    /// # assert_eq!(config.hasher_memory_size(), hasher_memory_size);
    /// # assert_eq!(config.hasher_iterations(), hasher_iterations);
    /// ```
    pub fn from(
        host: &str,
        port: u16,
        database_url: &str,
        secret_key: &str,
        hasher_memory_size: u32,
        hasher_iterations: u32,
    ) -> AppConfig {
        AppConfig {
            host: host.to_string(),
            port,
            database_url: database_url.to_string(),
            secret_key: secret_key.to_string(),
            hasher_memory_size,
            hasher_iterations,
        }
    }

    /// Configures the application using default test values.
    ///
    /// This is taking essential parameters such as the hostname and database credentials from the
    /// environment variables, and uses hardcoded default values for everything else. Note that this
    /// configuration is insecure and should only be used for testing.
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
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 512;
    /// # let hasher_iterations = 1;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    ///
    /// let config = AppConfig::from_test_defaults();
    ///
    /// # assert_eq!(config.host(), host);
    /// # assert_eq!(config.port(), port);
    /// # assert_eq!(config.database_url(), database_url);
    /// # assert_eq!(config.secret_key(), secret_key);
    /// # assert_eq!(config.hasher_memory_size(), hasher_memory_size);
    /// # assert_eq!(config.hasher_iterations(), hasher_iterations);
    /// ```
    pub fn from_test_defaults() -> AppConfig {
        import_env_vars();

        AppConfig {
            host: var("HOST").expect("HOST environment variable is not set."),
            port: var("PORT")
                .expect("PORT environment variable is not set.")
                .parse()
                .expect("PORT environment variable should be an integer value."),
            database_url: var("DATABASE_URL")
                .expect("DATABASE_URL environment variable is not set."),
            secret_key: "my_secret".to_string(),
            hasher_memory_size: 512,
            hasher_iterations: 1,
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
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 65536;
    /// # let hasher_iterations = 4096;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("SECRET_KEY", secret_key);
    /// # env::set_var("HASHER_MEMORY_SIZE", hasher_memory_size.to_string());
    /// # env::set_var("HASHER_ITERATIONS", hasher_iterations.to_string());
    ///
    /// let config = AppConfig::from_environment();
    ///
    /// # assert_eq!(config.host(), host);
    /// # assert_eq!(config.port(), port);
    /// # assert_eq!(config.database_url(), database_url);
    /// # assert_eq!(config.secret_key(), secret_key);
    /// # assert_eq!(config.hasher_memory_size(), hasher_memory_size);
    /// # assert_eq!(config.hasher_iterations(), hasher_iterations);
    /// ```
    pub fn from_environment() -> AppConfig {
        import_env_vars();

        let secret_key = var("SECRET_KEY").expect("SECRET_KEY environment variable is not set.");
        if secret_key.is_empty() {
            panic!("SECRET_KEY environment variable is empty.");
        }

        AppConfig {
            host: var("HOST").expect("HOST environment variable is not set."),
            port: var("PORT")
                .expect("PORT environment variable is not set.")
                .parse()
                .expect("PORT environment variable should be an integer value."),
            database_url: var("DATABASE_URL")
                .expect("DATABASE_URL environment variable is not set."),
            secret_key,
            hasher_memory_size: var("HASHER_MEMORY_SIZE")
                .expect("HASHER_MEMORY_SIZE environment variable is not set.")
                .parse()
                .expect("HASHER_MEMORY_SIZE environment variable should be an integer value."),
            hasher_iterations: var("HASHER_ITERATIONS")
                .expect("HASHER_ITERATIONS environment variable is not set.")
                .parse()
                .expect("HASHER_ITERATIONS environment variable should be an integer value."),
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
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 65536;
    /// # let hasher_iterations = 4096;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("SECRET_KEY", secret_key);
    /// # env::set_var("HASHER_MEMORY_SIZE", hasher_memory_size.to_string());
    /// # env::set_var("HASHER_ITERATIONS", hasher_iterations.to_string());
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
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 65536;
    /// # let hasher_iterations = 4096;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("SECRET_KEY", secret_key);
    /// # env::set_var("HASHER_MEMORY_SIZE", hasher_memory_size.to_string());
    /// # env::set_var("HASHER_ITERATIONS", hasher_iterations.to_string());
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
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 65536;
    /// # let hasher_iterations = 4096;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("SECRET_KEY", secret_key);
    /// # env::set_var("HASHER_MEMORY_SIZE", hasher_memory_size.to_string());
    /// # env::set_var("HASHER_ITERATIONS", hasher_iterations.to_string());
    ///
    /// let config = AppConfig::from_environment();
    /// assert_eq!(config.database_url(), "postgres://username:password@localhost/firetrack");
    /// ```
    pub fn database_url(&self) -> &str {
        self.database_url.as_str()
    }

    /// Returns the secret key.
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
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 65536;
    /// # let hasher_iterations = 4096;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("SECRET_KEY", secret_key);
    /// # env::set_var("HASHER_MEMORY_SIZE", hasher_memory_size.to_string());
    /// # env::set_var("HASHER_ITERATIONS", hasher_iterations.to_string());
    ///
    /// let config = AppConfig::from_environment();
    /// assert_eq!(config.secret_key(), "my_secret");
    /// ```
    pub fn secret_key(&self) -> &str {
        self.secret_key.as_str()
    }

    /// Returns the amount of memory to use for password hashing, in kibibytes.
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
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 65536;
    /// # let hasher_iterations = 4096;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("SECRET_KEY", secret_key);
    /// # env::set_var("HASHER_MEMORY_SIZE", hasher_memory_size.to_string());
    /// # env::set_var("HASHER_ITERATIONS", hasher_iterations.to_string());
    ///
    /// let config = AppConfig::from_environment();
    /// assert_eq!(config.hasher_memory_size(), 65536);
    /// ```
    pub fn hasher_memory_size(&self) -> u32 {
        self.hasher_memory_size
    }

    /// Returns the number of password hashing iterations to perform.
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
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 65536;
    /// # let hasher_iterations = 4096;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("SECRET_KEY", secret_key);
    /// # env::set_var("HASHER_MEMORY_SIZE", hasher_memory_size.to_string());
    /// # env::set_var("HASHER_ITERATIONS", hasher_iterations.to_string());
    ///
    /// let config = AppConfig::from_environment();
    /// assert_eq!(config.hasher_iterations(), 4096);
    /// ```
    pub fn hasher_iterations(&self) -> u32 {
        self.hasher_iterations
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
