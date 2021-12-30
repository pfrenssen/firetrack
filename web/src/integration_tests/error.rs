use super::super::*;
use crate::integration_tests::build_test_app;
use actix_web::http::StatusCode;
use actix_web::{dev::Service, test};

// Integration test for the 404 Page Not Found error page.
#[actix_rt::test]
async fn test_404() {
    let mut app = build_test_app().await;

    let req = test::TestRequest::get()
        .uri("/non-existing-path")
        .to_request();

    let response = app.call(req).await.unwrap();
    let body = get_response_body(response.response());

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    assert_page(
        &body,
        PageAssertOptions {
            title: Some("Page not found".to_string()),
            is_error_page: true,
            has_sidebar: false,
            ..PageAssertOptions::default()
        },
    );
}
