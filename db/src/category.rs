use super::schema::categories;
use super::schema::categories::dsl;
use super::user::User;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind::UniqueViolation, Error::DatabaseError};
use std::fmt;

#[derive(Associations, Clone, Debug, PartialEq, Queryable)]
#[belongs_to(User, foreign_key = "id")]
#[table_name = "categories"]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub user_id: i32,
    pub parent_id: Option<i32>,
}

// Possible errors thrown when handling categories.
#[derive(Debug, PartialEq)]
pub enum CategoryErrorKind {
    // Some required data is missing.
    MissingData(String),
    // The category with the given name and parent already exists.
    CategoryAlreadyExists {
        name: String,
        parent: Option<Category>,
    },
    // A category could not be created due to a database error.
    CreationFailed(diesel::result::Error),
    // A category could not be deleted due to a database error.
    DeletionFailed(diesel::result::Error),
    // A category was passed that belongs to the wrong user.
    ParentCategoryHasWrongUser(i32, i32),
}

impl fmt::Display for CategoryErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            CategoryErrorKind::MissingData(ref err) => write!(f, "Missing data for field: {}", err),
            CategoryErrorKind::CategoryAlreadyExists { name, parent } => match parent {
                Some(p) => write!(
                    f,
                    "The child category '{}' already exists in the parent category '{}'",
                    name, p.name
                ),
                None => write!(f, "The root category '{}' already exists", name),
            },
            CategoryErrorKind::CreationFailed(ref err) => {
                write!(f, "Database error when creating category: {}", err)
            }
            CategoryErrorKind::DeletionFailed(ref err) => {
                write!(f, "Database error when deleting category: {}", err)
            }
            CategoryErrorKind::ParentCategoryHasWrongUser(ref expected_user_id, actual_user_id) => {
                write!(
                    f,
                    "Expected parent category for user {} instead of user {}",
                    expected_user_id, actual_user_id
                )
            }
        }
    }
}

/// Creates a category.
pub fn create(
    connection: &PgConnection,
    user: &User,
    name: &str,
    description: Option<&str>,
    parent: Option<Category>,
) -> Result<Category, CategoryErrorKind> {
    // Validate the category name.
    let name = name.trim();
    if name.is_empty() {
        return Err(CategoryErrorKind::MissingData("category name".to_string()));
    }

    // Check that the parent category belongs to the same user.
    if let Some(parent) = &parent {
        if parent.user_id != user.id {
            return Err(CategoryErrorKind::ParentCategoryHasWrongUser(
                user.id,
                parent.user_id,
            ));
        }
    }

    let parent_id = parent.clone().map(|c| c.id);

    let result = diesel::insert_into(dsl::categories)
        .values((
            dsl::name.eq(&name),
            dsl::description.eq(description),
            dsl::user_id.eq(user.id),
            dsl::parent_id.eq(parent_id),
        ))
        .returning((
            dsl::id,
            dsl::name,
            dsl::description,
            dsl::user_id,
            dsl::parent_id,
        ))
        .get_result(connection);

    // Convert a UniqueViolation to a more informative CategoryAlreadyExists error.
    if let Err(DatabaseError(UniqueViolation, _)) = result {
        return Err(CategoryErrorKind::CategoryAlreadyExists {
            name: name.to_string(),
            parent,
        });
    }

    result.map_err(CategoryErrorKind::CreationFailed)
}

/// Retrieves the category with the given ID.
pub fn read(connection: &PgConnection, id: i32) -> Option<Category> {
    let category = dsl::categories.find(id).first::<Category>(connection);

    match category {
        Ok(c) => Some(c),
        Err(_) => None,
    }
}

/// Deletes the category with the given ID.
pub fn delete(connection: &PgConnection, id: i32) -> Result<(), CategoryErrorKind> {
    diesel::delete(dsl::categories.filter(dsl::id.eq(id)))
        .execute(connection)
        .map_err(CategoryErrorKind::DeletionFailed)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_test::create_test_user;
    use crate::{establish_connection, get_database_url};
    use app::AppConfig;
    use diesel::result::Error;

    // Tests super::create().
    #[test]
    fn test_create_with_empty_category_name() {
        let connection = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        connection.test_transaction::<_, Error, _>(|| {
            // Create a test user that will serve as the owner of the test categories.
            let user = create_test_user(&connection, &config);

            // When creating a category with an empty name an error should be returned.
            let mut empty_names = vec![
                "".to_string(),         // Empty string.
                " ".to_string(),        // Space.
                "\n".to_string(),       // Line feed.
                "\t".to_string(),       // Horizontal tab.
                '\u{0B}'.to_string(),   // Vertical tab.
                '\u{0C}'.to_string(),   // Form feed.
                '\u{85}'.to_string(),   // Next line.
                '\u{1680}'.to_string(), // Ogham space mark.
                '\u{2002}'.to_string(), // En space.
                '\u{2003}'.to_string(), // Em space.
                '\u{2004}'.to_string(), // Three-per-em space.
                '\u{2005}'.to_string(), // Four-per-em space.
                '\u{2006}'.to_string(), // Six-per-em space.
                '\u{2007}'.to_string(), // Figure space.
                '\u{2008}'.to_string(), // Punctuation space.
                '\u{2009}'.to_string(), // Thin space.
                '\u{200A}'.to_string(), // Hair space.
                '\u{2028}'.to_string(), // Line separator.
                '\u{2029}'.to_string(), // Paragraph separator.
                '\u{202F}'.to_string(), // Narrow no-break space.
                '\u{205F}'.to_string(), // Medium mathematical space.
                '\u{3000}'.to_string(), // Ideographic space.
            ];

            // Also test a combination of various whitespace characters.
            empty_names.push(format!(" \n\t{}{}{}", '\u{1680}', '\u{2005}', '\u{2028}'));

            for empty_name in empty_names {
                let created_category =
                    create(&connection, &user, &empty_name, None, None).unwrap_err();
                assert_eq!(
                    CategoryErrorKind::MissingData("category name".to_string()),
                    created_category
                );
            }

            Ok(())
        });
    }

    // Tests super::read().
    #[test]
    fn test_read() {
        let connection = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        connection.test_transaction::<_, Error, _>(|| {
            // Create a test user that will serve as the owner of the test categories.
            let user = create_test_user(&connection, &config);

            let name = "Groceries";

            // When no category with the given ID exists, `None` should be returned.
            assert!(read(&connection, 1).is_none());

            // Create a root category and assert that the `read()` function returns it.
            let created_category = create(&connection, &user, name, None, None).unwrap();
            let category = read(&connection, created_category.id).unwrap();
            assert_category(&category, created_category.id, name, None, user.id, None);

            // Delete the category. Now the `read()` function should return `None` again.
            assert!(delete(&connection, category.id).is_ok());
            assert!(read(&connection, category.id).is_none());

            Ok(())
        });
    }

    // Checks that the given category matches the given values.
    fn assert_category(
        // The category to check.
        category: &Category,
        // The expected category ID.
        id: i32,
        // The expected category name.
        name: &str,
        // The expected description.
        description: Option<String>,
        // The expected user ID of the category owner.
        user_id: i32,
        // The expected parent category ID.
        parent_id: Option<i32>,
    ) {
        assert_eq!(id, category.id);
        assert_eq!(name, category.name);
        assert_eq!(description, category.description);
        assert_eq!(user_id, category.user_id);
        assert_eq!(parent_id, category.parent_id);
    }
}
