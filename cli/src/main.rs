#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::{AppSettings, Arg, SubCommand};
use db::establish_connection;
use dotenv;
use std::env;
use std::process::exit;
use web::serve;

static APPLICATION_NAME: &str = "firetrack";

/// A trait that defines functions that will log an error and exit with an error code.
/// These can be used instead of panics to have clean logging in the console.
pub trait ExitWithError<T> {
    /// Unwraps an option or result, yielding the content of a [`Some`] or [`Ok`].
    ///
    /// # Exits
    ///
    /// Logs an error using the text provided by `msg` if the value is a [`None`] or [`Err`] and
    /// exits with an error code.
    fn expect_or_exit(self, msg: &str) -> T;

    /// Unwraps an option or result, yielding the content of a [`Some`] or [`Ok`].
    ///
    /// # Exits
    ///
    /// Exits with an error code if the value is a [`None`] or [`Err`]. If the value is an [`Err`]
    /// the corresponding error message will be logged.
    fn unwrap_or_exit(self) -> T;
}

impl<T> ExitWithError<T> for Option<T> {
    fn expect_or_exit(self, msg: &str) -> T {
        match self {
            Some(val) => val,
            None => {
                error!("{}", msg);
                exit(1);
            }
        }
    }

    fn unwrap_or_exit(self) -> T {
        match self {
            Some(val) => val,
            None => {
                error!("called `Option::unwrap()` on a `None` value");
                exit(1);
            }
        }
    }
}

impl<T, E: std::fmt::Display> ExitWithError<T> for Result<T, E> {
    fn expect_or_exit(self, msg: &str) -> T {
        match self {
            Ok(t) => t,
            Err(_) => {
                error!("{}", msg);
                exit(1);
            }
        }
    }

    fn unwrap_or_exit(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                error!("{}", &e);
                exit(1);
            }
        }
    }
}

fn main() {
    // Import environment variables from .env files, and initialize the logger.
    import_env_vars();
    env_logger::init();

    // Configure the CLI.
    let cli_app = clap::App::new(APPLICATION_NAME)
        .version(crate_version!())
        .subcommand(
            SubCommand::with_name("serve")
                .about(format!("Serve the {} web application", APPLICATION_NAME).as_str()),
        )
        .subcommand(
            SubCommand::with_name("useradd")
                .about("Create a new user account")
                .arg(
                    Arg::with_name("email")
                        .required(true)
                        .help("The user's email address"),
                )
                .arg(
                    Arg::with_name("password")
                        .required(true)
                        .help("The user's password"),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    // Launch the passed in subcommand.
    match cli_app.subcommand_name() {
        Some("serve") => {
            // Retrieve the hostname and port.
            let host = env::var("HOST").expect_or_exit("HOST environment variable is not set.");
            let port = env::var("PORT").expect_or_exit("PORT environment variable is not set.");

            serve(host.as_str(), port.as_str());
        }
        Some("useradd") => {
            if let Some(arguments) = cli_app.subcommand_matches("useradd") {
                let secret = env::var("SECRET_KEY")
                    .expect_or_exit("SECRET_KEY environment variable is not set.");
                if secret.is_empty() {
                    error!("SECRET_KEY environment variable is empty.");
                    exit(1);
                }
                let memory_size = env::var("HASHER_MEMORY_SIZE")
                    .expect_or_exit("HASHER_MEMORY_SIZE environment variable is not set.")
                    .parse()
                    .expect_or_exit(
                        "HASHER_MEMORY_SIZE environment variable should be an integer value.",
                    );
                let iterations = env::var("HASHER_ITERATIONS")
                    .expect_or_exit("HASHER_ITERATIONS environment variable is not set.")
                    .parse()
                    .expect_or_exit(
                        "HASHER_ITERATIONS environment variable should be an integer value.",
                    );

                db::user::create(
                    &establish_connection(),
                    arguments.value_of("email").unwrap(),
                    arguments.value_of("password").unwrap(),
                    secret.as_str(),
                    memory_size,
                    iterations,
                )
                .unwrap_or_exit();
            }
        }
        None => {}
        _ => unreachable!(),
    }
}

// Imports environment variables by reading the .env files.
fn import_env_vars() {
    // Populate environment variables from the local `.env` file.
    dotenv::dotenv().ok();

    // Populate environment variables from the `.env.dist` file. This file contains sane defaults
    // as a fallback.
    dotenv::from_filename(".env.dist").ok();
}
