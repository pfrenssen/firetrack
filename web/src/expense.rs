use super::{assert_authenticated, get_tera_context};
use crate::category::CategoryDropdownItems;

use crate::bootstrap_components::{Alert, AlertType};
use actix_identity::Identity;
use actix_web::{error, web, Error, HttpResponse};
use chrono::Utc;
use db::category::{get_categories_tree, Category};
use db::expense::create;
use db::user::User;
use diesel::PgConnection;
use rust_decimal::Decimal;
use std::str::FromStr;

// The POST data of the add expense form.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AddForm {
    amount: String,
    category: String,
    date: String,
}

impl AddForm {
    pub fn new(amount: &str, category: &str, date: &str) -> AddForm {
        AddForm {
            amount: amount.to_string(),
            category: category.to_string(),
            date: date.to_string(),
        }
    }

    // Resets the form input so it is ready for entering the next expense. This is intended to be
    // called after successfully saving an expense. The date and category are kept intact so that
    // multiple related expenses can be entered conveniently.
    pub fn reset(&self) -> AddForm {
        AddForm {
            amount: "".to_string(),
            category: self.category.clone(),
            date: self.date.clone(),
        }
    }
}

// Whether the form fields of the add expense form are valid.
#[derive(Serialize, Deserialize, Debug)]
struct AddFormValidation {
    form_is_validated: bool,
    amount: Result<Decimal, String>,
    category: Result<Category, String>,
    date: Result<chrono::NaiveDate, String>,
}

impl AddFormValidation {
    #[cfg(test)]
    pub fn new(
        form_is_validated: bool,
        amount: Result<Decimal, String>,
        category: Result<Category, String>,
        date: Result<chrono::NaiveDate, String>,
    ) -> AddFormValidation {
        AddFormValidation {
            form_is_validated,
            amount,
            category,
            date,
        }
    }

    #[cfg(test)]
    pub fn is_valid(&self) -> bool {
        self.form_is_validated && self.category.is_ok() && self.date.is_ok() && self.amount.is_ok()
    }

    // Instantiate a form validation struct with default values.
    pub fn default() -> AddFormValidation {
        AddFormValidation {
            form_is_validated: false,
            amount: Err("Not validated".to_string()),
            category: Err("Not validated".to_string()),
            date: Err("Not validated".to_string()),
        }
    }

    // Validates the add expense form.
    pub fn validate(input: &AddForm, user: &User, connection: &PgConnection) -> AddFormValidation {
        let mut validation_state = AddFormValidation::default();

        // Validate the amount.
        if input.amount.is_empty() {
            validation_state.amount = Err("Please enter an amount.".to_string());
        } else {
            validation_state.amount = match Decimal::from_str(input.amount.as_str()) {
                Err(_) => Err("Amount should be in the format '149.99'.".to_string()),
                Ok(amount) if amount < Decimal::new(1, 2) => {
                    Err("Amount should be 0.01 or greater.".to_string())
                }
                Ok(amount) if amount > Decimal::new(999_999_999, 2) => {
                    Err("Amount should be 9999999.99 or smaller.".to_string())
                }
                Ok(amount) => Ok(amount),
            }
        }

        // Validate the category.
        if input.category.is_empty() {
            validation_state.category = Err("Please choose a category.".to_string());
        } else {
            validation_state.category = match input.category.parse::<i32>() {
                Err(_) => Err("Invalid category ID.".to_string()),
                Ok(id) => match db::category::read(connection, id, Some(user.id)) {
                    Some(cat) if cat.user_id == user.id => Ok(cat),
                    _ => Err("Unknown category.".to_string()),
                },
            }
        }

        // Validate the date.
        if input.date.is_empty() {
            validation_state.date = Err("Please pick a date.".to_string());
        } else {
            validation_state.date =
                match chrono::NaiveDate::parse_from_str(input.date.as_str(), "%Y-%m-%d") {
                    Err(_) => Err("Date should be in the format YYYY-MM-DD.".to_string()),
                    Ok(date) => Ok(date),
                }
        }

        validation_state.form_is_validated = true;
        validation_state
    }

    // Resets the form state so it is ready for entering the next expense. This is intended to be
    // called after successfully saving an expense. The date and category are kept intact so that
    // multiple related expenses can be entered conveniently.
    pub fn reset(&self) -> AddFormValidation {
        AddFormValidation {
            form_is_validated: false,
            amount: Err("Not validated".to_string()),
            category: self.category.clone(),
            date: self.date.clone(),
        }
    }
}

// Request handler for the expenses overview.
pub async fn overview_handler(
    id: Identity,
    template: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    assert_authenticated(&id)?;

    let context = get_tera_context("Expenses", id);

    let content = template
        .render("expenses/overview.html", &context)
        .map_err(|err| error::ErrorInternalServerError(format!("Template error: {:?}", err)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

// GET request handler for the form to add an expense.
pub async fn add_handler(
    id: Identity,
    pool: web::Data<db::ConnectionPool>,
    template: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    let today = Utc::now().naive_utc().date().format("%Y-%m-%d").to_string();
    let input = AddForm::new("", "", today.as_str());
    let validation_state = AddFormValidation::default();
    let alerts = vec![];

    render_add(id, pool, template, input, validation_state, alerts)
}

// POST Submit handler for the form to add an expense.
pub async fn add_submit(
    id: Identity,
    pool: web::Data<db::ConnectionPool>,
    template: web::Data<tera::Tera>,
    input: web::Form<AddForm>,
) -> Result<HttpResponse, Error> {
    let email = assert_authenticated(&id)?;

    let connection = pool.get().map_err(error::ErrorInternalServerError)?;
    let user =
        db::user::read(&connection, email.as_str()).map_err(error::ErrorInternalServerError)?;

    let input = input.into_inner();
    let validation_state = AddFormValidation::validate(&input, &user, &connection);

    // Create the expense if the form validates and return a success or failure alert. If the form
    // doesn't validate, don't set an alert since the user will already be notified about invalid
    // values through the form feedback messages.
    let (input, validation_state, alerts): (AddForm, AddFormValidation, Vec<Alert>) = match (
        validation_state.form_is_validated,
        &validation_state.amount,
        &validation_state.category,
        &validation_state.date,
    ) {
        (true, Ok(amount), Ok(category), Ok(date)) => {
            let (input, validation_state, alert) =
                match create(&connection, &user, amount, category, None, Some(date)) {
                    Ok(_) => {
                        (
                            // The expense was saved successfully. Reset the form state so the next
                            // expense can be entered. Keep the date and category intact so that
                            // multiple related expenses can be entered conveniently.
                            input.reset(),
                            validation_state.reset(),
                            Alert {
                                alert_type: AlertType::Success,
                                message: format!(
                                    "Successfully added €{:.2} expense to the {} category.",
                                    amount, category.name
                                ),
                            },
                        )
                    }
                    Err(e) => (
                        input,
                        validation_state,
                        Alert {
                            alert_type: AlertType::Danger,
                            message: format!("Error: {}", e),
                        },
                    ),
                };
            (input, validation_state, vec![alert])
        }
        _ => (input, validation_state, vec![]),
    };

    let input = AddForm::new("", input.category.as_str(), input.date.as_str());

    render_add(id, pool, template, input, validation_state, alerts)
}

// Renders the form to add an expense. Used by both GET and POST requests.
fn render_add(
    id: Identity,
    pool: web::Data<db::ConnectionPool>,
    template: web::Data<tera::Tera>,
    input: AddForm,
    validation_state: AddFormValidation,
    alerts: Vec<Alert>,
) -> Result<HttpResponse, Error> {
    let email = assert_authenticated(&id)?;

    // Retrieve the categories for the current user.
    let connection = pool.get().map_err(error::ErrorInternalServerError)?;
    let user =
        db::user::read(&connection, email.as_str()).map_err(error::ErrorInternalServerError)?;
    let categories =
        get_categories_tree(&connection, &user).map_err(error::ErrorInternalServerError)?;

    let categories_dropdown_items = CategoryDropdownItems::from(categories);

    // Convert the category provided by the form input to an integer so we can select the chosen
    // category in the dropdown. Tera cannot compare two values of different types and doesn't
    // support type casting
    let current_category_id: Option<i32> = input.category.parse().ok();

    let mut context = get_tera_context("Add expense", id);
    context.insert("input", &input);
    context.insert("validation", &validation_state);
    context.insert("categories", &categories_dropdown_items.items);
    context.insert("current_category_id", &current_category_id);
    context.insert("alerts", &alerts);

    let content = template
        .render("expenses/add.html", &context)
        .map_err(|err| error::ErrorInternalServerError(format!("Template error: {:?}", err)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_database_url;
    use app::AppConfig;
    use db::db_test::create_test_user;
    use diesel::Connection;

    // Tests UserFormInputValid::validate() and ::is_valid().
    #[test]
    fn test_add_form_validation() {
        let test_cases = [
            // The amount and date are required fields.
            (
                AddForm::new("", "", ""),
                AddFormValidation::new(
                    true,
                    Err("Please enter an amount.".to_string()),
                    Err("Please choose a category.".to_string()),
                    Err("Please pick a date.".to_string()),
                ),
                false,
            ),
            // Invalid formats.
        ];

        let conn = db::establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();
        let user = create_test_user(&conn, &config);

        conn.test_transaction::<_, Error, _>(|| {
            for test_case in &test_cases {
                let input = &test_case.0;
                let expected_validate_result = &test_case.1;
                let expected_is_valid_result = test_case.2;
                let actual_validate_result = AddFormValidation::validate(input, &user, &conn);
                assert_eq!(
                    expected_validate_result.amount,
                    actual_validate_result.amount
                );
                assert_eq!(
                    expected_validate_result.category,
                    actual_validate_result.category
                );
                assert_eq!(expected_validate_result.date, actual_validate_result.date);
                assert_eq!(expected_is_valid_result, actual_validate_result.is_valid());
            }
            Ok(())
        });
    }

    // Tests UserFormInputValid::validate() and ::is_valid() with invalid formatted input.
    #[test]
    fn test_add_form_validation_invalid_input_format() {
        let test_cases = [
            AddForm::new("a", "a", "a"),
            AddForm::new("'", "'", "'"),
            AddForm::new(";", ";", ";"),
            AddForm::new(" ", " ", " "),
            AddForm::new("\"", "-z", "-0"),
            AddForm::new("\"", "-1.0", "-10"),
            AddForm::new("0x0f", "0x0f", "0x0f"),
            AddForm::new("00a0-11-11", "00a0-11-11", "00a0-11-11"),
            AddForm::new("99,9", "99,9", "99,9"),
            AddForm::new("99.9 ", "99.9 ", "99.9 "),
            AddForm::new("2020-13-12", "2020-13-12", "2020-13-12"),
            AddForm::new("12-12-2020", "12-12-2020", "12-12-2020"),
            AddForm::new("2020/12/12", "2020/12/12", "2020/12/12"),
        ];

        let conn = db::establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();
        let user = create_test_user(&conn, &config);

        conn.test_transaction::<_, Error, _>(|| {
            for input in &test_cases {
                let actual_validate_result = AddFormValidation::validate(input, &user, &conn);
                assert_eq!(
                    Err("Amount should be in the format '149.99'.".to_string()),
                    actual_validate_result.amount
                );
                assert_eq!(
                    Err("Invalid category ID.".to_string()),
                    actual_validate_result.category
                );
                assert_eq!(
                    Err("Date should be in the format YYYY-MM-DD.".to_string()),
                    actual_validate_result.date
                );
                assert_eq!(false, actual_validate_result.is_valid());
            }
            Ok(())
        });
    }
}
