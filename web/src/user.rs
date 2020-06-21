use super::bootstrap_components::{Alert, AlertType};
use super::get_tera_context;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{error, web, Error, HttpResponse};
use app::AppConfig;
use db::activation_code::ActivationCodeErrorKind;
use db::user::UserErrorKind;
use diesel::PgConnection;
use validator::validate_email;

// The form fields of the user form.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UserForm {
    email: String,
    password: String,
}

impl UserForm {
    pub fn new(email: String, password: String) -> UserForm {
        UserForm { email, password }
    }
}

// Whether the form fields of the user form are valid.
#[derive(Serialize, Deserialize)]
struct UserFormValidation {
    form_is_validated: bool,
    email: bool,
    password: bool,
}

impl UserFormValidation {
    // Instantiate a form validation struct.
    #[cfg(test)]
    pub fn new(form_is_validated: bool, email: bool, password: bool) -> UserFormValidation {
        UserFormValidation {
            form_is_validated,
            email,
            password,
        }
    }

    // Instantiate a form validation struct with default values.
    pub fn default() -> UserFormValidation {
        UserFormValidation {
            form_is_validated: false,
            email: true,
            password: true,
        }
    }

    // Validates the user form when registering.
    pub fn validate_registration(input: &UserForm) -> UserFormValidation {
        let mut validation_state = UserFormValidation::default();

        if !validate_email(&input.email) {
            validation_state.email = false;
        }

        if input.password.is_empty() {
            validation_state.password = false;
        }

        validation_state.form_is_validated = true;
        validation_state
    }

    // Validates the user form when logging in.
    pub fn validate_login(
        connection: &PgConnection,
        config: &AppConfig,
        input: &UserForm,
    ) -> UserFormValidation {
        let mut validation_state = UserFormValidation::default();

        if input.email.is_empty()
            || input.password.is_empty()
            || db::user::verify_password(connection, &input.email, &input.password, config).is_err()
        {
            // To prevent enumeration attacks we treat a non-existing email as a wrong password.
            validation_state.password = false;
        }

        validation_state.form_is_validated = true;
        validation_state
    }

    // Returns whether the form is validated and found valid.
    pub fn is_valid(&self) -> bool {
        self.form_is_validated && self.email && self.password
    }
}

// Request handler for the login form.
pub async fn login_handler(
    id: Identity,
    session: Session,
    tera: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    assert_not_authenticated(&id)?;

    let input = UserForm::new("".to_string(), "".to_string());
    let validation_state = UserFormValidation::default();
    render_login(id, session, tera, input, validation_state)
}

// Submit handler for the login form.
pub async fn login_submit(
    session: Session,
    id: Identity,
    tera: web::Data<tera::Tera>,
    input: web::Form<UserForm>,
    pool: web::Data<db::ConnectionPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    assert_not_authenticated(&id)?;

    let connection = pool.get().map_err(error::ErrorInternalServerError)?;

    // Validate the form input.
    let validation_state = UserFormValidation::validate_login(&connection, &config, &input);

    // If validation failed, show the form again with validation errors highlighted.
    if !validation_state.is_valid() {
        return render_login(id, session, tera, input.into_inner(), validation_state);
    }

    // The user has been validated, create a session.
    start_session(id, input.email.to_owned())
}

// Initiates a session for the user with the given email and redirects to the homepage.
fn start_session(id: Identity, email: String) -> Result<HttpResponse, Error> {
    // Start the session.
    id.remember(email);

    // Redirect to the homepage, using HTTP 303 redirect which will execute the redirection as a GET
    // request.
    Ok(HttpResponse::SeeOther().header("location", "/").finish())
}

// Renders the login form.
// Todo Don't pass the session, keep the logic in the caller.
fn render_login(
    id: Identity,
    session: Session,
    tera: web::Data<tera::Tera>,
    input: UserForm,
    validation_state: UserFormValidation,
) -> Result<HttpResponse, Error> {
    let mut context = get_tera_context("Log in", id);
    context.insert("input", &input);
    context.insert("validation", &validation_state);

    // If the user is coming from the activation form, show a success message.
    if session
        .get::<bool>("account_activated")
        .unwrap_or_else(|_| None)
        .is_some()
    {
        let alert = Alert {
            alert_type: AlertType::Success,
            message: "Your account has been activated. You can now log in.".to_string(),
        };
        context.insert("alerts", &vec![alert]);

        // Remove the values from the session so this message won't show up again.
        session.remove("account_activated");
        session.remove("email");
    }

    let content = tera
        .render("user/login.html", &context)
        .map_err(|err| error::ErrorInternalServerError(format!("Template error: {:?}", err)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Request handler for logging out.
pub async fn logout_handler(id: Identity, session: Session) -> Result<HttpResponse, Error> {
    assert_authenticated(&id)?;

    id.forget();
    session.purge();

    // Todo: show a temporary success message "You have been logged out".
    Ok(HttpResponse::SeeOther().header("location", "/").finish())
}

// Request handler for a GET request on the registration form.
pub async fn register_handler(
    id: Identity,
    tera: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    assert_not_authenticated(&id)?;

    // This returns the initial GET request for the registration form. The form fields are empty and
    // there are no validation errors.
    let input = UserForm::new("".to_string(), "".to_string());
    let validation_state = UserFormValidation::default();
    render_register(id, tera, input, validation_state)
}

// Submit handler for the registration form.
pub async fn register_submit(
    session: Session,
    id: Identity,
    tera: web::Data<tera::Tera>,
    input: web::Form<UserForm>,
    pool: web::Data<db::ConnectionPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    assert_not_authenticated(&id)?;

    // Validate the form input.
    let validation_state = UserFormValidation::validate_registration(&input);

    // If validation failed, show the form again with validation errors highlighted.
    if !validation_state.is_valid() {
        return render_register(id, tera, input.into_inner(), validation_state);
    }

    // Create the user account.
    let connection = pool.get().map_err(error::ErrorInternalServerError)?;
    let result = db::user::create(&connection, &input.email, &input.password, &config);
    match result {
        Err(UserErrorKind::UserWithEmailAlreadyExists(_)) => {
            return if db::user::verify_password(&connection, &input.email, &input.password, &config).is_ok() {
                start_session(id, input.email.to_owned())
            } else {
                Err(format!("email {} already exists but password is incorrect. Ref https://github.com/pfrenssen/firetrack/issues/68", input.email)).map_err(error::ErrorInternalServerError)
            }
        },
        _ => {}
    }
    let user = db::user::create(&connection, &input.email, &input.password, &config)
        .map_err(error::ErrorInternalServerError)?;

    // Send an activation email.
    let activation_code =
        db::activation_code::get(&connection, &user).map_err(error::ErrorInternalServerError)?;
    notifications::activate(&user, &activation_code, &config)
        .await
        .map_err(error::ErrorInternalServerError)?;

    // Pass the email address to the activation form by setting it on the session.
    session
        .set("email", user.email.as_str())
        .map_err(error::ErrorInternalServerError)?;

    // Redirect to the activation form, using HTTP 303 redirect which will execute the redirection
    // as a GET request.
    Ok(HttpResponse::SeeOther()
        .header("location", "/user/activate")
        .finish())
}

// Renders the registration form, including validation errors.
fn render_register(
    id: Identity,
    tera: web::Data<tera::Tera>,
    input: UserForm,
    validation_state: UserFormValidation,
) -> Result<HttpResponse, Error> {
    let mut context = get_tera_context("Sign up", id);
    context.insert("input", &input);
    context.insert("validation", &validation_state);

    let content = tera
        .render("user/register.html", &context)
        .map_err(|err| error::ErrorInternalServerError(format!("Template error: {:?}", err)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// The form fields of the activation form.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ActivationFormInput {
    activation_code: String,
}

impl ActivationFormInput {
    pub fn new(activation_code: String) -> ActivationFormInput {
        ActivationFormInput { activation_code }
    }
}

// Whether the form fields of the activation form are valid.
#[derive(Serialize, Deserialize)]
struct ActivationFormInputValid {
    // Whether or not the form input has been validated.
    form_is_validated: bool,
    // Whether or not the activation code is valid.
    activation_code: bool,
    // The validation message to show to the user.
    message: String,
}

impl ActivationFormInputValid {
    // Instantiate a form validation struct with default values.
    pub fn default() -> ActivationFormInputValid {
        ActivationFormInputValid {
            form_is_validated: false,
            activation_code: true,
            message: "".to_string(),
        }
    }

    // Instantiate a form validation struct with a validation error.
    pub fn invalid(message: &str) -> ActivationFormInputValid {
        ActivationFormInputValid {
            form_is_validated: true,
            activation_code: false,
            message: message.to_string(),
        }
    }
}

// Request handler for the activation form. This returns the initial GET request for the activation
// form. The form fields are empty and there are no validation errors.
pub async fn activate_handler(
    id: Identity,
    session: Session,
    tera: web::Data<tera::Tera>,
    pool: web::Data<db::ConnectionPool>,
) -> Result<HttpResponse, Error> {
    assert_not_authenticated(&id)?;

    // The email address is passed in the session by the registration / login form. Return an error
    // if it is not set or does not correspond with an existing, non-activated user.
    if let Some(email) = session.get::<String>("email").unwrap_or_else(|_| None) {
        let connection = pool.get().map_err(error::ErrorInternalServerError)?;
        if let Ok(user) = db::user::read(&connection, email.as_str()) {
            if !user.activated {
                let input = ActivationFormInput::new("".to_string());
                let validation_state = ActivationFormInputValid::default();
                return render_activate(id, tera, input, validation_state);
            }
        }
    }
    Err(error::ErrorForbidden(
        "Please log in before activating your account.",
    ))
}

// Submit handler for the activation form.
pub async fn activate_submit(
    id: Identity,
    session: Session,
    tera: web::Data<tera::Tera>,
    input: web::Form<ActivationFormInput>,
    pool: web::Data<db::ConnectionPool>,
) -> Result<HttpResponse, Error> {
    assert_not_authenticated(&id)?;

    let activation_code = input.activation_code.clone();

    // Convenience functions for easily returning error messages.
    let validation_error = |message| {
        render_activate(
            id,
            tera,
            input.into_inner(),
            ActivationFormInputValid::invalid(message),
        )
    };
    let authorization_failed = || {
        Err(error::ErrorForbidden(
            "Please log in before activating your account.",
        ))
    };

    // Check if the activation code is a 6 digit number.
    if !regex::Regex::new(r"^\d{6}$")
        .map_err(error::ErrorInternalServerError)?
        .is_match(activation_code.as_str())
    {
        return validation_error("Please enter a 6-digit number");
    }

    // Convert the user input to an integer. We know that the input is a 6 digit number, so we can
    // assume that the conversion will succeed, and return a 500 in the case that somehow doesn't.
    let activation_code: i32 = activation_code
        .parse()
        .map_err(error::ErrorInternalServerError)?;

    // Load the user from the email that is stored in the session.
    if let Some(email) = session.get::<String>("email").unwrap_or_else(|_| None) {
        let connection = pool.get().map_err(error::ErrorInternalServerError)?;
        if let Ok(user) = db::user::read(&connection, email.as_str()) {
            match db::activation_code::activate_user(&connection, user, activation_code) {
                Err(ActivationCodeErrorKind::Expired) => {
                    return validation_error("The expiration code has expired. Please re-send the activation email and try again.");
                }
                Err(ActivationCodeErrorKind::UserAlreadyActivated(_)) => {
                    // In order to not disclose which email addresses are registered we treat this
                    // the same as a non-existing user trying to access the form.
                    return authorization_failed();
                }
                Err(ActivationCodeErrorKind::MaxAttemptsExceeded) => {
                    return validation_error("You have exceeded the maximum number of activation attempts. Please try again later.");
                }
                Err(ActivationCodeErrorKind::InvalidCode) => {
                    return validation_error("Incorrect activation code. Please try again.");
                }
                Err(e) => {
                    return Err(error::ErrorInternalServerError(e));
                }
                Ok(_) => {
                    // Activation succeeded. Set a flag on the session and redirect to the login
                    // page using a HTTP 303 redirect which will issue a GET request.
                    session
                        .set("account_activated", true)
                        .map_err(error::ErrorInternalServerError)?;
                    return Ok(HttpResponse::SeeOther()
                        .header("location", "/user/login")
                        .finish());
                }
            }
        }
    }

    // No user passed in the session, or the passed user doesn't exist. Do not authorize the usage
    // of this form.
    authorization_failed()
}

// Renders the activation form.
fn render_activate(
    id: Identity,
    tera: web::Data<tera::Tera>,
    input: ActivationFormInput,
    validation_state: ActivationFormInputValid,
) -> Result<HttpResponse, Error> {
    let mut context = get_tera_context("Activate account", id);
    context.insert("input", &input);
    context.insert("validation", &validation_state);

    let content = tera
        .render("user/activate.html", &context)
        .map_err(|err| error::ErrorInternalServerError(format!("Template error: {:?}", err)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Checks that the user is not authenticated. Used to control access on login and registration
// forms.
fn assert_not_authenticated(id: &Identity) -> Result<(), Error> {
    if id.identity().is_some() {
        return Err(error::ErrorForbidden("You are already logged in."));
    }
    Ok(())
}

// Checks that the user is authenticated.
fn assert_authenticated(id: &Identity) -> Result<(), Error> {
    if id.identity().is_none() {
        return Err(error::ErrorForbidden(
            "You need to be logged in to access this page.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests UserFormInputValid::is_valid().
    #[test]
    fn test_user_form_input_valid_is_valid() {
        let test_cases = [
            // Unvalidated forms are never valid.
            (UserFormValidation::new(false, false, false), false),
            (UserFormValidation::new(false, false, true), false),
            (UserFormValidation::new(false, true, false), false),
            (UserFormValidation::new(false, true, true), false),
            // Validated forms where one of the fields do not validate are invalid.
            (UserFormValidation::new(true, false, false), false),
            (UserFormValidation::new(true, false, true), false),
            (UserFormValidation::new(true, true, false), false),
            // A validated form with valid fields is valid.
            (UserFormValidation::new(true, true, true), true),
        ];

        for test_case in &test_cases {
            let validator = &test_case.0;
            let expected = test_case.1;
            assert_eq!(validator.is_valid(), expected);
        }
    }
}
