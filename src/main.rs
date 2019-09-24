#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tera;

#[cfg(test)]
mod firetrack_test;
#[cfg(test)]
mod integration_tests;

#[cfg(test)]
use crate::firetrack_test::*;
#[cfg(test)]
use actix_web::test;

mod user;

use actix_files;
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
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
            .wrap(middleware::Logger::default())
            .configure(app_config)
    };

    // Start the web server.
    let addr = format!("{}:{}", host, port);
    HttpServer::new(app).bind(addr).unwrap().run().unwrap();
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
            .route("/user/login", web::get().to(user::login))
            .route("/user/register", web::get().to(user::register))
            .route("/user/register", web::post().to(user::register_submit)),
    );
}
