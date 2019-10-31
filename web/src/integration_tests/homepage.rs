use super::super::*;

use actix_web::{dev::Service, http::StatusCode, test, App};

#[test]
fn access_homepage() {
    let mut app = test::init_service(App::new().configure(app_config));
    let req = test::TestRequest::get().uri("/").to_request();
    let response = test::block_on(app.call(req)).unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Call to '/' returns 200 OK."
    );
}
