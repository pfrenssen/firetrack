use crate::get_tera_context;
use actix_http::body::{Body, ResponseBody};
use actix_http::Response;
use actix_identity::RequestIdentity;
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::web::Data;
use actix_web::Result;
use tera::Tera;

/// Custom error handlers that show error messages as HTML pages.
pub fn error_handlers() -> ErrorHandlers<Body> {
    ErrorHandlers::new()
        .handler(StatusCode::FORBIDDEN, forbidden)
        .handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
// This conforms to an error handler signature. Ignore clippy warning that the Result is unneeded.
// Todo: Remove unknown_clippy_lints line when we are on Rust 1.50.0.
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::unnecessary_wraps)]
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let response = get_response(
        &res,
        "Page not found",
        "Sorry, this page does not exist",
        Some("We can't seem to find the page you're looking for."),
    );
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

// Error handler for a 403 Forbidden error.
// This conforms to an error handler signature. Ignore clippy warning that the Result is unneeded.
// Todo: Remove unknown_clippy_lints line when we are on Rust 1.50.0.
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::unnecessary_wraps)]
fn forbidden(res: ServiceResponse<Body>) -> Result<ErrorHandlerResponse<Body>> {
    let resp = res.response();
    let default_message = "Please log in and try again";
    let message = if let ResponseBody::Body(body) = resp.body() {
        // Convert the response in Bytes to a string slice.
        match body {
            Body::Bytes(b) => std::str::from_utf8(b).unwrap_or(default_message),
            _ => default_message,
        }
    } else {
        default_message
    };

    let response = get_response(&res, "Access denied", message, None);
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

fn get_response<B>(
    res: &ServiceResponse<B>,
    title: &str,
    message: &str,
    explanation: Option<&str>,
) -> Response<Body> {
    // Retrieve the current user identity from the request. Note that unlike route handlers this
    // does not return an `Identity` struct but rather the user email address as a string.
    let request = res.request();
    let identity = request.get_identity();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |m: &str| {
        Response::build(res.status())
            .content_type("text/plain")
            .body(m.to_string())
    };

    // Render the error page or fall back to a simple text message if Tera is not available.
    let tera = request.app_data::<Data<Tera>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let mut context = get_tera_context(title, identity);
            context.insert("body_classes", &vec!["error"]);
            context.insert("message", message);
            context.insert("explanation", &explanation);
            context.insert("status_code", res.status().as_str());
            let content = tera.render("error.html", &context);

            match content {
                Ok(content) => Response::build(res.status())
                    .content_type("text/html")
                    .body(content),
                Err(_) => fallback(message),
            }
        }
        None => fallback(message),
    }
}
