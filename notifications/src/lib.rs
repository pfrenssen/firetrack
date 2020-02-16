use app::AppConfig;
use db::activation_code::ActivationCode;
use db::user::User;
use mailgun_v3::email::{send_email, Message, MessageBody};
use mailgun_v3::{Credentials, EmailAddress};

pub fn activate(user: &User, activation_code: &ActivationCode, config: &AppConfig) {
    let sender = EmailAddress::name_address(
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

    let result = send_email(&credentials, &sender, message);
    dbg!(result);
}
