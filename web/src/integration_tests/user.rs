use super::super::*;
use actix_web::http::StatusCode;
use actix_web::{dev::Service, test, App};
use db::user::asserts::hashed_password_is_valid;

#[actix_rt::test]
async fn register_with_valid_data() {
    dotenv::dotenv().ok();
    dotenv::from_filename(".env.dist").ok();

    let config = app::AppConfig::from_test_defaults();

    let _mock = mailgun_mock(&config);

    let database_url = config.database_url();
    let pool = db::create_test_connection_pool(database_url).unwrap();
    let mut app = test::init_service(
        App::new().configure(|c| configure_application(c, pool.clone(), config.clone())),
    )
    .await;

    // Register with a valid email address and password.
    let email = "test@example.com";
    let password = "mypass";
    let payload = user::UserFormInput::new(email.to_string(), password.to_string());

    let req = test::TestRequest::post()
        .uri("/user/register")
        .set_form(&payload)
        .to_request();

    // We should get redirected to the activation form.
    let response = app.call(req).await.unwrap();
    assert_response_see_other(&response.response(), "/user/activate");

    // Check that a user with the given username and password exists in the database.
    let user = db::user::read(&pool.get().unwrap(), email).unwrap();

    assert_eq!(user.email, email);
    assert!(hashed_password_is_valid(
        user.password.as_str(),
        password,
        config.secret_key()
    ));
    assert_eq!(user.activated, false);

    let now = chrono::Local::now().naive_local();
    let two_seconds_ago = chrono::Local::now()
        .checked_add_signed(chrono::Duration::seconds(-2))
        .unwrap()
        .naive_local();
    assert!(user.created < now);
    assert!(user.created > two_seconds_ago);

    // Try to create the user a second time.
    // Todo This should not result in an error and should not disclose that the user exists.
    let req = test::TestRequest::post()
        .uri("/user/register")
        .set_form(&payload)
        .to_request();

    let response = app.call(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR,);

    let body = get_response_body(&response.response());
    assert_eq!(
        body.as_str(),
        "A user with email test@example.com already exists"
    );
}

// Integration tests for the user login form handler.
#[actix_rt::test]
async fn test_login_handler() {
    dotenv::dotenv().ok();
    dotenv::from_filename(".env.dist").ok();

    let config = app::AppConfig::from_test_defaults();
    let database_url = config.database_url();
    let pool = db::create_test_connection_pool(database_url).unwrap();
    let mut app = test::init_service(
        App::new().configure(|c| configure_application(c, pool.clone(), config.clone())),
    )
    .await;

    let req = test::TestRequest::get().uri("/user/login").to_request();

    let response = app.call(req).await.unwrap();
    let body = get_response_body(&response.response());

    assert_response_ok(&response.response());
    assert_header_title(&body, "Log in");
    assert_page_title(&body, "Log in");
    assert_navbar(&body);
}
