use actix_web::{error, web, Error, HttpResponse};
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
pub fn login_handler(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    context.insert("title", &"Log in");

    let content = template
        .render("user/login.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Request handler for a GET request on the registration form.
pub fn register_handler(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    // This returns the initial GET request for the registration form. The form fields are empty and
    // there are no validation errors.
    let input = UserFormInput::new("".to_string(), "".to_string());
    let validation_state = UserFormInputValid::default();
    render_register(template, input, validation_state)
}

// Submit handler for the registration form.
pub fn register_submit(
    template: web::Data<tera::Tera>,
    input: web::Form<UserFormInput>,
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
        return render_register(template, input.into_inner(), validation_state);
    }

    Ok(HttpResponse::Ok().content_type("text/plain").body(format!(
        "Your email is {} with password {}",
        input.email, input.password
    )))
}

// Renders the registration form, including validation errors.
fn render_register(
    template: web::Data<tera::Tera>,
    input: UserFormInput,
    validation_state: UserFormInputValid,
) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    context.insert("title", &"Sign up");
    context.insert("input", &input);
    context.insert("valid", &validation_state);

    let content = template
        .render("user/register.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::firetrack_test::*;

    use actix_web::test::{block_on, TestRequest};

    // Unit tests for the user login page.
    #[test]
    fn test_login() {
        dotenv::dotenv().ok();

        // Wrap the Tera struct in a HttpRequest and then retrieve it from the request as a Data struct.
        let tera = crate::compile_templates();
        let request = TestRequest::get().data(tera).to_http_request();
        let app_data = request.get_app_data().unwrap();

        // Pass the Data struct containing the Tera templates to the controller. This mimics how
        // actix-web passes the data to the controller.
        let controller = login_handler(app_data);
        let response = block_on(controller).unwrap();
        let body = get_response_body(&response);

        assert_response_ok(&response);
        assert_header_title(&body, "Log in");
        assert_page_title(&body, "Log in");
        assert_navbar(&body);
    }

    // Unit tests for the user registration page.
    #[test]
    fn test_register() {
        dotenv::dotenv().ok();

        // Wrap the Tera struct in a HttpRequest and then retrieve it from the request as a Data struct.
        let tera = crate::compile_templates();
        let request = TestRequest::get().data(tera).to_http_request();
        let app_data = request.get_app_data().unwrap();

        // Pass the Data struct containing the Tera templates to the controller. This mimics how
        // actix-web passes the data to the controller.
        let controller = register_handler(app_data);
        let response = block_on(controller).unwrap();
        let body = get_response_body(&response);

        assert_response_ok(&response);
        assert_header_title(&body, "Sign up");
        assert_page_title(&body, "Sign up");
        assert_navbar(&body);
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
