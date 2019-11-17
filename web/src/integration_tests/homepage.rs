use super::super::*;

use actix_web::{dev::Service, http::StatusCode, test, App};

#[test]
fn access_homepage() {
    dotenv::dotenv().ok();
    dotenv::from_filename(".env.dist").ok();
    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = db::create_connection_pool(database_url.as_str()).unwrap();
    let mut app = test::init_service(
        App::new()
            .configure(|c| configure_application(c, pool, app::AppConfig::from_test_defaults())),
    );
    let req = test::TestRequest::get().uri("/").to_request();
    let response = test::block_on(app.call(req)).unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Call to '/' returns 200 OK."
    );
}
