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
        // The actual filename of the compiled binary is "cli" but we plan to rename this to
        // "firetrack" when packaging.
        .bin_name(APPLICATION_NAME)
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
        .subcommand(
            SubCommand::with_name("notify")
                .about("Send a notification")
                .subcommand(
                    SubCommand::with_name("activate")
                        .about("Send an activation email")
                        .arg(
                            Arg::with_name("email")
                                .required(true)
                                .help("The email address to activate"),
                        ),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    // Launch the passed in subcommand.
    match cli_app.subcommand() {
        ("serve", _) => {
            serve(config);
        }
        ("useradd", Some(arguments)) => {
            db::user::create(
                &establish_connection(&config.database_url()),
                arguments.value_of("email").unwrap(),
                arguments.value_of("password").unwrap(),
                &config,
            )
            .unwrap_or_exit();
        }
        ("notify", Some(notify)) => match notify.subcommand() {
            ("activate", Some(arguments)) => {
                let connection = establish_connection(&config.database_url());
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap();
                notifications::activate(&user, &config);
            }
            _ => unreachable!(),
        },
        ("", None) => {}
        _ => unreachable!(),
    }
}
