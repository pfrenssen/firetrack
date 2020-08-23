use super::{assert_authenticated, get_tera_context};
use actix_identity::Identity;
use actix_web::{error, web, Error, HttpResponse};

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
    template: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    assert_authenticated(&id)?;

    let context = get_tera_context("Add expense", id);

    let content = template
        .render("expenses/add.html", &context)
        .map_err(|err| error::ErrorInternalServerError(format!("Template error: {:?}", err)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}
