#[macro_use]
extern crate tera;

#[macro_use]
extern crate log;

#[cfg(test)] mod main_test;
#[cfg(test)] mod integration_tests;

mod user;

use actix_files;
use actix_web::{error, web, App, Error, HttpResponse, HttpServer};
use dotenv;
use std::env;
use std::process::exit;

fn main() {
    // Populate environment variables from the local `.env` file.
    dotenv::dotenv().ok();

    // Populate environment variables from the `.env.dist` file. This file contains sane defaults
    // as a fallback.
    dotenv::from_filename(".env.dist").ok();

    // Initialize the logger.
    env_logger::init();

    // Retrieve the hostname and port.
    let host = match env::var("HOST") {
        Ok(value) => value,
        Err(_) => {
            error!("HOST environment variable is not set.");
            exit(1);
        }
    };
    let port = match env::var("PORT") {
        Ok(value) => value,
        Err(_) => {
            error!("PORT environment variable is not set.");
            exit(1);
        }
    };

    // Configure the application.
    let app = || {
        App::new()
            .configure(app_config)
    };

    // Start the web server.
    let addr = format!("{}:{}", host, port);
    HttpServer::new(app)
        .bind(addr)
        .unwrap()
        .run()
        .unwrap();
}

fn index(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    context.insert("title", &"Home");
    let content = template.render("index.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
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
            .route("/user/login", web::get().to(user::login))
    );
}
