use super::{assert_authenticated, get_tera_context};
use crate::category::CategoryDropdownItems;

use actix_identity::Identity;
use actix_web::{error, web, Error, HttpResponse};
use db::category::get_categories_tree;
use db::user::read;

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

// Request handler for the add form.
pub async fn add_handler(
    id: Identity,
    pool: web::Data<db::ConnectionPool>,
    template: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    let email = assert_authenticated(&id)?;

    // Retrieve the categories for the current user.
    let connection = pool.get().map_err(error::ErrorInternalServerError)?;
    let user = read(&connection, email.as_str()).map_err(error::ErrorInternalServerError)?;
    let categories =
        get_categories_tree(&connection, &user).map_err(error::ErrorInternalServerError)?;

    let categories_dropdown_items = CategoryDropdownItems::from(categories);

    let mut context = get_tera_context("Add expense", id);
    let current_category_id: Option<i32> = None;
    context.insert("categories", &categories_dropdown_items.items);
    context.insert("current_category_id", &current_category_id);

    let content = template
        .render("expenses/add.html", &context)
        .map_err(|err| error::ErrorInternalServerError(format!("Template error: {:?}", err)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}
