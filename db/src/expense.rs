use super::category::Category;
use super::schema::expenses;
use super::schema::expenses::dsl;
use super::user::User;
use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;
use std::fmt;

#[derive(Associations, Clone, Debug, PartialEq, Queryable, Serialize)]
#[belongs_to(Category, foreign_key = "id")]
#[belongs_to(User, foreign_key = "id")]
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
    CategoryHasWrongUser,
    // An expense could not be created due to a database error.
    CreationFailed(diesel::result::Error),
    // An expense could not be deleted due to a database error.
    DeletionFailed(diesel::result::Error),
    // The amount should be greater than 0.
    InvalidAmount,
    // An expense does not exist.
    NotFound(i32),
    // A database error occurred while reading expenses.
    ReadFailed(diesel::result::Error),
}

impl fmt::Display for ExpenseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            ExpenseErrorKind::CategoryHasWrongUser => write!(f, "Category is from the wrong user",),
            ExpenseErrorKind::CreationFailed(ref err) => {
                write!(f, "Database error when creating expense: {}", err)
            }
            ExpenseErrorKind::DeletionFailed(ref err) => {
                write!(f, "Database error when deleting expense: {}", err)
            }
            ExpenseErrorKind::InvalidAmount => {
                write!(f, "Amount should be between 0.01 and 9999999.99")
            }
            ExpenseErrorKind::NotFound(ref id) => write!(f, "Expense {} not found", id),
            ExpenseErrorKind::ReadFailed(ref err) => {
                write!(f, "Database error when reading expense: {}", err)
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
        return Err(ExpenseErrorKind::CategoryHasWrongUser);
    }

    if *amount <= Decimal::new(0, 2) || *amount > Decimal::new(999_999_999, 2) {
        return Err(ExpenseErrorKind::InvalidAmount);
    }

    diesel::insert_into(dsl::expenses)
        .values((
            dsl::amount.eq(amount),
            dsl::description.eq(description),
            dsl::category_id.eq(category.id),
            dsl::user_id.eq(user.id),
            dsl::date.eq(date.unwrap_or(&Utc::now().naive_utc().date())),
        ))
        .returning((
            dsl::id,
            dsl::amount,
            dsl::description,
            dsl::category_id,
            dsl::user_id,
            dsl::date,
        ))
        .get_result(connection)
        .map_err(ExpenseErrorKind::CreationFailed)
}

/// Retrieves the expense with the given ID.
pub fn read(connection: &PgConnection, id: i32) -> Option<Expense> {
    let expense = dsl::expenses.find(id).first::<Expense>(connection);

    match expense {
        Ok(c) => Some(c),
        Err(_) => None,
    }
}

/// Deletes the expense with the given ID.
pub fn delete(connection: &PgConnection, id: i32) -> Result<(), ExpenseErrorKind> {
    let result = diesel::delete(dsl::expenses.filter(dsl::id.eq(id))).execute(connection);

    let result = result.map_err(ExpenseErrorKind::DeletionFailed)?;

    // Throw an error if nothing was deleted.
    if result == 0 {
        return Err(ExpenseErrorKind::NotFound(id));
    }

    Ok(())
}

/// Returns all expenses, optionally filtered by user ID.
pub fn list(
    connection: &PgConnection,
    user_id: Option<i32>,
) -> Result<Vec<Expense>, ExpenseErrorKind> {
    let result = match user_id {
        Some(user_id) => dsl::expenses
            .filter(expenses::user_id.eq(&user_id))
            .load::<Expense>(connection),
        None => dsl::expenses.load::<Expense>(connection),
    };

    result.map_err(ExpenseErrorKind::ReadFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_test::*;
    use crate::{establish_connection, get_database_url};
    use app::AppConfig;
    use diesel::result::Error;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // Tests super::create().
    #[test]
    fn test_create() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // Create 2 test users with each 2 categories.
            let mut test_user_cats: Vec<(User, (Category, Category))> = vec![];
            for _i in 0..2 {
                let user = create_test_user(&conn, &config);
                let cat1 = create_test_category(&conn, &user);
                let cat2 = create_test_category(&conn, &user);
                test_user_cats.push((user, (cat1, cat2)));
            }

            let test_cases = vec![
                ("0.01", None, "2020-01-01"),
                ("0.01", Some("Rubber band"), "2250-01-01"),
                ("99.99", None, "1883-08-26"),
                ("99.99", Some("Sushi"), "2020-05-01"),
                ("9999999.99", None, "1984-03-03"),
                ("9999999.99", Some("Another yacht"), "2019-12-31"),
            ];

            // At the start of the test we should have no expenses.
            let mut expected_count = 0;
            assert_expense_count(&conn, expected_count);

            for (amount, desc, date) in test_cases {
                let amount = Decimal::from_str(amount).unwrap();
                let date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
                for (user, (cat1, cat2)) in &test_user_cats {
                    let expense = create(&conn, user, &amount, cat1, desc, Some(&date)).unwrap();
                    assert_expense(&expense, None, &amount, desc, cat1.id, user.id, date);
                    expected_count += 1;
                    assert_expense_count(&conn, expected_count);
                    let expense = create(&conn, user, &amount, cat2, desc, Some(&date)).unwrap();
                    assert_expense(&expense, None, &amount, desc, cat2.id, user.id, date);
                    expected_count += 1;
                    assert_expense_count(&conn, expected_count);
                }
            }

            Ok(())
        });
    }

    // Tests that an expense created without passing a date will get today's date.
    #[test]
    fn test_create_without_date() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            let user = create_test_user(&conn, &config);
            let cat = create_test_category(&conn, &user);
            let amount = Decimal::from_str("1474.95").unwrap();
            let expense = create(&conn, &user, &amount, &cat, None, None).unwrap();
            assert_expense(
                &expense,
                None,
                &amount,
                None,
                cat.id,
                user.id,
                Utc::now().naive_utc().date(),
            );

            Ok(())
        });
    }

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
            let other_user_cat =
                crate::category::create(&connection, &other_user, "Utilities", None, None).unwrap();

            // Try creating an expense using a category belonging to a different user. This should
            // result in an error.
            let result = create(
                &connection,
                &user,
                &Decimal::from_str("22.02").unwrap(),
                &other_user_cat,
                None,
                None,
            )
            .unwrap_err();

            assert_eq!(ExpenseErrorKind::CategoryHasWrongUser, result);

            Ok(())
        });
    }

    // Test that an error is returned when the passed in amount is 0 or lower.
    #[test]
    fn test_create_with_invalid_amount() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        let min_value = Decimal::min_value().to_string();
        let test_cases = vec!["0.00", "-0.01", "-1.00", min_value.as_str(), "10000000.00"];

        conn.test_transaction::<_, Error, _>(|| {
            let user = create_test_user(&conn, &config);
            let cat = create_test_category(&conn, &user);

            for test_case in test_cases {
                let amount = &Decimal::from_str(test_case).unwrap();
                let result = create(&conn, &user, amount, &cat, None, None);
                assert_eq!(ExpenseErrorKind::InvalidAmount, result.unwrap_err());
            }

            Ok(())
        });
    }

    // Tests super::read().
    #[test]
    fn test_read() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // When no expense with the given ID exists, `None` should be returned.
            assert!(read(&conn, 1).is_none());

            // Create an expense and assert that the `read()` function returns it.
            let user = create_test_user(&conn, &config);
            let cat = create_test_category(&conn, &user);
            let amount = Decimal::from_str("99.95").unwrap();
            let result = create(&conn, &user, &amount, &cat, None, None).unwrap();
            let expense = read(&conn, result.id).unwrap();
            assert_expense(
                &expense,
                Some(result.id),
                &amount,
                None,
                cat.id,
                user.id,
                Utc::now().naive_utc().date(),
            );

            // Delete the expense. Now the `read()` function should return `None` again.
            assert!(delete(&conn, expense.id).is_ok());
            assert!(read(&conn, expense.id).is_none());

            Ok(())
        });
    }

    // Tests super::list().
    #[test]
    fn test_list() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // When no expenses exist, an empty vector should be returned.
            assert!(list(&conn, None).unwrap().is_empty());
            assert!(list(&conn, Some(1)).unwrap().is_empty());

            // Create 2 users with 2 expenses each.
            let mut users: Vec<User> = vec![];
            let mut expenses: Vec<Expense> = vec![];
            for _ in 0..2 {
                let user = create_test_user(&conn, &config);
                for _ in 0..2 {
                    let cat = create_test_category(&conn, &user);
                    expenses.push(create_test_expense(&conn, &user, &cat));
                }
                users.push(user);
            }
            assert_expense_count(&conn, 4);

            // Check that all expenses are returned when we don't filter by user.
            let result = list(&conn, None).unwrap();
            assert_eq!(expenses, result);

            // Check that we can retrieve the expenses of both users.
            for _ in 0..2 {
                let user = users.remove(0);
                let expected_expenses = expenses.drain(0..2);
                let result = list(&conn, Some(user.id)).unwrap();
                assert_eq!(expected_expenses.as_slice(), result.as_slice());
            }

            Ok(())
        });
    }

    // Tests super::delete().
    #[test]
    fn test_delete() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // Initially there should not be any expenses.
            assert_expense_count(&conn, 0);

            // Create an expense. Now there should be one expense.
            let user = create_test_user(&conn, &config);
            let cat = create_test_category(&conn, &user);
            let amount = Decimal::from_str("99.95").unwrap();
            let expense = create(&conn, &user, &amount, &cat, None, None).unwrap();
            assert_expense_count(&conn, 1);

            // Delete the expense. This should not result in any errors, and there should again be 0
            // expenses in the database.
            assert!(delete(&conn, expense.id).is_ok());
            assert_expense_count(&conn, 0);

            // Try deleting the expense again.
            let result = delete(&conn, expense.id);
            assert!(result.is_err());
            assert_eq!(ExpenseErrorKind::NotFound(expense.id), result.unwrap_err());

            Ok(())
        });
    }

    // Checks that the given expense matches the given values.
    fn assert_expense(
        // The expense to check.
        expense: &Expense,
        // The expected expense ID. If None this will not be checked.
        id: Option<i32>,
        // The expected amount.
        amount: &Decimal,
        // The expected description.
        description: Option<&str>,
        // The expected category ID.
        category_id: i32,
        // The expected user ID of the category owner.
        user_id: i32,
        // The expected date.
        date: chrono::NaiveDate,
    ) {
        if let Some(id) = id {
            assert_eq!(id, expense.id);
        }
        assert_eq!(*amount, expense.amount);
        assert_eq!(description.map(|d| d.to_string()), expense.description);
        assert_eq!(category_id, expense.category_id);
        assert_eq!(user_id, expense.user_id);
        assert_eq!(date, expense.date);
    }

    // Checks that the number of expenses stored in the database matches the expected count.
    fn assert_expense_count(connection: &PgConnection, expected_count: i64) {
        let actual_count: i64 = dsl::expenses
            .select(diesel::dsl::count_star())
            .first(connection)
            .unwrap();
        assert_eq!(expected_count, actual_count);
    }
}
