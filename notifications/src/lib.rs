#[macro_use]
extern crate log;

use app::AppConfig;
use db::activation_code::{ActivationCode, ActivationCodeErrorKind};
use db::user::User;
use mailgun_v3::email::{async_impl::send_with_request_builder, Message, MessageBody};
use mailgun_v3::{Credentials, EmailAddress};
use reqwest::RequestBuilder;
use std::fmt;

// Mailgun API endpoint URI, copied from the private mailgun_v3::email::MESSAGES_ENDPOINT constant.
const MAILGUN_API_ENDPOINT_URI: &str = "messages";

// Errors that might occur when handling notifications.
#[derive(Debug, PartialEq)]
pub enum NotificationErrorKind {
    // The activation notification could not be delivered due to a Mailgun error.
    ActivationNotificationNotDelivered(String),
    // The activation notification could not be sent because the notification code is not valid.
    InvalidActivationCode(ActivationCodeErrorKind),
    // The user ID in the passed activation code did not match that from the passed user.
    WrongActivationCodeUser(i32, i32),
}

impl fmt::Display for NotificationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NotificationErrorKind::ActivationNotificationNotDelivered(ref err) => write!(
                f,
                "Mailgun error when attempting to deliver activation notification: {}",
                err
            ),
            NotificationErrorKind::InvalidActivationCode(ref err) => write!(
                f,
                "Activation mail could not be delivered due to an invalid activation code: {}",
                err
            ),
            NotificationErrorKind::WrongActivationCodeUser(ref user_id, ref activation_id) => write!(
                f,
                "Activation mail could not be delivered because the activation code is for the user with ID {} but the passed user ID is {}",
                activation_id,
                user_id
            ),
        }
    }
}

// Sends a activation mail containing the given activation code to the given user.
pub async fn activate(
    user: &User,
    activation_code: &ActivationCode,
    config: &AppConfig,
) -> Result<(), NotificationErrorKind> {
    // Sanity check: ensure that the activation code is valid.
    activation_code
        .validate()
        .map_err(NotificationErrorKind::InvalidActivationCode)?;

    // Sanity check: the user ID should match the one from the activation code.
    if user.id != activation_code.id {
        return Err(NotificationErrorKind::WrongActivationCodeUser(
            user.id,
            activation_code.id,
        ));
    }

    let sender = EmailAddress::name_address(
        // Todo: Make sender name configurable.
        "Firetrack team",
        format!(
            "{}@{}",
            config.mailgun_user_name(),
            config.mailgun_user_domain()
        )
        .as_str(),
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

    let credentials = Credentials::new(config.mailgun_api_key(), config.mailgun_user_domain());
    let request_builder = get_request_builder(&config);
    send_with_request_builder(request_builder, &credentials, &sender, message)
        .await
        .map_err(|err| {
            error!(
                "Mailgun error when attempting to deliver activation notification: {:?}",
                err
            );
            NotificationErrorKind::ActivationNotificationNotDelivered(err.to_string())
        })?;
    Ok(())
}

// Returns a reqwest request builder for a POST request to the Mailgun API endpoint.
fn get_request_builder(config: &AppConfig) -> RequestBuilder {
    let url = get_mailgun_url(config);
    let client = reqwest::Client::new();
    client.post(&url)
}

// Returns the domain of the Mailgun API endpoint. In release builds this will return the Mailgun
// production endpoint, while in test builds it will return the domain of a mock server.
fn get_mailgun_domain(_config: &AppConfig) -> String {
    #[cfg(not(test))]
    let domain = _config.mailgun_api_endpoint().to_string();

    #[cfg(test)]
    let domain = mockito::server_url();
    domain
}

// Returns the URI of the Mailgun API endpoint.
pub fn get_mailgun_uri(config: &AppConfig) -> String {
    let uri = format!(
        "/{}/{}",
        config.mailgun_user_domain(),
        MAILGUN_API_ENDPOINT_URI
    );
    uri
}

// Returns the URL of the Mailgun API endpoint. In release builds this will return the Mailgun
// production URL, while in test builds it will return the URL of a mock endpoint.
fn get_mailgun_url(config: &AppConfig) -> String {
    let mut domain = get_mailgun_domain(config);
    let uri = get_mailgun_uri(config);

    // Strip trailing slash from the domain, since the URI already starts with a slash.
    if domain.ends_with('/') {
        domain.pop();
    }

    let url = format!("{}{}", domain, uri);
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    // Tests sending activation notifications.
    async fn test_activate() {
        use mockito::Matcher;
        use serde_json::json;

        // Initialize test config.
        let config = AppConfig::from_test_defaults();

        // Create a test user.
        let user = get_user();

        // Create a test activation code.
        let activation_code = get_activation_code();

        // A mocked response that is returned by the Mailgun API for a valid notification request.
        let valid_response = json!({
            "id": format!("<0123456789abcdef.0123456789abcdef@{}>", config.mailgun_user_domain()),
            "message": "Queued. Thank you."
        });

        let uri = get_mailgun_uri(&config);

        // Set up mocked responses. Note that these are matched in the defined order, so the last
        // mocked response is returned only when none of the previous ones match.

        // Return a valid response if a request is received that contains all of the required data.
        let _m1 = mockito::mock("POST", uri.as_str())
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
                    format!("Activation code for {}", app::APPLICATION_NAME),
                ),
                Matcher::UrlEncoded(
                    "from".to_string(),
                    format!(
                        "Firetrack team <{}@{}>",
                        config.mailgun_user_name(),
                        config.mailgun_user_domain()
                    ),
                ),
                Matcher::UrlEncoded(
                    "text".to_string(),
                    format!("Activation code: {}", activation_code.code),
                ),
                Matcher::UrlEncoded("to".to_string(), user.email.clone()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(valid_response.to_string())
            .create();

        // Unused response which matches on a valid API key, this allows the following response to
        // match on invalid API keys.
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

        // Return a 401 unauthorized if an invalid API key is passed. Note that this matches only
        // because the previous response (which checks that the API key is valid) _doesn't_ match.
        // Mockito doesn't have negative matching so we handle it this way.
        let _m3 = mockito::mock("POST", uri.as_str())
            // The API key is passed as a base64 encoded basic authentication string.
            .match_header("authorization", Matcher::Any)
            .with_status(401)
            .create();

        // Test that a valid request for sending an activation email is made to the Mailgun API when
        // valid parameters are passed.
        assert!(activate(&user, &activation_code, &config).await.is_ok());

        // Test that an authentication error is returned when passing an invalid API key.
        let mut bad_config = config;
        bad_config.set_mailgun_api_key("invalid-api-key".to_string());
        // Todo: Check that this returns a `NotificationErrorKind::ActivationNotificationNotSent`.
        assert!(activate(&user, &activation_code, &bad_config)
            .await
            .is_err());
    }

    #[actix_rt::test]
    // Checks that an error is returned when trying to activate a user with an activation code for a
    // different user.
    async fn test_activate_wrong_user() {
        let user = get_user();

        let activation_code = ActivationCode {
            id: user.id + 1,
            ..get_activation_code()
        };

        assert_eq!(
            NotificationErrorKind::WrongActivationCodeUser(user.id, activation_code.id),
            activate(&user, &activation_code, &AppConfig::from_test_defaults())
                .await
                .unwrap_err()
        );
    }

    #[actix_rt::test]
    // Checks that an error is returned when trying to activate a user with an expired activation
    // code.
    async fn test_activate_expired() {
        let user = get_user();

        let activation_code = ActivationCode {
            expiration_time: chrono::Local::now()
                .checked_sub_signed(chrono::Duration::minutes(1))
                .unwrap()
                .naive_local(),
            ..get_activation_code()
        };

        assert_eq!(
            NotificationErrorKind::InvalidActivationCode(ActivationCodeErrorKind::Expired),
            activate(&user, &activation_code, &AppConfig::from_test_defaults())
                .await
                .unwrap_err()
        );
    }

    #[actix_rt::test]
    // Checks that an error is returned when trying to activate a user which has exceeded the
    // maximum number of attempts.
    async fn test_activate_max_attempts_exceeded() {
        let user = get_user();

        let activation_code = ActivationCode {
            attempts: 6,
            ..get_activation_code()
        };

        assert_eq!(
            NotificationErrorKind::InvalidActivationCode(
                ActivationCodeErrorKind::MaxAttemptsExceeded
            ),
            activate(&user, &activation_code, &AppConfig::from_test_defaults())
                .await
                .unwrap_err()
        );
    }

    // Returns a test user.
    fn get_user() -> User {
        User {
            id: 1,
            activated: false,
            email: "testuser@example.com".to_string(),
            created: chrono::Local::now().naive_local(),
            password: "123456".to_string(),
        }
    }

    // Returns a test activation code.
    fn get_activation_code() -> ActivationCode {
        ActivationCode {
            id: 1,
            code: 123_456,
            expiration_time: chrono::Local::now()
                .checked_add_signed(chrono::Duration::minutes(30))
                .unwrap()
                .naive_local(),
            attempts: 0,
        }
    }
}
