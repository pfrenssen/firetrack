use super::super::*;

use crate::integration_tests::build_test_app;
use actix_web::{dev::Service, test};

#[actix_rt::test]
async fn access_homepage() {
    let mut app = build_test_app().await;
    let req = test::TestRequest::get().uri("/").to_request();
    let response = app.call(req).await.unwrap();

    assert_response_ok(&response.response());

    let body = get_response_body(&response.response());
    assert_page(
        &body,
        PageAssertOptions {
            title: Some("Home".to_string()),
            ..PageAssertOptions::default()
        },
    );
}
