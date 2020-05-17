use crate::configure_application;

use actix_http::{body::Body, error::Error, Request};
use actix_service::Service;
use actix_web::{dev::ServiceResponse, test, App};
use app::AppConfig;

pub mod error;
pub mod homepage;
pub mod user;

/// Returns the Firetrack web application using the default test configuration.
pub async fn build_test_app(
) -> impl Service<Request = Request, Response = ServiceResponse<Body>, Error = Error> {
    // Import environment variables for the host, port, database credentials etc.
    dotenv::dotenv().ok();
    dotenv::from_filename(".env.dist").ok();

    let config = AppConfig::from_test_defaults();
    let database_url = config.database_url();
    let pool = db::create_test_connection_pool(database_url).unwrap();
    test::init_service(
        App::new().configure(|c| configure_application(c, pool.clone(), config.clone())),
    )
    .await
}
