use actix_web::{error, web, App, Error, HttpResponse, HttpServer};

pub fn login(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    debug!("Request user login form");
    let content = template.render("user/login.html", &tera::Context::new())
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}
