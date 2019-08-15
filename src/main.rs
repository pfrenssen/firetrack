#[macro_use]
extern crate tera;

#[macro_use]
extern crate log;

#[cfg(test)] mod tests;

use actix_files;
use actix_web::{error, web, App, Error, HttpResponse, HttpServer};

fn main() {
    env_logger::init();
    debug!("Debug");
    info!("Info");
    warn!("Warning");
    error!("Error");

    // Configure the application.
    let app = || {
        App::new()
            .configure(app_config)
    };

    // Start the web server.
    HttpServer::new(app)
        .bind("127.0.0.1:8088")
        .unwrap()
        .run()
        .unwrap();
}

fn index(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let content = template.render("index.html", &tera::Context::new())
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Configure the application.
fn app_config(cfg: &mut web::ServiceConfig) {
    let tera = compile_templates!("templates/**/*");
    cfg.service(
        web::scope("")
            .data(tera)
            .service(actix_files::Files::new("/css", "static/css"))
            .service(actix_files::Files::new("/images", "static/images"))
            .service(actix_files::Files::new("/js", "static/js"))
            .route("/", web::get().to(index))
    );
}
