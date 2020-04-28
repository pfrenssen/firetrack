use super::category::Category;
use super::schema::categories;
use super::schema::categories::dsl;
use super::user::User;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind::{ForeignKeyViolation, UniqueViolation};
use diesel::result::Error::DatabaseError;
use rust_decimal::Decimal;
use serde::Serialize;
use std::fmt;

#[derive(Associations, Clone, Debug, PartialEq, Queryable, Serialize)]
#[belongs_to(Category, foreign_key = "id")]
#[belongs_to(User, foreign_key = "id")]
#[table_name = "categories"]
pub struct Expense {
    pub id: i32,
    pub amount: Decimal,
    pub description: Option<String>,
    pub category_id: i32,
    pub user_id: i32,
    pub date: chrono::NaiveDate,
}

// Possible errors thrown when handling expenses.
#[derive(Debug, PartialEq)]
pub enum ExpenseErrorKind {
    // A category was passed that belongs to the wrong user.
    CategoryHasWrongUser(i32, i32),
}

impl fmt::Display for ExpenseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            ExpenseErrorKind::CategoryHasWrongUser(ref expected_user_id, actual_user_id) => {
                write!(
                    f,
                    "Expected category for user {} instead of user {}",
                    expected_user_id, actual_user_id
                )
            }
        }
    }
}

/// Creates an expense.
pub fn create(
    connection: &PgConnection,
    user: &User,
    amount: &Decimal,
    category: &Category,
    description: Option<&str>,
    date: Option<&chrono::NaiveDate>,
) -> Result<Expense, ExpenseErrorKind> {
    // Check that the category belongs to the same user.
    if category.user_id != user.id {
        return Err(ExpenseErrorKind::CategoryHasWrongUser(
            user.id,
            category.user_id,
        ));
    }
    
    Err(ExpenseErrorKind::CategoryHasWrongUser(0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_test::create_test_user;
    use crate::{establish_connection, get_database_url};
    use app::AppConfig;
    use diesel::result::Error;
    use rust_decimal::Decimal;

    // Test that an error is returned when passing in a category from a different user.
    #[test]
    fn test_create_with_invalid_category() {
        let connection = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        connection.test_transaction::<_, Error, _>(|| {
            // Create a test user that will serve as the owner of the expense.
            let user = create_test_user(&connection, &config);

            // Create a different user that owns the category being passed in.
            let other_user = create_test_user(&connection, &config);
            let other_user_cat = crate::category::create(&connection, &other_user, "Utilities", None, None).unwrap();

            // Try creating an expense using a category belonging to a different user. This should
            // result in an error.
            let result = create(
                &connection,
                &user,
                &Decimal::new(2202, 2),
                &other_user_cat,
                None,
                None
            )
            .unwrap_err();

            assert_eq!(
                ExpenseErrorKind::CategoryHasWrongUser(user.id, other_user.id),
                result
            );

            Ok(())
        });
    }

}
