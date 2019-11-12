#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use app::*;
use clap::{AppSettings, Arg, SubCommand};
use db::establish_connection;
use std::env;
use std::process::exit;
use web::serve;

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
    // Use custom log levels. This can be configured in the .env files.
    initialize_logger();

    let config = AppConfig::from_environment();

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
            serve(config);
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

                let database_url = env::var("DATABASE_URL")
                    .expect_or_exit("DATABASE_URL environment variable is not set.");

                db::user::create(
                    &establish_connection(&database_url),
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
