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

#[actix_rt::main]
async fn main() {
    // Use custom log levels. This can be configured in the .env files.
    initialize_logger();

    let config = AppConfig::from_environment();

    // Configure the CLI.
    let cli_app =
        clap::App::new(APPLICATION_NAME)
            .version(crate_version!())
            // The actual filename of the compiled binary is "cli" but this is renamed to
            // "firetrack" during packaging.
            .bin_name(APPLICATION_NAME)
            .subcommand(
                SubCommand::with_name("serve")
                    .about(format!("Serve the {} web application", APPLICATION_NAME).as_str()),
            )
            .subcommand(
                SubCommand::with_name("user")
                    .about("Commands for managing users")
                    .subcommands(vec![
                        SubCommand::with_name("add")
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
                        SubCommand::with_name("delete")
                            .about("Delete a user account")
                            .arg(
                                Arg::with_name("email")
                                    .required(true)
                                    .help("The email address of the user to delete"),
                            ),
                        SubCommand::with_name("activate")
                            .about("Activates a user account")
                            .arg(
                                Arg::with_name("email")
                                    .required(true)
                                    .help("The user's email address"),
                            )
                            .arg(
                                Arg::with_name("code")
                                    .required(true)
                                    .help("The activation code"),
                            ),
                    ])
                    .setting(AppSettings::SubcommandRequiredElseHelp),
            )
            .subcommand(
                SubCommand::with_name("activation-code")
                    .about("Commands for managing activation codes")
                    .subcommands(vec![
                        SubCommand::with_name("get")
                            .about("Retrieves an activation code")
                            .arg(Arg::with_name("email").required(true).help(
                                "The email address for which to retrieve an activation code",
                            )),
                        SubCommand::with_name("delete")
                            .about("Deletes an activation code")
                            .arg(
                                Arg::with_name("email").required(true).help(
                                    "The email address for which to delete the activation code",
                                ),
                            ),
                        SubCommand::with_name("purge").about("Purges expired activation codes"),
                    ])
                    .setting(AppSettings::SubcommandRequiredElseHelp),
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
                    )
                    .setting(AppSettings::SubcommandRequiredElseHelp),
            )
            .subcommand(
                SubCommand::with_name("mailgun-mock-server").about("Start the Mailgun mock server"),
            )
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .get_matches();

    // Launch the passed in subcommand.
    match cli_app.subcommand() {
        ("serve", _) => {
            serve(config).await.unwrap_or_exit();
        }
        ("user", Some(arguments)) => match arguments.subcommand() {
            ("add", Some(arguments)) => {
                db::user::create(
                    &establish_connection(&config.database_url()),
                    arguments.value_of("email").unwrap(),
                    arguments.value_of("password").unwrap(),
                    &config,
                )
                .unwrap_or_exit();
            }
            ("delete", Some(arguments)) => {
                db::user::delete(
                    &establish_connection(&config.database_url()),
                    arguments.value_of("email").unwrap(),
                )
                .unwrap_or_exit();
            }
            ("activate", Some(arguments)) => {
                let connection = establish_connection(&config.database_url());
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap_or_exit();
                let activation_code = arguments.value_of("code").unwrap().parse().unwrap_or_exit();
                db::activation_code::activate_user(&connection, user, activation_code)
                    .unwrap_or_exit();
            }
            ("", None) => {}
            _ => unreachable!(),
        },
        ("activation-code", Some(arguments)) => match arguments.subcommand() {
            ("get", Some(arguments)) => {
                let connection = establish_connection(&config.database_url());
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap_or_exit();
                let activation_code = db::activation_code::get(&connection, &user).unwrap_or_exit();
                println!("{}", activation_code.code);
            }
            ("delete", Some(arguments)) => {
                let connection = establish_connection(&config.database_url());
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap_or_exit();
                db::activation_code::delete(&connection, &user).unwrap_or_exit();
            }
            ("purge", _) => {
                let connection = establish_connection(&config.database_url());
                db::activation_code::purge(&connection).unwrap_or_exit();
            }
            ("", None) => {}
            _ => unreachable!(),
        },
        ("notify", Some(notify)) => match notify.subcommand() {
            ("activate", Some(arguments)) => {
                let connection = establish_connection(&config.database_url());
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap_or_exit();
                let activation_code = db::activation_code::get(&connection, &user).unwrap_or_exit();
                notifications::activate(&user, &activation_code, &config)
                    .await
                    .unwrap_or_exit();
            }
            ("", None) => {}
            _ => unreachable!(),
        },
        ("mailgun-mock-server", _) => {
            mailgun_mock::serve(config).await.unwrap_or_exit();
        }
        ("", None) => {}
        _ => unreachable!(),
    }
}
