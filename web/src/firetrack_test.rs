use super::*;

use actix_http::body::{Body, ResponseBody};
use actix_web::http::StatusCode;
use libxml::{parser::Parser, xpath::Context};
use serde_json::json;
use std::str;

// Checks that the page returns a 200 OK response.
pub fn assert_response_ok(response: &HttpResponse) {
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "The HTTP response object has status 200 OK."
    );
}

// Checks that the page returns a 303 See Other response.
pub fn assert_response_see_other(response: &HttpResponse, location: &str) {
    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "The HTTP response object has status 303 See Other."
    );
    assert!(
        response.head().headers().get("location").is_some(),
        "The location header is set."
    );
    assert_eq!(
        response.head().headers().get("location").unwrap(),
        location,
        "The response is redirecting to the expected location."
    );
}

// Defines options for checking the content of a page.
pub struct PageAssertOptions {
    // Optional title to check. When omitted, the page title is not checked.
    pub title: Option<String>,
    // Whether this is an error page.
    pub is_error_page: bool,
}

impl PageAssertOptions {
    // Default values.
    pub fn default() -> PageAssertOptions {
        PageAssertOptions {
            title: None,
            is_error_page: false,
        }
    }
}

// Checks the contents of the given HTML page, depending on the given options.
pub fn assert_page(body: &str, ops: PageAssertOptions) {
    if let Some(title) = ops.title {
        assert_page_title(body, title.as_str());
    }

    // Error pages include an additional CSS file with styling.
    if ops.is_error_page {
        assert_stylesheet(body, "/css/error.css");
    } else {
        assert_no_stylesheet(body, "/css/error.css");
    }

    assert_page_header(body);
}

// Checks that the page title and main header match the given string.
pub fn assert_page_title(body: &str, title: &str) {
    let header_title = format!("Firetrack - {}", title);
    assert_xpath(body, "//head//title", header_title.as_str());

    // Check that there is only 1 <h1> title on the page.
    let xpath = "//body//h1";
    assert_xpath_result_count(body, xpath, 1);

    assert_xpath(body, xpath, title);
}

// Checks that the header elements are present.
pub fn assert_page_header(body: &str) {
    let expressions = [
        // The logo, linking to the homepage.
        "//body//aside[contains(concat(' ', normalize-space(@class), ' '), 'main-sidebar')]/a[@href='/']/img[@src='/images/logo.png']",
        // The site name, linking to the homepage.
        "//body//aside[contains(concat(' ', normalize-space(@class), ' '), 'main-sidebar')]/a[@href='/']/span[text()='Firetrack']",
        // The button to toggle the sidebar.
        "//body//nav[contains(concat(' ', normalize-space(@class), ' '), 'navbar')]/ul[@class='navbar-nav']/li[@class='nav-item']/a[@data-widget='pushmenu']",
        // The link to the registration page.
        "//body//nav[contains(concat(' ', normalize-space(@class), ' '), 'navbar')]//a[@href='/user/register']",
        // The link to the login page.
        "//body//nav[contains(concat(' ', normalize-space(@class), ' '), 'navbar')]//a[@href='/user/login']",
    ];

    for expression in &expressions {
        assert_xpath_result_count(body, expression, 1);
    }
}

// Checks that the form input element with the given attributes is present in the body.
pub fn assert_form_input(body: &str, id: &str, name: &str, input_type: &str, label: &str) {
    // Check for the label.
    let xpath = format!("//body//label[@for='{}' and text()='{}']", id, label);
    assert_xpath_result_count(body, xpath.as_str(), 1);

    // Check the input element.
    let xpath = format!(
        "//body//input[@id='{}' and @name='{}' and @type='{}']",
        id, name, input_type
    );
    assert_xpath_result_count(body, xpath.as_str(), 1);
}

// Checks that the form submit button with the given label is present.
pub fn assert_form_submit(body: &str, label: &str) {
    // Check the input element.
    let xpath = format!("//body//button[@type='submit' and text()='{}']", label);
    assert_xpath_result_count(body, xpath.as_str(), 1);
}

// Checks that the stylesheet with the given path is included.
pub fn assert_stylesheet(body: &str, path: &str) {
    let xpath = format!("//head/link[@rel='stylesheet' and @href='{}']", path);
    assert_xpath_result_count(body, xpath.as_str(), 1);
}

// Checks that the stylesheet with the given path is not included.
pub fn assert_no_stylesheet(body: &str, path: &str) {
    let xpath = format!("//head/link[@rel='stylesheet' and @href='{}']", path);
    assert_xpath_result_count(body, xpath.as_str(), 0);
}

// Given an HttpResponse, returns the response body as a string.
pub fn get_response_body(response: &HttpResponse) -> String {
    // Get the response body.
    let body = match response.body() {
        // This returns `Body` for regular route handlers, and `Other` for error handlers.
        ResponseBody::Body(b) => b,
        ResponseBody::Other(o) => o,
    };
    // Convert the response in Bytes to a string slice.
    let body = match body {
        Body::Bytes(b) => str::from_utf8(b).unwrap(),
        _ => "",
    };

    // Strip off the doctype declaration. This is invalid XML and prevents us from using XPath.
    body.to_string().replace("<!doctype html>\n", "")
}

// Asserts that the given XPath expression results in the given string for the given XML document.
fn assert_xpath(xml: &str, expression: &str, expected_result: &str) {
    // This should only be used for expressions that result in a single node.
    assert_xpath_result_count(xml, expression, 1);

    let parser = Parser::default();
    let doc = parser.parse_string(xml.as_bytes()).unwrap();
    let context = Context::new(&doc).unwrap();
    let result = context.evaluate(expression).unwrap();
    assert_eq!(expected_result, result.to_string());
}

// Asserts that an XPath expression returns an expected result count for a given document.
fn assert_xpath_result_count(xml: &str, expression: &str, expected_count: usize) {
    let parser = Parser::default();
    let doc = parser.parse_string(xml.as_bytes()).unwrap();
    let context = Context::new(&doc).unwrap();
    let result = context.evaluate(expression).unwrap();
    assert_eq!(
        expected_count,
        result.get_number_of_nodes(),
        "Expecting {} instances of {}",
        expected_count,
        expression
    );
}

// Sets up a Mailgun mock server that will respond positively to every request on its endpoint.
pub fn mailgun_mock(config: &AppConfig) -> mockito::Mock {
    // A mocked response that is returned by the Mailgun API for a valid notification request.
    let valid_response = json!({
        "id": format!("<0123456789abcdef.0123456789abcdef@{}>", config.mailgun_user_domain()),
        "message": "Queued. Thank you."
    });

    // Return a valid response for any request to the endpoint.
    let uri = notifications::get_mailgun_uri(&config);
    mockito::mock("POST", uri.as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(valid_response.to_string())
        .create()
}
