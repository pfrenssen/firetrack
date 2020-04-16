#[macro_use]
extern crate serde_derive;

#[cfg(test)]
mod firetrack_test;
#[cfg(test)]
mod integration_tests;

#[cfg(test)]
use crate::firetrack_test::*;

mod bootstrap_components;
mod user;

use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_session::CookieSession;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use app::AppConfig;
use std::env;

// Starts the web server on the host address and port as configured in the application.
pub async fn serve(config: AppConfig) -> Result<(), String> {
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
        Ok(server) => server.run().await.map_err(|e| e.to_string()),
        Err(e) => Err(format!(
            "Failed to start web server on {}:{} - {}",
            config.host(),
            config.port(),
            e.to_string()
        )),
    }
}

// Controller for the homepage.
async fn index(id: Identity, template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = get_tera_context(id);
    context.insert("title", &"Home");

    let content = template
        .render("index.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Returns a new Tera context object.
pub fn get_tera_context(id: Identity) -> tera::Context {
    let mut context = tera::Context::new();

    // Set a flag to indicate if the user is logged in.
    context.insert("authenticated", &id.identity().is_some());

    context
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
            // Todo: Allow to toggle the secure flag on both the session and identity providers.
            // Ref. https://github.com/pfrenssen/firetrack/issues/96
            .wrap(CookieSession::signed(&session_key).secure(false))
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&session_key)
                    .name("auth")
                    .secure(false),
            ))
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
            .route("/user/logout", web::get().to(user::logout_handler))
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
