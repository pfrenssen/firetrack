#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
mod firetrack_test;
#[cfg(test)]
mod integration_tests;

#[cfg(test)]
use crate::firetrack_test::*;
#[cfg(test)]
use actix_web::test;

mod bootstrap_components;
mod user;

use actix_session::CookieSession;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use app::AppConfig;
use std::env;
use std::process::exit;

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

// Starts the web server on the host address and port as configured in the application.
pub async fn serve(config: AppConfig) -> std::io::Result<()> {
    let pool = db::create_connection_pool(&config.database_url()).unwrap();
    let cloned_config = config.clone();

    // Configure the application.
    let app = move || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(|c| configure_application(c, pool.clone(), cloned_config.clone()))
    };

    // Start the web server.
    let addr = format!("{}:{}", config.host(), config.port());
    match HttpServer::new(app).bind(addr) {
        Ok(server) => server.run().await,
        Err(e) => {
            error!(
                "Failed to start web server on {}:{}",
                config.host(),
                config.port()
            );
            error!("{}", e.to_string());
            exit(1);
        }
    }
}

// Controller for the homepage.
async fn index(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    context.insert("title", &"Home");
    let content = template
        .render("index.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Configure the application.
pub fn configure_application(
    config: &mut web::ServiceConfig,
    pool: db::ConnectionPool,
    app_config: AppConfig,
) {
    let tera = compile_templates();
    let session_key = app_config.session_key();
    config.service(
        web::scope("")
            .data(tera)
            .data(pool)
            .data(app_config)
            .wrap(CookieSession::signed(&session_key).secure(false))
            .service(actix_files::Files::new("/css", "web/static/css/"))
            .service(actix_files::Files::new("/images", "web/static/images/"))
            .service(actix_files::Files::new("/js", "web/static/js/"))
            .service(actix_files::Files::new(
                "/third-party",
                "web/static/third-party/",
            ))
            .route("/", web::get().to(index))
            .route("/user/activate", web::get().to(user::activate_handler))
            .route("/user/activate", web::post().to(user::activate_submit))
            .route("/user/login", web::get().to(user::login_handler))
            .route("/user/login", web::post().to(user::login_submit))
            .route("/user/register", web::get().to(user::register_handler))
            .route("/user/register", web::post().to(user::register_submit)),
    );
}

// Compile the Tera templates.
fn compile_templates() -> tera::Tera {
    // Determine the path to the templates folder. This depends on whether we are running from the
    // root of the application (e.g. when launched using `cargo run`) or from the library folder
    // (e.g. when running tests).
    let path = if env::current_dir().unwrap().ends_with("web") {
        "templates/**/*"
    } else {
        "web/templates/**/*"
    };
    tera::Tera::new(path).unwrap()
}

#[cfg(test)]
// Unit tests for the homepage.
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_index() {
        dotenv::dotenv().ok();

        // Wrap the Tera struct in a HttpRequest and then retrieve it from the request as a Data struct.
        let tera = compile_templates();
        let request = test::TestRequest::get().data(tera).to_http_request();
        let app_data_tera = request.app_data::<web::Data<tera::Tera>>().unwrap();

        // Pass the Data struct containing the Tera templates to the index() function. This mimics how
        // actix-web passes the data to the controller.
        let controller = index(app_data_tera.clone());
        let response = controller.await.unwrap();
        let body = get_response_body(&response);

        assert_response_ok(&response);
        assert_header_title(&body, "Home");
        assert_page_title(&body, "Home");
        assert_navbar(&body);
    }
}
