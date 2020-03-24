use super::super::*;
use actix_web::http::StatusCode;
use actix_web::{dev::Service, test, App};
use db::user::asserts::hashed_password_is_valid;
use serde_json::json;

#[actix_rt::test]
async fn register_with_valid_data() {
    dotenv::dotenv().ok();
    dotenv::from_filename(".env.dist").ok();

    let config = app::AppConfig::from_test_defaults();

    // Return a valid response if a request is received that contains all of the required data.
    // A mocked response that is returned by the Mailgun API for a valid notification request.
    let valid_response = json!({
        "id": format!("<0123456789abcdef.0123456789abcdef@{}>", config.mailgun_user_domain()),
        "message": "Queued. Thank you."
    });
    let uri = notifications::get_mailgun_uri(&config);
    let _m3 = mockito::mock("POST", uri.as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(valid_response.to_string())
        .create();
    // Todo: get db url from config.
    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = db::create_test_connection_pool(database_url.as_str()).unwrap();
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

    let response = app.call(req).await.unwrap();
    assert_response_see_other(&response.response(), &"/user/activate".to_string());

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
