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

    // The session key used to generate session IDs.
    session_key: [u8; 32],

    // The database URL.
    database_url: String,

    // The secret key used in password hashing.
    secret_key: String,

    // The amount of memory to use for password hashing, in kibibytes.
    hasher_memory_size: u32,

    // The number of password hashing iterations to perform.
    hasher_iterations: u32,

    // The path to the JSON file which lists the default categories for new users.
    default_categories_json_path: String,

    // The Mailgun API endpoint.
    mailgun_api_endpoint: String,

    // The API key for Mailgun.
    mailgun_api_key: String,

    // The domain to use for sending notifications.
    mailgun_user_domain: String,

    // The username used for sending notifications.
    mailgun_user_name: String,

    // The port to use for the Mailgun mock server.
    mailgun_mock_server_port: u16,
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
    /// # let session_key = [0; 32];
    /// # let database_url = "postgres://username:password@localhost/firetrack";
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 512;
    /// # let hasher_iterations = 1;
    /// # let default_categories_json_path = "../resources/fixtures/default-categories.json";
    /// # let mailgun_api_endpoint = mockito::server_url();
    /// # let mailgun_api_key = "0123456789abcdef0123456789abcdef-01234567-89abcdef";
    /// # let mailgun_user_domain = "sandbox0123456789abcdef0123456789abcdef.mailgun.org";
    /// # let mailgun_user_name = "postmaster";
    /// # let mailgun_mock_server_port = 8889;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("MAILGUN_MOCK_SERVER_PORT", mailgun_mock_server_port.to_string());
    ///
    /// let config = AppConfig::from_test_defaults();
    ///
    /// # assert_eq!(config.host(), host);
    /// # assert_eq!(config.port(), port);
    /// # assert_eq!(config.session_key(), session_key);
    /// # assert_eq!(config.database_url(), database_url);
    /// # assert_eq!(config.secret_key(), secret_key);
    /// # assert_eq!(config.hasher_memory_size(), hasher_memory_size);
    /// # assert_eq!(config.hasher_iterations(), hasher_iterations);
    /// # assert_eq!(config.default_categories_json_path(), default_categories_json_path);
    /// # assert_eq!(config.mailgun_api_endpoint(), mailgun_api_endpoint);
    /// # assert_eq!(config.mailgun_api_key(), mailgun_api_key);
    /// # assert_eq!(config.mailgun_user_domain(), mailgun_user_domain);
    /// # assert_eq!(config.mailgun_user_name(), mailgun_user_name);
    /// # assert_eq!(config.mailgun_mock_server_port(), mailgun_mock_server_port);
    /// ```
    pub fn from_test_defaults() -> AppConfig {
        import_env_vars();

        AppConfig {
            host: var("HOST").expect("HOST environment variable is not set."),
            port: var("PORT")
                .expect("PORT environment variable is not set.")
                .parse()
                .expect("PORT environment variable should be an integer value."),
            session_key: [0; 32],
            database_url: var("DATABASE_URL")
                .expect("DATABASE_URL environment variable is not set."),
            secret_key: "my_secret".to_string(),
            hasher_memory_size: 512,
            hasher_iterations: 1,
            default_categories_json_path: "../resources/fixtures/default-categories.json"
                .to_string(),
            mailgun_api_endpoint: mockito::server_url(),
            mailgun_api_key: "0123456789abcdef0123456789abcdef-01234567-89abcdef".to_string(),
            mailgun_user_domain: "sandbox0123456789abcdef0123456789abcdef.mailgun.org".to_string(),
            mailgun_user_name: "postmaster".to_string(),
            mailgun_mock_server_port: var("MAILGUN_MOCK_SERVER_PORT")
                .expect("MAILGUN_MOCK_SERVER_PORT environment variable is not set.")
                .parse()
                .expect(
                    "MAILGUN_MOCK_SERVER_PORT environment variable should be an integer value.",
                ),
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
    /// # let session_key = "1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1";
    /// # let database_url = "postgres://username:password@localhost/firetrack";
    /// # let secret_key = "my_secret";
    /// # let hasher_memory_size = 65536;
    /// # let hasher_iterations = 4096;
    /// # let default_categories_json_path = "resources/fixtures/default-categories.json";
    /// # let mailgun_api_endpoint = "https://api.mailgun.net/v3";
    /// # let mailgun_api_key = "0123456789abcdef0123456789abcdef-01234567-89abcdef";
    /// # let mailgun_user_domain = "sandbox0123456789abcdef0123456789abcdef.mailgun.org";
    /// # let mailgun_user_name = "postmaster";
    /// # let mailgun_mock_server_port = 8889;
    /// # env::set_var("HOST", host);
    /// # env::set_var("PORT", port.to_string());
    /// # env::set_var("SESSION_KEY", session_key.to_string());
    /// # env::set_var("DATABASE_URL", database_url);
    /// # env::set_var("SECRET_KEY", secret_key);
    /// # env::set_var("HASHER_MEMORY_SIZE", hasher_memory_size.to_string());
    /// # env::set_var("HASHER_ITERATIONS", hasher_iterations.to_string());
    /// # env::set_var("DEFAULT_CATEGORIES_JSON_PATH", default_categories_json_path.to_string());
    /// # env::set_var("MAILGUN_API_ENDPOINT", mailgun_api_endpoint.to_string());
    /// # env::set_var("MAILGUN_API_KEY", mailgun_api_key.to_string());
    /// # env::set_var("MAILGUN_USER_DOMAIN", mailgun_user_domain.to_string());
    /// # env::set_var("MAILGUN_USER_NAME", mailgun_user_name.to_string());
    /// # env::set_var("MAILGUN_MOCK_SERVER_PORT", mailgun_mock_server_port.to_string());
    ///
    /// let config = AppConfig::from_environment();
    ///
    /// # assert_eq!(config.host(), host);
    /// # assert_eq!(config.port(), port);
    /// # assert_eq!(config.session_key(), [1; 32]);
    /// # assert_eq!(config.database_url(), database_url);
    /// # assert_eq!(config.secret_key(), secret_key);
    /// # assert_eq!(config.hasher_memory_size(), hasher_memory_size);
    /// # assert_eq!(config.hasher_iterations(), hasher_iterations);
    /// # assert_eq!(config.default_categories_json_path(), default_categories_json_path);
    /// # assert_eq!(config.mailgun_api_endpoint(), mailgun_api_endpoint);
    /// # assert_eq!(config.mailgun_api_key(), mailgun_api_key);
    /// # assert_eq!(config.mailgun_user_domain(), mailgun_user_domain);
    /// # assert_eq!(config.mailgun_user_name(), mailgun_user_name);
    /// # assert_eq!(config.mailgun_mock_server_port(), mailgun_mock_server_port);
    /// ```
    pub fn from_environment() -> AppConfig {
        import_env_vars();

        // Check that the secret key is not empty.
        let secret_key = var("SECRET_KEY").expect("SECRET_KEY environment variable is not set.");
        if secret_key.is_empty() {
            panic!("SECRET_KEY environment variable is empty.");
        }

        // Cast the session key into a [u8; 32].
        let session_key = var("SESSION_KEY").expect("SESSION_KEY environment variable is not set.");
        let regex =
            r"^((1?[0-9]?[0-9]|2[0-4][0-9]|25[0-5]),){31}(1?[0-9]?[0-9]|2[0-4][0-9]|25[0-5])$";
        if !regex::Regex::new(regex)
            .unwrap()
            .is_match(session_key.as_str())
        {
            panic!("SESSION_KEY environment variable must be an array of 32 8-bit numbers.");
        }
        let session_key = session_key.split(',').map(|s| s.parse().unwrap()).cast();

        AppConfig {
            host: var("HOST").expect("HOST environment variable is not set."),
            port: var("PORT")
                .expect("PORT environment variable is not set.")
                .parse()
                .expect("PORT environment variable should be an integer value."),
            session_key,
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
            default_categories_json_path: var("DEFAULT_CATEGORIES_JSON_PATH")
                .expect("DEFAULT_CATEGORIES environment variable is not set."),
            mailgun_api_endpoint: var("MAILGUN_API_ENDPOINT")
                .expect("MAILGUN_API_ENDPOINT environment variable is not set."),
            mailgun_api_key: var("MAILGUN_API_KEY")
                .expect("MAILGUN_API_KEY environment variable is not set."),
            mailgun_user_domain: var("MAILGUN_USER_DOMAIN")
                .expect("MAILGUN_USER_DOMAIN environment variable is not set."),
            mailgun_user_name: var("MAILGUN_USER_NAME")
                .expect("MAILGUN_USER_NAME environment variable is not set."),
            mailgun_mock_server_port: var("MAILGUN_MOCK_SERVER_PORT")
                .expect("MAILGUN_MOCK_SERVER_PORT environment variable is not set.")
                .parse()
                .expect(
                    "MAILGUN_MOCK_SERVER_PORT environment variable should be an integer value.",
                ),
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

    /// Returns the session key.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.session_key(), [0; 32]);
    /// ```
    pub fn session_key(&self) -> [u8; 32] {
        self.session_key
    }

    /// Returns the database URL.
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

    /// Returns the path to the JSON file that contains default categories for new users.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.default_categories_json_path(), "../resources/fixtures/default-categories.json");
    /// ```
    pub fn default_categories_json_path(&self) -> &str {
        self.default_categories_json_path.as_str()
    }

    /// Returns the Mailgun API endpoint.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.mailgun_api_endpoint(), mockito::server_url());
    /// ```
    pub fn mailgun_api_endpoint(&self) -> &str {
        self.mailgun_api_endpoint.as_str()
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

    /// Returns the domain used for sending notifications.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.mailgun_user_domain(), "sandbox0123456789abcdef0123456789abcdef.mailgun.org");
    /// ```
    pub fn mailgun_user_domain(&self) -> &str {
        self.mailgun_user_domain.as_str()
    }

    /// Returns the user used for sending notifications.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.mailgun_user_name(), "postmaster");
    /// ```
    pub fn mailgun_user_name(&self) -> &str {
        self.mailgun_user_name.as_str()
    }

    /// Returns the port for the Mailgun mock server.
    ///
    /// # Example
    ///
    /// ```
    /// use app::AppConfig;
    ///
    /// let config = AppConfig::from_test_defaults();
    /// assert_eq!(config.mailgun_mock_server_port(), 8089);
    /// ```
    pub fn mailgun_mock_server_port(&self) -> u16 {
        self.mailgun_mock_server_port
    }

    // Todo: this should only be used for testing. Adding #[cfg(test)] doesn't work if the test code
    // is in another crate, because the method will not be found. Define a newtype in the test?
    pub fn set_default_categories_json_path(&mut self, default_categories_json_path: String) {
        self.default_categories_json_path = default_categories_json_path;
    }

    // Todo: this should only be used for testing.
    pub fn set_mailgun_api_key(&mut self, mailgun_api_key: String) {
        self.mailgun_api_key = mailgun_api_key;
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

use std::convert::AsMut;
use std::default::Default;

// Trait for casting a vector into an array.
// https://stackoverflow.com/a/60572615/350644
trait CastExt<T, U: Default + AsMut<[T]>>: Sized + Iterator<Item = T> {
    fn cast(mut self) -> U {
        let mut out: U = U::default();
        let arr: &mut [T] = out.as_mut();
        #[allow(clippy::needless_range_loop)]
        for i in 0..arr.len() {
            match self.next() {
                None => panic!("Array was not filled"),
                Some(v) => arr[i] = v,
            }
        }
        assert!(self.next().is_none(), "Array was overfilled");
        out
    }
}

impl<T, U: Iterator<Item = T>, V: Default + AsMut<[T]>> CastExt<T, V> for U {}
