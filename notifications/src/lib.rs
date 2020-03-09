use app::AppConfig;
use db::activation_code::ActivationCode;
use db::user::User;
use mailgun_v3::email::{send_with_request_builder, Message, MessageBody};
use mailgun_v3::{Credentials, EmailAddress, ReqError};
use reqwest::blocking::RequestBuilder;
use std::fmt;

// Mailgun API endpoint URI, copied from the private mailgun_v3::email::MESSAGES_ENDPOINT constant.
const MAILGUN_API_ENDPOINT_URI: &str = "messages";

// Mailgun API endpoint domain, copied from the private mailgun_v3::MAILGUN_API constant.
#[cfg(not(test))]
const MAILGUN_API_ENDPOINT_DOMAIN: &str = "https://api.mailgun.net/v3";

// Possible errors thrown when handling notifications.
#[derive(Debug)]
pub enum NotificationErrorKind {
    // The activation notification could not be sent due to a Mailgun error.
    ActivationNotificationNotSent(ReqError),
}

impl fmt::Display for NotificationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NotificationErrorKind::ActivationNotificationNotSent(ref err) => write!(
                f,
                "Mailgun error when sending activation notification: {}",
                err
            ),
        }
    }
}

pub fn activate(
    user: &User,
    activation_code: &ActivationCode,
    config: &AppConfig,
) -> Result<(), NotificationErrorKind> {
    let sender = EmailAddress::name_address(
        // Todo: Make sender name configurable.
        "Firetrack team",
        format!("{}@{}", config.mailgun_user(), config.mailgun_domain()).as_str(),
    );
    let recipient = EmailAddress::address(user.email.as_str());
    let body_text = format!("Activation code: {}", activation_code.code);
    let body = MessageBody::Text(body_text);
    let message = Message {
        to: vec![recipient],
        subject: format!("Activation code for {}", app::APPLICATION_NAME),
        body,
        ..Default::default()
    };

    let credentials = Credentials::new(config.mailgun_api_key(), config.mailgun_domain());
    let request_builder = get_request_builder(&config);
    send_with_request_builder(request_builder, &credentials, &sender, message)
        .map_err(NotificationErrorKind::ActivationNotificationNotSent)?;
    Ok(())
}

fn get_request_builder(config: &AppConfig) -> RequestBuilder {
    let url = get_mailgun_url(config);
    let client = reqwest::blocking::Client::new();
    client.post(&url)
}

// Returns the domain of the Mailgun API endpoint. In release builds this will return the Mailgun
// production endpoint, while in test builds it will return the domain of a mock server.
fn get_mailgun_domain() -> String {
    // Todo: Put in AppConfig?
    #[cfg(not(test))]
    let domain = MAILGUN_API_ENDPOINT_DOMAIN.to_string();

    #[cfg(test)]
    let domain = mockito::server_url();
    domain
}

// Returns the URI of the Mailgun API endpoint.
fn get_mailgun_uri(config: &AppConfig) -> String {
    let uri = format!("/{}/{}", config.mailgun_domain(), MAILGUN_API_ENDPOINT_URI);
    uri
}

// Returns the URL of the Mailgun API endpoint. In release builds this will return the Mailgun
// production URL, while in test builds it will return the URL of a mock endpoint.
fn get_mailgun_url(config: &AppConfig) -> String {
    let mut domain = get_mailgun_domain();
    let uri = get_mailgun_uri(config);

    // Strip trailing slash from the domain, since the URI already starts with a slash.
    if domain.ends_with('/') {
        domain.pop();
    }

    let url = format!("{}{}", domain, uri);
    url
}

#[test]
fn test_activate() {
    use mockito::Matcher;
    use serde_json::json;

    // Initialize test config.
    let config = AppConfig::from_test_defaults();

    // Create a test user.
    let user = User {
        activated: false,
        email: "testuser@example.com".to_string(),
        created: chrono::Local::now().naive_local(),
        password: "123456".to_string(),
    };

    // Create a test activation code.
    let activation_code = ActivationCode {
        email: "testuser@example.com".to_string(),
        code: 123_456,
        expiration_time: chrono::Local::now()
            .checked_add_signed(time::Duration::minutes(30))
            .unwrap()
            .naive_local(),
        attempts: 0,
    };

    // A mocked response that is returned by the Mailgun API for a valid notification request.
    let valid_response = json!({
        "id": format!("<0123456789abcdef.0123456789abcdef@{}>", config.mailgun_domain()),
        "message": "Queued. Thank you."
    });

    let uri = get_mailgun_uri(&config);

    // Set up mocked responses. Note that these are matched in reverse order, so the first mocked
    // response is returned only when none of the others match.

    // Return a 401 unauthorized if an invalid API key is passed. Note that this matches only
    // because the next response (which has precedence over this one and checks that the API key is
    // valid) _doesn't_ match. Mockito doesn't have negative matching so we handle it this way.
    let _m1 = mockito::mock("POST", uri.as_str())
        // The API key is passed as a base64 encoded basic authentication string.
        .match_header("authorization", Matcher::Any)
        .with_status(401)
        .create();

    // Unused response which matches on a valid API key, this allows the previously defined response
    // to match on invalid API keys.
    let _m2 = mockito::mock("POST", uri.as_str())
        // The API key is passed as a base64 encoded basic authentication string.
        .match_header(
            "authorization",
            format!(
                "Basic {}",
                base64::encode(format!("api:{}", config.mailgun_api_key()).as_bytes())
            )
            .as_str(),
        )
        .create();

    // Return a valid response if a request is received that contains all of the required data.
    let _m3 = mockito::mock("POST", uri.as_str())
        // The API key is passed as a base64 encoded basic authentication string.
        .match_header(
            "authorization",
            format!(
                "Basic {}",
                base64::encode(format!("api:{}", config.mailgun_api_key()).as_bytes())
            )
            .as_str(),
        )
        .match_body(Matcher::AllOf(vec![
            Matcher::UrlEncoded(
                "subject".to_string(),
                format!("Activation+code+for+{}", app::APPLICATION_NAME),
            ),
            Matcher::UrlEncoded(
                "from".to_string(),
                format!(
                    "Firetrack+team+<{}@{}>",
                    config.mailgun_user(),
                    config.mailgun_domain()
                ),
            ),
            Matcher::UrlEncoded(
                "text".to_string(),
                format!("Activation+code:+{}", activation_code.code),
            ),
            Matcher::UrlEncoded("to".to_string(), user.email.clone()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(valid_response.to_string())
        .create();

    // Test that a valid request for sending an activation email is made to the Mailgun API when
    // valid parameters are passed.
    assert!(activate(&user, &activation_code, &config).is_ok());

    // Test that an authentication error is returned when passing an invalid API key.
    let mut bad_config = config;
    bad_config.set_mailgun_api_key("invalid-api-key".to_string());
    // Todo: Check that this returns a `NotificationErrorKind::ActivationNotificationNotSent`.
    assert!(activate(&user, &activation_code, &bad_config).is_err());
}
