#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use app::*;
use clap::{AppSettings, Arg, SubCommand};
use db::establish_connection;
use rust_decimal::Decimal;
use serde_json::json;
use std::env;
use std::process::exit;
use std::str::FromStr;
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
                SubCommand::with_name("category")
                    .about("Commands for managing categories")
                    .subcommands(vec![
                        SubCommand::with_name("add")
                            .about("Create a new category")
                            .arg(Arg::with_name("email").required(true).help(
                                "The email address of the account for which to create the category",
                            ))
                            .arg(
                                Arg::with_name("name")
                                    .required(true)
                                    .help("The category name"),
                            )
                            .arg(
                                Arg::with_name("description")
                                    .long("description")
                                    .short("d")
                                    .takes_value(true)
                                    .help("The description"),
                            )
                            .arg(
                                Arg::with_name("parent_id")
                                    .long("parent")
                                    .short("p")
                                    .takes_value(true)
                                    .help("The ID of the parent category"),
                            ),
                        SubCommand::with_name("get")
                            .about("Outputs a category as JSON data")
                            .arg(Arg::with_name("id").required(true).help("The category ID")),
                        SubCommand::with_name("delete")
                            .about("Deletes a category")
                            .arg(Arg::with_name("id").required(true).help("The category ID")),
                        SubCommand::with_name("populate")
                            .about("Populates the categories for a new user")
                            .arg(Arg::with_name("email").required(true).help(
                                "The email address of the account for which to populate the categories",
                            )),
                    ])
                    .setting(AppSettings::SubcommandRequiredElseHelp),
            )
            .subcommand(
                SubCommand::with_name("expense")
                    .about("Commands for managing expenses")
                    .subcommands(vec![
                        SubCommand::with_name("add")
                            .about("Create a new expense")
                            .arg(Arg::with_name("email").required(true).help(
                                "The email address of the account for which to create the expense",
                            ))
                            .arg(
                                Arg::with_name("amount")
                                    .required(true)
                                    .help("The amount that was spent"),
                            )
                            .arg(
                                Arg::with_name("category_id")
                                    .required(true)
                                    .help("The ID of the category"),
                            )
                            .arg(
                                Arg::with_name("description")
                                    .long("description")
                                    .short("d")
                                    .takes_value(true)
                                    .help("The description"),
                            )
                            .arg(
                                Arg::with_name("date")
                                    .long("date")
                                    .takes_value(true)
                                    .help("The date for the expense, in the format YYYY-MM-DD. If omitted, today's date will be used."),
                            ),
                        SubCommand::with_name("get")
                            .about("Outputs an expense as JSON data")
                            .arg(Arg::with_name("id").required(true).help("The expense ID")),
                        SubCommand::with_name("delete")
                            .about("Deletes an expense")
                            .arg(Arg::with_name("id").required(true).help("The expense ID")),
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
                    &establish_connection(&config.database_url()).unwrap_or_exit(),
                    arguments.value_of("email").unwrap(),
                    arguments.value_of("password").unwrap(),
                    &config,
                )
                .unwrap_or_exit();
            }
            ("delete", Some(arguments)) => {
                db::user::delete(
                    &establish_connection(&config.database_url()).unwrap_or_exit(),
                    arguments.value_of("email").unwrap(),
                )
                .unwrap_or_exit();
            }
            ("activate", Some(arguments)) => {
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
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
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap_or_exit();
                let activation_code = db::activation_code::get(&connection, &user).unwrap_or_exit();
                println!("{}", activation_code.code);
            }
            ("delete", Some(arguments)) => {
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap_or_exit();
                db::activation_code::delete(&connection, &user).unwrap_or_exit();
            }
            ("purge", _) => {
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                db::activation_code::purge(&connection).unwrap_or_exit();
            }
            ("", None) => {}
            _ => unreachable!(),
        },
        ("category", Some(arguments)) => match arguments.subcommand() {
            ("add", Some(arguments)) => {
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap_or_exit();

                // Check that the parent category ID is a numeric value.
                let parent_id =
                    assert_integer_argument(arguments.value_of("parent_id"), "parent category ID");

                // Check that the parent with the given ID exists.
                let parent = match parent_id {
                    Some(id) => {
                        let category = db::category::read(&connection, id);
                        if category.is_none() {
                            let message = format!("Category with ID {} could not be loaded", id);
                            Err::<String, _>(message).unwrap_or_exit();
                        };
                        category
                    }
                    None => None,
                };

                db::category::create(
                    &establish_connection(&config.database_url()).unwrap_or_exit(),
                    &user,
                    arguments.value_of("name").unwrap(),
                    arguments.value_of("description"),
                    parent.as_ref(),
                )
                .unwrap_or_exit();
            }
            ("get", Some(arguments)) => {
                let id = assert_integer_argument(arguments.value_of("id"), "category ID").unwrap();
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                let category = db::category::read(&connection, id);
                if category.is_none() {
                    Err::<String, _>("Category not found").unwrap_or_exit();
                };
                println!("{}", json!(category.unwrap()));
            }
            ("delete", Some(arguments)) => {
                let id = assert_integer_argument(arguments.value_of("id"), "category ID").unwrap();
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                db::category::delete(&connection, id).unwrap_or_exit();
            }
            ("populate", Some(arguments)) => {
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap_or_exit();

                db::category::populate_categories(
                    &establish_connection(&config.database_url()).unwrap_or_exit(),
                    &user,
                    &config,
                )
                .unwrap_or_exit();
            }
            ("", None) => {}
            _ => unreachable!(),
        },
        ("expense", Some(arguments)) => match arguments.subcommand() {
            ("add", Some(arguments)) => {
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                let email = arguments.value_of("email").unwrap();
                let user = db::user::read(&connection, email).unwrap_or_exit();

                // Check that the amount is in decimal format with maximum two fractional digits.
                let amount = arguments.value_of("amount").unwrap();
                if amount.is_empty()
                    || !regex::Regex::new(r"^\d{0,7}(\.\d{1,2})?$")
                        .unwrap()
                        .is_match(amount)
                {
                    Err::<String, _>("Amount should be in the format \"149.99\"").unwrap_or_exit();
                }
                let amount = Decimal::from_str(amount).unwrap_or_exit();

                let date = arguments.value_of("date").map(|d| {
                    chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                        .map_err(|_| {
                            "The date should be valid and in the format YYYY-MM-DD".to_string()
                        })
                        .unwrap_or_exit()
                });

                // Check that the category ID is a numeric value.
                let category_id =
                    assert_integer_argument(arguments.value_of("category_id"), "category ID")
                        .unwrap();

                // Load the category.
                let category = db::category::read(&connection, category_id);
                if category.is_none() {
                    let message = format!("Category with ID {} could not be loaded", category_id);
                    Err::<String, _>(message).unwrap_or_exit();
                };

                db::expense::create(
                    &establish_connection(&config.database_url()).unwrap_or_exit(),
                    &user,
                    &amount,
                    &category.unwrap(),
                    arguments.value_of("description"),
                    date.as_ref(),
                )
                .unwrap_or_exit();
            }
            ("get", Some(arguments)) => {
                let id = assert_integer_argument(arguments.value_of("id"), "expense ID").unwrap();
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                let expense = db::expense::read(&connection, id);
                if expense.is_none() {
                    Err::<String, _>("Expense not found").unwrap_or_exit();
                };
                println!("{}", json!(expense.unwrap()));
            }
            ("delete", Some(arguments)) => {
                let id = assert_integer_argument(arguments.value_of("id"), "expense ID").unwrap();
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
                db::expense::delete(&connection, id).unwrap_or_exit();
            }
            ("", None) => {}
            _ => unreachable!(),
        },
        ("notify", Some(notify)) => match notify.subcommand() {
            ("activate", Some(arguments)) => {
                let connection = establish_connection(&config.database_url()).unwrap_or_exit();
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

    // Checks that the given argument can be casted to an integer.
    fn assert_integer_argument(arg: Option<&str>, arg_type: &str) -> Option<i32> {
        let msg = format!("The {} must be an integer", arg_type);
        arg.map(|v| v.parse().map_err(|_| msg).unwrap_or_exit())
    }
}
