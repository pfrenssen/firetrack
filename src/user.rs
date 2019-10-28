use super::schema::users;
use actix_web::{error, web, Error, HttpResponse};
use argonautica::Hasher;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::fmt;
use validator::validate_email;

#[derive(Debug, Queryable)]
pub struct User {
    pub email: String,
    pub password: String,
    pub created: chrono::NaiveDateTime,
    pub validated: bool,
}

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

// Possible errors being thrown when dealing with users.
#[derive(Debug, PartialEq)]
pub enum UserError {
    // The passed in email address is not valid.
    InvalidEmail(String),
    // The user password could not be hashed. This is usually due to a requirement not being met,
    // such as a missing password.
    PasswordHashFailed(argonautica::Error),
    // A new user could not be created due to a database error.
    UserCreationFailed(diesel::result::Error),
    // The user with the given email address does not exist.
    UserNotFound(String),
    // A user could not be read due to a database error.
    UserReadFailed(diesel::result::Error),
    // A new user could not be created because a user with the same email address has already been
    // registered.
    UserWithEmailAlreadyExists(String),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UserError::InvalidEmail(ref email) => write!(f, "Invalid email adress: {}", email),
            UserError::PasswordHashFailed(ref err) => write!(f, "Password hashing error: {}", err),
            UserError::UserCreationFailed(ref err) => {
                write!(f, "Database error when creating user: {}", err)
            }
            UserError::UserNotFound(ref email) => {
                write!(f, "The user with email {} does not exist", email)
            }
            UserError::UserReadFailed(ref err) => {
                write!(f, "Database error when reading user: {}", err)
            }
            UserError::UserWithEmailAlreadyExists(ref email) => {
                write!(f, "A user with email {} already exists", email)
            }
        }
    }
}

/// Creates a user.
pub fn create(
    connection: &PgConnection,
    email: &str,
    password: &str,
    secret: &str,
    memory_size: u32,
    iterations: u32,
) -> Result<User, UserError> {
    if !validate_email(email) {
        return Err(UserError::InvalidEmail(email.to_string()));
    }

    let existing_user = read(connection, email);
    if existing_user.is_ok() {
        return Err(UserError::UserWithEmailAlreadyExists(email.to_string()));
    }

    let hashed_password = hash_password(password, secret, memory_size, iterations)
        .map_err(UserError::PasswordHashFailed)?;

    diesel::insert_into(users::table)
        .values((
            users::email.eq(email),
            users::password.eq(hashed_password),
            users::created.eq(chrono::Local::now().naive_local()),
            users::validated.eq(false),
        ))
        .returning((
            users::email,
            users::password,
            users::created,
            users::validated,
        ))
        .get_result(connection)
        .map_err(UserError::UserCreationFailed)
}

// Performs an Argon2 hash of the password.
fn hash_password(
    password: &str,
    secret: &str,
    memory_size: u32,
    iterations: u32,
) -> Result<String, argonautica::Error> {
    Hasher::default()
        .configure_memory_size(memory_size)
        .configure_iterations(iterations)
        .with_password(password)
        .with_secret_key(secret)
        .hash()
}

/// Retrieves the user with the given email address from the database.
pub fn read(connection: &PgConnection, email: &str) -> Result<User, UserError> {
    use super::schema::users::dsl::users;
    let user = users.find(email).first::<User>(connection);
    match user {
        Ok(u) => Ok(u),
        Err(diesel::result::Error::NotFound) => Err(UserError::UserNotFound(email.to_string())),
        Err(e) => Err(UserError::UserReadFailed(e)),
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

    use crate::{establish_connection, import_env_vars};
    use actix_web::test::{block_on, TestRequest};
    use argonautica::Verifier;

    // Unit tests for the user login page.
    #[test]
    fn test_login() {
        dotenv::dotenv().ok();

        // Wrap the Tera struct in a HttpRequest and then retrieve it from the request as a Data struct.
        let tera = compile_templates!("templates/**/*");
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
        let tera = compile_templates!("templates/**/*");
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

    // Tests hash_password().
    #[test]
    fn test_hash_password() {
        // Use low values for the memory size and iterations to speed up testing.
        let memory_size = 512;
        let iterations = 1;

        let test_cases = [("mypass", "mysecret"), ("œ∑´®†¥¨ˆøπ“‘", "¡™£¢∞§¶•ªº–≠")];

        for test_case in &test_cases {
            let password = &test_case.0;
            let secret = &test_case.1;

            // Check that a hashed password is returned.
            let result = hash_password(password, secret, memory_size, iterations).unwrap();
            assert!(result.starts_with("$argon2id$"));

            // Check that the hashed password is valid.
            assert!(hashed_password_is_valid(result.as_str(), password, secret));

            // If we use a different password or key the result should be invalid.
            assert!(!hashed_password_is_valid(
                result.as_str(),
                "incorrect password",
                secret
            ));
            assert!(!hashed_password_is_valid(
                result.as_str(),
                password,
                "incorrect secret"
            ));
            assert!(!hashed_password_is_valid(
                result.as_str(),
                "incorrect password",
                "incorrect secret"
            ));
        }

        // Empty passwords are not allowed.
        let result = hash_password("", "mysecret", memory_size, iterations);
        assert!(result.is_err());

        // Iterations must be larger than 0.
        let result = hash_password("mypass", "mysecret", memory_size, 0);
        assert!(result.is_err());

        // Memory size must be at least 8x the number of cores in the machine.
        let result = hash_password("mypass", "mysecret", 7, iterations);
        assert!(result.is_err());
    }

    #[test]
    fn test_create() {
        import_env_vars();
        let connection = establish_connection();
        let email = "test@example.com";
        let password = "mypass";
        let secret = "mysecret";
        connection.test_transaction::<_, Error, _>(|| {
            let user = create(&connection, email, password, secret, 512, 1).unwrap();

            // Check that the user object is returned with the correct values.
            assert_eq!(user.email, email);
            assert!(hashed_password_is_valid(
                user.password.as_str(),
                password,
                secret
            ));
            assert_eq!(user.validated, false);

            // Check that the creation timestamp is located somewhere in the last few seconds.
            let now = chrono::Local::now().naive_local();
            let two_seconds_ago = chrono::Local::now()
                .checked_add_signed(time::Duration::seconds(-2))
                .unwrap()
                .naive_local();
            assert!(user.created < now);
            assert!(user.created > two_seconds_ago);

            // Creating a second user with the same email address should result in an error.
            let same_email_user =
                create(&connection, email, "some_other_password", secret, 512, 1).unwrap_err();
            assert_eq!(
                same_email_user,
                UserError::UserWithEmailAlreadyExists(email.to_string())
            );

            // The email address should be valid.
            let invalid_email = "invalid_email";
            let invalid_email_user =
                create(&connection, invalid_email, password, secret, 512, 1).unwrap_err();
            assert_eq!(
                invalid_email_user,
                UserError::InvalidEmail(invalid_email.to_string())
            );

            // The password should not be empty.
            let empty_password_user = create(&connection, "test2@example.com", "", secret, 512, 1);
            assert!(empty_password_user.is_err());

            Ok(())
        });
    }

    #[test]
    fn test_read() {
        import_env_vars();
        let connection = establish_connection();
        let email = "test@example.com";
        let password = "mypass";
        let secret = "mysecret";
        connection.test_transaction::<_, Error, _>(|| {
            create(&connection, email, password, secret, 512, 1).unwrap();

            // Check that the retrieved user object has the correct values.
            let user = read(&connection, email).unwrap();
            assert_eq!(user.email, email);
            assert!(hashed_password_is_valid(
                user.password.as_str(),
                password,
                secret
            ));
            assert_eq!(user.validated, false);

            // Check that the creation timestamp is located somewhere in the last few seconds.
            let now = chrono::Local::now().naive_local();
            let two_seconds_ago = chrono::Local::now()
                .checked_add_signed(time::Duration::seconds(-2))
                .unwrap()
                .naive_local();
            assert!(user.created < now);
            assert!(user.created > two_seconds_ago);

            // Retrieving a non-existing user should result in an error.
            let non_existing_email = "non-existing@example.com";
            let non_existing_user = read(&connection, non_existing_email).unwrap_err();
            assert_eq!(
                non_existing_user,
                UserError::UserNotFound(non_existing_email.to_string())
            );

            Ok(())
        });
    }

    // Checks that the given password hash matches the given password and secret key.
    fn hashed_password_is_valid(h: &str, p: &str, s: &str) -> bool {
        Verifier::default()
            .with_hash(h)
            .with_password(p)
            .with_secret_key(s)
            .verify()
            .unwrap()
    }
}
