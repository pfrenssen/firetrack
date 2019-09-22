use super::*;

use actix_http::body::{Body, ResponseBody};
use actix_web::{http::StatusCode, test};
use libxml::{parser::Parser, xpath::Context};
use regex::Regex;
use std::str;

// Unit tests for the main page.
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

    // Strip off the doctype declaration. This is invalid XML and prevents us from using XPath.
    let re = Regex::new(r"<!doctype html>").unwrap();
    let body = re.replace(body.as_str(), "");

    assert_response_ok(&response);
    assert_header_title(&body, "Home");
    assert_page_title(&body, "Home");
}

// Checks that the page returns a 200 OK response.
fn assert_response_ok(response: &HttpResponse) {
    assert_eq!(response.status(), StatusCode::OK, "The HTTP response object has status 200 OK.");
}

// Checks that the header title matches the given string.
fn assert_header_title(body: &str, title: &str) {
    let header_title = format!("Firetrack - {}", title);
    assert_xpath(body, "//head//title", header_title.as_str());
}

// Checks that the page title matches the given string.
fn assert_page_title(body: &str, title: &str) {
    let xpath = "//body//h1";

    // Check that there is only 1 <h1> title on the page.
    assert_xpath_result_count(body, xpath, 1);

    assert_xpath(body, xpath, title);
}

// Given an HttpResponse, returns the response body as a string.
fn get_response_body(response: &HttpResponse) -> String {
    // Get the response body.
    let body = match response.body() {
        // It is not clear to me under which circumstances this will return a 'Body' or a 'Response'
        // so let's print out some debugging info.
        ResponseBody::Body(b) => { println!("'Body' response body is returned."); b },
        ResponseBody::Other(o) => { println!("'Other' response body is returned."); o },
    };
    // Convert the response in Bytes to a string slice.
    let body = match body {
        Body::Bytes(b) => str::from_utf8(b).unwrap(),
        _ => "",
    };

    body.to_string()
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
