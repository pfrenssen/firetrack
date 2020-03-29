#[macro_use]
extern crate log;

use app::AppConfig;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Request, Response, Server, StatusCode};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::SocketAddr;
use std::str::from_utf8;

// Starts the mock server on the port as configured in the application.
pub async fn serve(config: AppConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.mailgun_mock_server_port()));
    let service = make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(messages)) });
    Server::bind(&addr).serve(service).await?;
    Ok(())
}

// Mocks the `messages` command on the Mailgun API. Will always return a valid response, and will
// log request body to a file for use in tests.
async fn messages(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    // Retrieve the full body stream, decode it and write it to the log file.
    let full_body = hyper::body::to_bytes(req.into_body()).await?;
    let body_content = urlencoding::decode(from_utf8(&full_body).unwrap()).unwrap();

    let filename = "mailgun-mock-server.log";
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
        .unwrap_or_else(|_| panic!("Cannot open file {} for writing", filename));
    if let Err(e) = writeln!(file, "{}", body_content) {
        error!("Couldn't write to {}: {}", filename, e);
    }

    // Return a valid response.
    let response = json!({
        "id": "<0123456789abcdef.0123456789abcdef@sandbox0123456789abcdef0123456789abcdef.mailgun.org>",
        "message": "Queued. Thank you."
    });
    let http_response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(response.to_string()))
        .unwrap();
    Ok(http_response)
}
