#[macro_use]
extern crate clap;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tera;
extern crate time;

#[cfg(test)]
mod firetrack_test;
#[cfg(test)]
mod integration_tests;

#[cfg(test)]
use crate::firetrack_test::*;
#[cfg(test)]
use actix_web::test;

mod schema;
mod user;

use actix_files;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use clap::{AppSettings, Arg, SubCommand};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv;
use std::env;
use std::process::exit;

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

                user::create(
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

// Starts the web server on the given host address and port.
fn serve(host: &str, port: &str) {
    // Configure the application.
    let app = || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(app_config)
    };

    // Start the web server.
    let addr = format!("{}:{}", host, port);
    match HttpServer::new(app).bind(addr) {
        Ok(server) => {
            server.run().unwrap();
        }
        Err(e) => {
            error!("Failed to start web server on {}:{}", host, port);
            error!("{}", e.to_string());
            exit(1);
        }
    }
}

// Establishes a non-pooled database connection.
pub fn establish_connection() -> PgConnection {
    let database_url =
        env::var("DATABASE_URL").expect_or_exit("DATABASE_URL environment variable is not set.");

    match PgConnection::establish(&database_url) {
        Ok(value) => value,
        Err(e) => {
            error!("Could not connect to PostgreSQL.");
            error!("Error connecting to {}", database_url);
            error!("{}", e.to_string());
            exit(1);
        }
    }
}

// Controller for the homepage.
fn index(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    context.insert("title", &"Home");
    let content = template
        .render("index.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Unit tests for the homepage.
#[test]
fn test_index() {
    dotenv::dotenv().ok();

    // Wrap the Tera struct in a HttpRequest and then retrieve it from the request as a Data struct.
    let tera = compile_templates!("templates/**/*");
    let request = test::TestRequest::get().data(tera).to_http_request();
    let app_data = request.get_app_data().unwrap();

    // Pass the Data struct containing the Tera templates to the index() function. This mimics how
    // actix-web passes the data to the controller.
    let controller = index(app_data);
    let response = test::block_on(controller).unwrap();
    let body = get_response_body(&response);

    assert_response_ok(&response);
    assert_header_title(&body, "Home");
    assert_page_title(&body, "Home");
    assert_navbar(&body);
}

// Configure the application.
fn app_config(config: &mut web::ServiceConfig) {
    let tera = compile_templates!("templates/**/*");
    config.service(
        web::scope("")
            .data(tera)
            .service(actix_files::Files::new("/css", "static/css"))
            .service(actix_files::Files::new("/images", "static/images"))
            .service(actix_files::Files::new("/js", "static/js"))
            .route("/", web::get().to(index))
            .route("/user/login", web::get().to(user::login_handler))
            .route("/user/register", web::get().to(user::register_handler))
            .route("/user/register", web::post().to(user::register_submit)),
    );
}

// Imports environment variables by reading the .env files.
fn import_env_vars() {
    // Populate environment variables from the local `.env` file.
    dotenv::dotenv().ok();

    // Populate environment variables from the `.env.dist` file. This file contains sane defaults
    // as a fallback.
    dotenv::from_filename(".env.dist").ok();
}
