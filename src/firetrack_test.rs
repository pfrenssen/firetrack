use super::*;

use actix_http::body::{Body, ResponseBody};
use actix_web::http::StatusCode;
use libxml::{parser::Parser, xpath::Context};
use std::str;

// Checks that the page returns a 200 OK response.
pub fn assert_response_ok(response: &HttpResponse) {
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "The HTTP response object has status 200 OK."
    );
}

// Checks that the header title matches the given string.
pub fn assert_header_title(body: &str, title: &str) {
    let header_title = format!("Firetrack - {}", title);
    assert_xpath(body, "//head//title", header_title.as_str());
}

// Checks that the page title matches the given string.
pub fn assert_page_title(body: &str, title: &str) {
    let xpath = "//body//h1";

    // Check that there is only 1 <h1> title on the page.
    assert_xpath_result_count(body, xpath, 1);

    assert_xpath(body, xpath, title);
}

// Checks that the navbar is present.
pub fn assert_navbar(body: &str) {
    let expressions = [
        // The navbar itself.
        "//body//nav[contains(concat(' ', normalize-space(@class), ' '), 'navbar')]",
        // The logo, linking to the homepage.
        "//body//nav[contains(concat(' ', normalize-space(@class), ' '), 'navbar')]//a[@href='/']//img[@src='/images/logo.png']",
        // The site name, linking to the homepage.
        "//body//nav[contains(concat(' ', normalize-space(@class), ' '), 'navbar')]//a[@href='/']//h3[text()='Firetrack']",
        // The button to toggle the navbar.
        "//body//nav[contains(concat(' ', normalize-space(@class), ' '), 'navbar')]//button[@class='navbar-toggler']",
        // The link to the registration page.
        "//body//nav[contains(concat(' ', normalize-space(@class), ' '), 'navbar')]//a[@href='/user/register']",
        // The link to the login page.
        "//body//nav[contains(concat(' ', normalize-space(@class), ' '), 'navbar')]//a[@href='/user/login']",
    ];

    for expression in &expressions {
        assert_xpath_result_count(body, expression, 1);
    }
}

// Given an HttpResponse, returns the response body as a string.
pub fn get_response_body(response: &HttpResponse) -> String {
    // Get the response body.
    let body = match response.body() {
        // It is not clear to me under which circumstances this will return a 'Body' or a 'Response'
        // so let's print out some debugging info.
        ResponseBody::Body(b) => {
            println!("'Body' response body is returned.");
            b
        }
        ResponseBody::Other(o) => {
            println!("'Other' response body is returned.");
            o
        }
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
    assert_eq!(expected_count, result.get_number_of_nodes());
}
