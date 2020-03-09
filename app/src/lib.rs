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

    // The API key for Mailgun.
    mailgun_api_key: String,

    // The domain used for sending notifications.
    mailgun_domain: String,

    // The username used for sending notifications.
    mailgun_user: String,
}

impl AppConfig {
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
    /// # let mailgun_api_key = "0123456789abcdef0123456789abcdef-01234567-89abcdef";
    /// # let mailgun_domain = "sandbox0123456789abcdef0123456789abcdef.mailgun.org";
    /// # let mailgun_user = "postmaster";
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
    /// # assert_eq!(config.mailgun_api_key(), mailgun_api_key);
    /// # assert_eq!(config.mailgun_domain(), mailgun_domain);
    /// # assert_eq!(config.mailgun_user(), mailgun_user);
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
            mailgun_api_key: "0123456789abcdef0123456789abcdef-01234567-89abcdef".to_string(),
            mailgun_domain: "sandbox0123456789abcdef0123456789abcdef.mailgun.org".to_string(),
            mailgun_user: "postmaster".to_string(),
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
    /// # let mailgun_api_key = "0123456789abcdef0123456789abcdef-01234567-89abcdef";
    /// # let mailgun_domain = "sandbox0123456789abcdef0123456789abcdef.mailgun.org";
    /// # let mailgun_user = "postmaster";
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("SECRET_KEY", secret_key);
    /// # env::set_var("HASHER_MEMORY_SIZE", hasher_memory_size.to_string());
    /// # env::set_var("HASHER_ITERATIONS", hasher_iterations.to_string());
    /// # env::set_var("MAILGUN_API_KEY", mailgun_api_key.to_string());
    /// # env::set_var("MAILGUN_DOMAIN", mailgun_domain.to_string());
    /// # env::set_var("MAILGUN_USER", mailgun_user.to_string());
    ///
    /// let config = AppConfig::from_environment();
    ///
    /// # assert_eq!(config.host(), host);
    /// # assert_eq!(config.port(), port);
    /// # assert_eq!(config.database_url(), database_url);
    /// # assert_eq!(config.secret_key(), secret_key);
    /// # assert_eq!(config.hasher_memory_size(), hasher_memory_size);
    /// # assert_eq!(config.hasher_iterations(), hasher_iterations);
    /// # assert_eq!(config.mailgun_api_key(), mailgun_api_key);
    /// # assert_eq!(config.mailgun_domain(), mailgun_domain);
    /// # assert_eq!(config.mailgun_user(), mailgun_user);
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
            mailgun_api_key: var("MAILGUN_API_KEY")
                .expect("MAILGUN_API_KEY environment variable is not set."),
            mailgun_domain: var("MAILGUN_DOMAIN")
                .expect("MAILGUN_DOMAIN environment variable is not set."),
            mailgun_user: var("MAILGUN_USER")
                .expect("MAILGUN_USER environment variable is not set."),
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
    /// let host = "127.0.0.1";
    /// # env::set_var("HOST", host);
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.host(), host);
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
    /// let port = 8888;
    /// # env::set_var("PORT", port.to_string());
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.port(), port);
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
    /// let database_url = "postgres://username:password@localhost/firetrack";
    /// # env::set_var("DATABASE_URL", database_url);
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.database_url(), database_url);
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
    ///
    /// let config = AppConfig::from_test_defaults();
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
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.hasher_memory_size(), 512);
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
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.hasher_iterations(), 1);
    /// ```
    pub fn hasher_iterations(&self) -> u32 {
        self.hasher_iterations
    }

    /// Returns the Mailgun API key.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.mailgun_api_key(), "0123456789abcdef0123456789abcdef-01234567-89abcdef");
    /// ```
    pub fn mailgun_api_key(&self) -> &str {
        self.mailgun_api_key.as_str()
    }

    // Todo: this should only be used for testing. Adding #[cfg(test)] doesn't work if the test code
    // is in another crate, because the method will not be found. Define a newtype in the test?
    pub fn set_mailgun_api_key(&mut self, mailgun_api_key: String) {
        self.mailgun_api_key = mailgun_api_key;
    }

    /// Returns the domain used for sending notifications.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.mailgun_domain(), "sandbox0123456789abcdef0123456789abcdef.mailgun.org");
    /// ```
    pub fn mailgun_domain(&self) -> &str {
        self.mailgun_domain.as_str()
    }

    /// Returns the user used for sending notifications.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.mailgun_user(), "postmaster");
    /// ```
    pub fn mailgun_user(&self) -> &str {
        self.mailgun_user.as_str()
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
