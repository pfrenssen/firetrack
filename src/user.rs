#[cfg(test)] use regex::Regex;
#[cfg(test)] use actix_web::test;
#[cfg(test)] use crate::firetrack_test::*;

use actix_web::{error, web, Error, HttpResponse};

// Controller for the login form.
pub fn login(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    debug!("Request user login form");

    let mut context = tera::Context::new();
    context.insert("title", &"Log in");

    let content = template.render("user/login.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Controller for the registration form.
pub fn register(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    debug!("Request user registration form");

    let mut context = tera::Context::new();
    context.insert("title", &"Sign up");

    let content = template.render("user/register.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Unit tests for the user pages.
#[test]
fn test_login() {
    dotenv::dotenv().ok();

    // Wrap the Tera struct in a HttpRequest and then retrieve it from the request as a Data struct.
    let tera = compile_templates!("templates/**/*");
    let request = test::TestRequest::get().data(tera).to_http_request();
    let app_data = request.get_app_data().unwrap();

    // Pass the Data struct containing the Tera templates to the index() function. This mimics how
    // actix-web passes the data to the controller.
    let controller = login(app_data);
    let response = test::block_on(controller).unwrap();
    let body = get_response_body(&response);

    // Strip off the doctype declaration. This is invalid XML and prevents us from using XPath.
    let re = Regex::new(r"<!doctype html>").unwrap();
    let body = re.replace(body.as_str(), "");

    assert_response_ok(&response);
    assert_header_title(&body, "Log in");
    assert_page_title(&body, "Log in");
    assert_navbar(&body);
}
