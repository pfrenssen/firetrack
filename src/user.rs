use actix_web::{error, web, Error, HttpResponse};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UserFormInput {
    email: String,
    password: String,
}

impl UserFormInput {
    pub fn new(email: String, password: String) -> UserFormInput {
        UserFormInput {
            email,
            password,
        }
    }
}

// Controller for the login form.
pub fn login(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    debug!("Request user login form");

    let mut context = tera::Context::new();
    context.insert("title", &"Log in");

    let content = template.render("user/login.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Controller for the registration form.
pub fn register(template: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    debug!("Request user registration form");

    let mut context = tera::Context::new();
    context.insert("title", &"Sign up");

    let content = template.render("user/register.html", &context)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// Submit handler for the registration form.
pub fn register_submit(template: web::Data<tera::Tera>, input: web::Form<UserFormInput>) -> Result<HttpResponse, Error> {
    debug!("Submit user registration form");

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("Your email is {} with password {}", input.email, input.password)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::firetrack_test::*;

    use actix_web::test::{block_on, TestRequest};
    use regex::Regex;

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
        let controller = login(app_data);
        let response = block_on(controller).unwrap();
        let body = get_response_body(&response);

        // Strip off the doctype declaration. This is invalid XML and prevents us from using XPath.
        let re = Regex::new(r"<!doctype html>").unwrap();
        let body = re.replace(body.as_str(), "");

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
        let controller = register(app_data);
        let response = block_on(controller).unwrap();
        let body = get_response_body(&response);

        // Strip off the doctype declaration. This is invalid XML and prevents us from using XPath.
        let re = Regex::new(r"<!doctype html>").unwrap();
        let body = re.replace(body.as_str(), "");

        assert_response_ok(&response);
        assert_header_title(&body, "Sign up");
        assert_page_title(&body, "Sign up");
        assert_navbar(&body);
    }

}
