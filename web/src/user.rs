use actix_session::Session;
use actix_web::{error, web, Error, HttpResponse};
use app::AppConfig;
use db::activation_code::ActivationCodeErrorKind;
use validator::validate_email;

// The form fields of the user form.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UserFormInput {
    email: String,
    password: String,
}

impl UserFormInput {
    pub fn new(email: String, password: String) -> UserFormInput {
        UserFormInput { email, password }
    }
}

// Whether the form fields of the user form are valid.
#[derive(Serialize, Deserialize)]
struct UserFormInputValid {
    form_is_validated: bool,
    email: bool,
    password: bool,
}

impl UserFormInputValid {
    // Instantiate a form validation struct.
    #[cfg(test)]
    pub fn new(form_is_validated: bool, email: bool, password: bool) -> UserFormInputValid {
        UserFormInputValid {
            form_is_validated,
            email,
            password,
        }
    }

    // Instantiate a form validation struct with default values.
    pub fn default() -> UserFormInputValid {
        UserFormInputValid {
            form_is_validated: false,
            email: true,
            password: true,
        }
    }

    // Returns whether the form is validated and found valid.
    pub fn is_valid(&self) -> bool {
        self.form_is_validated && self.email && self.password
    }
}

// Request handler for the login form.
pub async fn login_handler(tera: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    context.insert("title", &"Log in");

    let content = tera
        .render("user/login.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Request handler for a GET request on the registration form.
pub async fn register_handler(tera: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    // This returns the initial GET request for the registration form. The form fields are empty and
    // there are no validation errors.
    let input = UserFormInput::new("".to_string(), "".to_string());
    let validation_state = UserFormInputValid::default();
    render_register(tera, input, validation_state)
}

// Submit handler for the registration form.
pub async fn register_submit(
    session: Session,
    tera: web::Data<tera::Tera>,
    input: web::Form<UserFormInput>,
    pool: web::Data<db::ConnectionPool>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    // Validate the form input.
    let mut validation_state = UserFormInputValid::default();

    if !validate_email(&input.email) {
        validation_state.email = false;
    }

    if input.password.is_empty() {
        validation_state.password = false;
    }

    validation_state.form_is_validated = true;

    // If validation failed, show the form again with validation errors highlighted.
    if !validation_state.is_valid() {
        return render_register(tera, input.into_inner(), validation_state);
    }

    // Create the user account.
    let connection = pool.get().map_err(error::ErrorInternalServerError)?;
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
    tera: web::Data<tera::Tera>,
    input: UserFormInput,
    validation_state: UserFormInputValid,
) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    context.insert("title", &"Sign up");
    context.insert("input", &input);
    context.insert("valid", &validation_state);

    let content = tera
        .render("user/register.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
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
    session: Session,
    tera: web::Data<tera::Tera>,
    pool: web::Data<db::ConnectionPool>,
) -> Result<HttpResponse, Error> {
    // The email address is passed in the session by the registration / login form. Return an error
    // if it is not set or does not correspond with an existing, non-activated user.
    if let Some(email) = session.get::<String>("email").unwrap_or_else(|_| None) {
        let connection = pool.get().map_err(error::ErrorInternalServerError)?;
        if let Ok(user) = db::user::read(&connection, email.as_str()) {
            if !user.activated {
                let input = ActivationFormInput::new("".to_string());
                let validation_state = ActivationFormInputValid::default();
                return render_activate(tera, input, validation_state);
            }
        }
    }
    Err(error::ErrorUnauthorized(
        "Please log in before activating your account.",
    ))
}

// Submit handler for the activation form.
pub async fn activate_submit(
    session: Session,
    tera: web::Data<tera::Tera>,
    input: web::Form<ActivationFormInput>,
    pool: web::Data<db::ConnectionPool>,
) -> Result<HttpResponse, Error> {
    let activation_code = input.activation_code.clone();

    // Convenience functions for easily returning error messages.
    let validation_error = |message| {
        render_activate(
            tera,
            input.into_inner(),
            ActivationFormInputValid::invalid(message),
        )
    };
    let authorization_failed = || {
        Err(error::ErrorUnauthorized(
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
                    return Ok(HttpResponse::Ok()
                        .content_type("text/html")
                        .body("Your account has been activated. You can now log in."));
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
    tera: web::Data<tera::Tera>,
    input: ActivationFormInput,
    validation_state: ActivationFormInputValid,
) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    context.insert("title", &"Activate account");
    context.insert("input", &input);
    context.insert("validation", &validation_state);

    let content = tera
        .render("user/activate.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::firetrack_test::*;

    use actix_web::test::TestRequest;

    // Unit tests for the user login form handler.
    #[actix_rt::test]
    async fn test_login_handler() {
        dotenv::dotenv().ok();

        // Wrap the Tera struct in a HttpRequest and then retrieve it from the request as a Data struct.
        let tera = crate::compile_templates();
        let request = TestRequest::get().data(tera).to_http_request();
        let app_data_tera = request.app_data::<web::Data<tera::Tera>>().unwrap();

        // Pass the Data struct containing the Tera templates to the controller. This mimics how
        // actix-web passes the data to the controller.
        let controller = login_handler(app_data_tera.clone());
        let response = controller.await.unwrap();
        let body = get_response_body(&response);

        assert_response_ok(&response);
        assert_header_title(&body, "Log in");
        assert_page_title(&body, "Log in");
        assert_navbar(&body);
    }

    // Unit tests for the user registration form handler.
    #[actix_rt::test]
    async fn test_register_handler() {
        dotenv::dotenv().ok();

        // Wrap the Tera struct in a HttpRequest and then retrieve it from the request as a Data struct.
        let tera = crate::compile_templates();
        let request = TestRequest::get().data(tera).to_http_request();
        let app_data_tera = request.app_data::<web::Data<tera::Tera>>().unwrap();

        // Pass the Data struct containing the Tera templates to the controller. This mimics how
        // actix-web passes the data to the controller.
        let controller = register_handler(app_data_tera.clone());
        let response = controller.await.unwrap();
        let body = get_response_body(&response);

        assert_response_ok(&response);
        assert_header_title(&body, "Sign up");
        assert_page_title(&body, "Sign up");
        assert_navbar(&body);

        // Check that the email and password fields and submit button are present.
        assert_form_input(&body, "email", "email", "email", "Email address");
        assert_form_input(&body, "password", "password", "password", "Password");
        assert_form_submit(&body, "Sign up");
    }

    // Tests UserFormInputValid::is_valid().
    #[test]
    fn test_user_form_input_valid_is_valid() {
        let test_cases = [
            // Unvalidated forms are never valid.
            (UserFormInputValid::new(false, false, false), false),
            (UserFormInputValid::new(false, false, true), false),
            (UserFormInputValid::new(false, true, false), false),
            (UserFormInputValid::new(false, true, true), false),
            // Validated forms where one of the fields do not validate are invalid.
            (UserFormInputValid::new(true, false, false), false),
            (UserFormInputValid::new(true, false, true), false),
            (UserFormInputValid::new(true, true, false), false),
            // A validated form with valid fields is valid.
            (UserFormInputValid::new(true, true, true), true),
        ];

        for test_case in &test_cases {
            let validator = &test_case.0;
            let expected = test_case.1;
            assert_eq!(validator.is_valid(), expected);
        }
    }
}
