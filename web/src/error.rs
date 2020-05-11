use crate::{compile_templates, get_tera_context};
use actix_http::{body::Body, Response};
use actix_identity::RequestIdentity;
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::Result;

/// Custom error handlers that show error messages as HTML pages.
pub fn error_handlers() -> ErrorHandlers<Body> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    // Retrieve the current user identity from the request. Note that unlike route handlers this
    // does not return an `Identity` struct but rather the user email address as a string.
    let request = res.request();
    let identity = request.get_identity();

    // Render the error page.
    let tera = compile_templates();
    let mut context = get_tera_context("Page not found", identity);
    context.insert("body_classes", &vec!["error"]);
    context.insert("message", "Sorry, this page does not seem to exist.");
    context.insert("status_code", res.status().as_str());
    let content = tera.render("error.html", &context).map_err(|err| {
        actix_web::error::ErrorInternalServerError(format!("Template error: {:?}", err))
    });

    // Generate an HTML response.
    let new_resp = Response::build(res.status())
        .content_type("text/html")
        .body(content.unwrap());

    Ok(ErrorHandlerResponse::Response(
        res.into_response(new_resp.into_body()),
    ))
}
