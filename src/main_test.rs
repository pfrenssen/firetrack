use super::*;

use actix_web::{http::StatusCode, test};

#[test]
fn test_index() {
    // Wrap the Tera struct in a HttpRequest and then retrieve it from the request as a Data struct.
    let tera = compile_templates!("templates/**/*");
    let request = test::TestRequest::get().data(tera).to_http_request();
    let data = request.get_app_data().unwrap();

    // Pass the Data struct containing the Tera templates to the index() function. This mimics how
    // actix-web passes the data to the controller.
    let response = test::block_on(index(data)).unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Unit test for index() returns a HTTP response object with status 200");
}
