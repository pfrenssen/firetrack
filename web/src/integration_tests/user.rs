use super::super::*;
use actix_web::{dev::Service, test, App};
use db::user::asserts::hashed_password_is_valid;

#[test]
fn register_with_valid_data() {
    dotenv::dotenv().ok();
    dotenv::from_filename(".env.dist").ok();
    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = db::create_test_connection_pool(database_url.as_str()).unwrap();
    let config = app::AppConfig::from_test_defaults();
    let mut app = test::init_service(
        App::new().configure(|c| configure_application(c, pool.clone(), config.clone())),
    );

    // Register with a valid email address and password.
    let email = "test@example.com";
    let password = "mypass";
    let payload = user::UserFormInput::new(email.to_string(), password.to_string());

    let req = test::TestRequest::post()
        .uri("/user/register")
        .set_form(&payload)
        .to_request();

    let response = test::block_on(app.call(req)).unwrap();
    assert_response_ok(&response.response());

    let body = get_response_body(&response.response());
    assert_eq!(body.as_str(), "Your account has been created successfully.");

    // Check that a user with the given username and password exists in the database.
    let user = db::user::read(&pool.get().unwrap(), email).unwrap();

    assert_eq!(user.email, email);
    assert!(hashed_password_is_valid(
        user.password.as_str(),
        password,
        config.secret_key()
    ));
    assert_eq!(user.validated, false);

    let now = chrono::Local::now().naive_local();
    let two_seconds_ago = chrono::Local::now()
        .checked_add_signed(time::Duration::seconds(-2))
        .unwrap()
        .naive_local();
    assert!(user.created < now);
    assert!(user.created > two_seconds_ago);
}
