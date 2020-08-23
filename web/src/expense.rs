use super::bootstrap_components::{Alert, AlertType};
use super::{assert_authenticated, get_tera_context};
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{error, web, Error, HttpResponse};
use app::AppConfig;
use db::activation_code::ActivationCodeErrorKind;
use db::user::UserErrorKind;
use diesel::PgConnection;
use validator::validate_email;

// Request handler for the expenses overview.
pub async fn overview_handler(
    id: Identity,
    session: Session,
    tera: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    assert_authenticated(&id)?;

    let content = "Expenses".to_string();
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}
// Request handler for the add form.
pub async fn add_handler(
    id: Identity,
    session: Session,
    tera: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    assert_authenticated(&id)?;

    let content = "Add expense".to_string();
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}
