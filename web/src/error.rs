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
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let response = get_response(
        &res,
        "Page not found",
        "Sorry, this page does not seem to exist.",
    );
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

// Error handler for a 403 Forbidden error.
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

    let response = get_response(&res, "Access denied", message);
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

fn get_response<B>(res: &ServiceResponse<B>, title: &str, message: &str) -> Response<Body> {
    // Retrieve the current user identity from the request. Note that unlike route handlers this
    // does not return an `Identity` struct but rather the user email address as a string.
    let request = res.request();
    let identity = request.get_identity();

    // Render the error page or fall back to a simple text message if Tera is not available.
    let tera = request.app_data::<Data<Tera>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let mut context = get_tera_context(title, identity);
            context.insert("body_classes", &vec!["error"]);
            context.insert("message", message);
            context.insert("status_code", res.status().as_str());
            let content = tera.render("error.html", &context).map_err(|err| {
                actix_web::error::ErrorInternalServerError(format!("Template error: {:?}", err))
            });

            // Generate an HTML response.
            Response::build(res.status())
                .content_type("text/html")
                .body(content.unwrap())
        }
        None => {
            // Generate a text response.
            Response::build(res.status())
                .content_type("text/plain")
                .body(message.to_string())
        }
    }
}
