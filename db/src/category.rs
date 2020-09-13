use super::schema::categories;
use super::schema::categories::dsl;
use super::user::User;
use app::AppConfig;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind::{ForeignKeyViolation, UniqueViolation};
use diesel::result::Error::DatabaseError;
use diesel::{dsl::exists, select};
use serde::Serialize;
use serde_json::{from_reader, Value};
use std::{fmt, fs::File};

#[derive(Associations, Clone, Debug, PartialEq, Queryable, Serialize)]
#[belongs_to(User, foreign_key = "id")]
#[table_name = "categories"]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub user_id: i32,
    pub parent_id: Option<i32>,
}

#[derive(Debug)]
pub struct Categories {
    pub category: Option<Category>,
    pub children: Vec<Categories>,
}

// Converts a flat list of Category objects into a Categories tree.
// Todo: test.
impl From<Vec<Category>> for Categories {
    fn from(list: Vec<Category>) -> Self {
        let mut categories = Categories {
            category: None,
            children: vec![],
        };

        let (children, remaining_list) = get_child_categories_from_flat_list(None, list);
        categories.children = children;

        // Log a warning if there are orphaned categories. This shouldn't happen in practice since
        // the database should maintain the integrity of the relationships.
        let orphan_count = remaining_list.len();
        if orphan_count > 0 {
            let user_id = remaining_list.first().map(|c| c.user_id).unwrap_or(0);
            warn!(
                "User {} has {} orphaned {}",
                user_id,
                orphan_count,
                if orphan_count > 1 {
                    "categories"
                } else {
                    "category"
                }
            );
        }

        categories
    }
}

// Todo: test and document.
fn get_child_categories_from_flat_list(
    parent_id: Option<i32>,
    mut list: Vec<Category>,
) -> (Vec<Categories>, Vec<Category>) {
    let mut categories = vec![];

    let mut i = 0;
    while i != list.len() {
        let cat = &mut list[i];

        if cat.parent_id == parent_id {
            // We found a category that is a child of the passed in parent. Retrieve the children of
            // this category recursively, and build a Categories struct with the result.
            let category = list.remove(i);
            let (mut children, updated_list) =
                get_child_categories_from_flat_list(Some(category.id), list);
            list = updated_list;

            // Sort the child categories alphabetically.
            // Todo: There must be a simpler way to do this.
            children.sort_unstable_by(|a, b| {
                a.category
                    .as_ref()
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "".to_string())
                    .cmp(
                        b.category
                            .as_ref()
                            .map(|c| c.name.clone())
                            .as_ref()
                            .unwrap_or(&"".to_string()),
                    )
            });

            let child_categories = Categories {
                category: Some(category),
                children,
            };
            categories.push(child_categories);

            // Start counting again from the beginning, since the list has been reshuffled.
            i = 0;
        } else {
            i += 1;
        };
    }
    (categories, list)
}

// Possible errors thrown when handling categories.
#[derive(Debug, PartialEq)]
pub enum CategoryErrorKind {
    // Default categories could not be created because the user already has categories.
    AlreadyPopulated(String),
    // The category with the given name and parent already exists.
    CategoryAlreadyExists {
        name: String,
        parent: Option<String>,
    },
    // A database error occurred.
    DatabaseError(diesel::result::Error),
    // A category could not be deleted because it has children.
    HasChildren(i32, String),
    // An error occurred while reading the file containing the default category layout.
    IoError(String, String),
    // The default category listing has malformed or unexpected JSON data.
    MalformedCategoryList,
    // Some required data is missing.
    MissingData(String),
    // The category does not exist.
    NotFound(i32),
    // A category was passed that belongs to the wrong user.
    ParentCategoryHasWrongUser,
}

impl fmt::Display for CategoryErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            CategoryErrorKind::AlreadyPopulated(ref email) => {
                write!(f, "Categories for user {} are already populated", email)
            }
            CategoryErrorKind::CategoryAlreadyExists { name, parent } => match parent {
                Some(p) => write!(
                    f,
                    "The child category '{}' already exists in the parent category '{}'",
                    name, p
                ),
                None => write!(f, "The root category '{}' already exists", name),
            },
            CategoryErrorKind::DatabaseError(ref err) => write!(f, "Database error: {}", err),
            CategoryErrorKind::HasChildren(ref id, orphan_type) => write!(
                f,
                "The category with ID {} could not be deleted because it contains at least one {}",
                id, orphan_type
            ),
            CategoryErrorKind::IoError(ref path, ref err) => {
                write!(f, "I/O error when reading {}: {}", path, err)
            }
            CategoryErrorKind::MalformedCategoryList => write!(
                f,
                "Default categories could not be imported due to malformed data"
            ),
            CategoryErrorKind::MissingData(ref err) => write!(f, "Missing data for field: {}", err),
            CategoryErrorKind::NotFound(ref id) => write!(f, "Category {} not found", id),
            CategoryErrorKind::ParentCategoryHasWrongUser => {
                write!(f, "Parent category should be for the same user",)
            }
        }
    }
}

impl From<diesel::result::Error> for CategoryErrorKind {
    fn from(e: diesel::result::Error) -> Self {
        CategoryErrorKind::DatabaseError(e)
    }
}

/// Creates a category.
pub fn create(
    connection: &PgConnection,
    user: &User,
    name: &str,
    description: Option<&str>,
    parent: Option<&Category>,
) -> Result<Category, CategoryErrorKind> {
    // Validate the category name.
    let name = name.trim();
    if name.is_empty() {
        return Err(CategoryErrorKind::MissingData("category name".to_string()));
    }

    // Check that the parent category belongs to the same user.
    if let Some(parent) = parent {
        if parent.user_id != user.id {
            return Err(CategoryErrorKind::ParentCategoryHasWrongUser);
        }
    }

    let parent_id = parent.map(|c| c.id);

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
            parent: parent.map(|p| p.name.clone()),
        });
    }

    result.map_err(CategoryErrorKind::DatabaseError)
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
    let result = diesel::delete(dsl::categories.filter(dsl::id.eq(id))).execute(connection);

    // Convert a ForeignKeyViolation to a more informative error. This error is thrown when trying
    // to delete a category that still contains an expense or a child category.
    if let Err(DatabaseError(ForeignKeyViolation, info)) = result {
        let orphan_type = if info.message().contains("expenses_category_id_fkey") {
            "expense".to_string()
        } else {
            "category".to_string()
        };

        return Err(CategoryErrorKind::HasChildren(id, orphan_type));
    }

    // Throw an error if nothing was deleted.
    if result? == 0 {
        return Err(CategoryErrorKind::NotFound(id));
    }

    Ok(())
}

/// Returns whether or not the given user has any categories.
pub fn has_categories(connection: &PgConnection, user: &User) -> Result<bool, CategoryErrorKind> {
    select(exists(dsl::categories.filter(dsl::user_id.eq(user.id))))
        .get_result(connection)
        .map_err(CategoryErrorKind::DatabaseError)
}

/// Returns the given user's categories as a flat list.
pub fn get_categories(
    connection: &PgConnection,
    user: &User,
) -> Result<Vec<Category>, CategoryErrorKind> {
    Ok(dsl::categories
        .filter(dsl::user_id.eq(user.id))
        .load::<Category>(connection)?)
}

/// Returns the given user's categories as a tree.
pub fn get_categories_tree(
    connection: &PgConnection,
    user: &User,
) -> Result<Categories, CategoryErrorKind> {
    let categories: Vec<Category> = dsl::categories
        .filter(dsl::user_id.eq(user.id))
        .load::<Category>(connection)?;
    Ok(Categories::from(categories))
}

/// Creates a set of default categories for the given user. The categories are sourced from a JSON
/// file which is set in the app configuration.
pub fn populate_categories(
    connection: &PgConnection,
    user: &User,
    config: &AppConfig,
) -> Result<(), CategoryErrorKind> {
    // Return an error if the user already has categories.
    match has_categories(connection, user) {
        Ok(true) => Err(CategoryErrorKind::AlreadyPopulated(user.email.clone())),
        Ok(false) => Ok(()),
        Err(e) => Err(e),
    }?;

    let path = config.default_categories_json_path();
    let file = File::open(path)
        .map_err(|e| CategoryErrorKind::IoError(path.to_string(), e.to_string()))?;
    let categories: Value =
        from_reader(file).map_err(|_| CategoryErrorKind::MalformedCategoryList)?;

    connection.transaction::<(), CategoryErrorKind, _>(|| {
        populate_categories_from_json(&connection, user.id, &categories, None)
    })
}

// Creates child categories inside the given parent category using the given JSON data.
// This is a recursive function intended for populating the initial set of categories for a new
// user, using the JSON file that contains the category list.
fn populate_categories_from_json(
    connection: &PgConnection,
    // The user for which to create the categories.
    user_id: i32,
    // The JSON data. Can be either:
    // - a JSON object: in this case a set of categories will be created using the object keys as
    //   category names. For each key we will recurse, passing the key as parent category and the
    //   values as children.
    // - a JSON array: the array values will become category names. Any value other than strings
    //   will cause a MalformedCategoryList error to be returned.
    // - an other value: will cause a MalformedCategoryList error.
    json: &Value,
    // The ID of the category which will be the parent of the newly created categories. If `None`
    // the categories will be created in the root.
    parent_id: Option<i32>,
) -> Result<(), CategoryErrorKind> {
    match json {
        Value::Object(o) => {
            let categories = o.keys().map(|k| (k.as_str(), None)).collect();
            let category_ids =
                insert_child_categories(&connection, user_id, parent_id, categories)?;
            let iter = category_ids.iter().zip(o.keys());
            for (id, key) in iter {
                let children = json
                    .get(key)
                    .ok_or(CategoryErrorKind::MalformedCategoryList)?;
                populate_categories_from_json(&connection, user_id, children, Some(*id))?;
            }
            Ok(())
        }
        Value::Array(a) => {
            // Convert the array into a vector of string slices, returning an error if it contains
            // anything that cannot be transformed into a string slice.
            let category_names = a
                .iter()
                .map(|c| c.as_str())
                .collect::<Option<Vec<&str>>>()
                .ok_or(CategoryErrorKind::MalformedCategoryList)?;

            // Todo: add support for category descriptions.
            let categories = category_names.iter().map(|c| (*c, None)).collect();
            insert_child_categories(&connection, user_id, parent_id, categories)?;
            Ok(())
        }
        _ => Err(CategoryErrorKind::MalformedCategoryList),
    }
}

// Creates multiple child categories inside a parent category.
// This is intended for initially populating the categories for a new user. No checks are done to
// ensure that the passed in parent category belongs to the passed in user.
fn insert_child_categories(
    connection: &PgConnection,
    user_id: i32,
    // If the parent ID is omitted the categories will be created in the root.
    parent_id: Option<i32>,
    // A list of child categories consisting of a tuple containing the category name and an optional
    // description.
    categories: Vec<(&str, Option<&str>)>,
) -> Result<Vec<i32>, CategoryErrorKind> {
    let mut records = vec![];
    for (name, description) in categories {
        records.push((
            dsl::name.eq(name),
            dsl::description.eq(description),
            dsl::user_id.eq(user_id),
            dsl::parent_id.eq(parent_id),
        ));
    }

    let result = diesel::insert_into(dsl::categories)
        .values(&records)
        .returning(dsl::id)
        .get_results(connection)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_test::*;
    use crate::{establish_connection, get_database_url};
    use app::AppConfig;
    use diesel::result::Error;
    use serde_json::json;
    use std::collections::{BTreeMap, HashMap};

    // Tests creation of root level categories.
    #[test]
    fn test_create_root_category() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // Create two test users that will serve as the owners of the test categories.
            let user1 = create_test_user(&conn, &config);
            let user2 = create_test_user(&conn, &config);

            // At the start of the test we should have no categories.
            assert_category_count(&conn, 0);

            // Create a root category without a description.
            let name1 = "Housing";
            let create_root_cat = || create(&conn, &user1, name1, None, None);
            let rootcat = create_root_cat().unwrap();
            assert_category(&rootcat, None, name1, None, user1.id, None);
            assert_category_count(&conn, 1);

            // We can create a root category for a different user with the same name.
            let rootcat_user2 = create(&conn, &user2, name1, None, None).unwrap();
            assert_category(&rootcat_user2, None, name1, None, user2.id, None);
            assert_category_count(&conn, 2);

            // We can create a root category with a description.
            let name2 = "Shopping";
            let desc = Some("Clothing, books, hobbies, …");
            let rootcat_desc = create(&conn, &user1, name2, desc, None).unwrap();
            assert_category(&rootcat_desc, None, name2, desc, user1.id, None);
            assert_category_count(&conn, 3);

            // Check that if we try to create a root category with a name that already exists we get
            // an error.
            assert_category_exists_err(create_root_cat().unwrap_err(), name1, None);

            Ok(())
        });
    }

    // Tests creation of child categories.
    #[test]
    fn test_create_child_categories() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        // Test cases, keyed by category name, with optional description and parent category.
        let test_cases: BTreeMap<i8, (&str, Option<&str>, Option<i8>)> = [
            (0, ("Food", Some("Root category"), None)),
            (1, ("Groceries", None, Some(0))),
            (2, ("Groceries", Some("Same name as parent"), Some(1))),
            (3, ("Restaurants", Some("Eating out"), Some(0))),
            (4, ("Japanese restaurants", None, Some(3))),
            (5, ("Sushi", Some("Including delivery"), Some(4))),
            (6, ("Conveyor belt sushi", Some("Choo choo"), Some(5))),
        ]
        .iter()
        .cloned()
        .collect();

        conn.test_transaction::<_, Error, _>(|| {
            // Create two test users that will serve as the owners of the test categories.
            let user1 = create_test_user(&conn, &config);
            let user2 = create_test_user(&conn, &config);

            // At the start of the test we should have no categories.
            let mut count = 0;
            assert_category_count(&conn, count);

            let mut categories = HashMap::new();
            for (id, (name, description, parent_id)) in test_cases {
                let mut create_category = |u: &User| {
                    let parent = parent_id
                        .map(|id| categories.get(&(id, u.id)))
                        .unwrap_or(None);
                    // Create the category for test user 1.
                    let category = create(&conn, &u, name, description, parent);
                    categories.insert((id, u.id), category.unwrap());
                    count += 1;
                    assert_category_count(&conn, count);
                };

                // Different users should be able to create categories with the same names and the
                // same parent categories. Try creating each category for both test users.
                create_category(&user1);
                create_category(&user2);
            }

            // Check that if we try to create a category with a name that already exists for the
            // parent category we get an error. We are using test case 5 (Sushi) which has test case
            // 4 (Japanese restaurants) as parent category.
            let parent = categories.get(&(4, user1.id));
            assert_category_exists_err(
                create(&conn, &user1, "Sushi", None, parent).unwrap_err(),
                "Sushi",
                parent,
            );

            Ok(())
        });
    }

    // Test that an error is returned when creating a category with an empty name.
    #[test]
    fn test_create_with_empty_category_name() {
        let connection = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        connection.test_transaction::<_, Error, _>(|| {
            // Create a test user that will serve as the owner of the test categories.
            let user = create_test_user(&connection, &config);

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

    // Test that an error is returned when passing in a parent category from a different user.
    #[test]
    fn test_create_with_invalid_parent_category() {
        let connection = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        connection.test_transaction::<_, Error, _>(|| {
            // Create a test user that will serve as the owner of the test category.
            let user = create_test_user(&connection, &config);

            // Create a different user that owns some other category.
            let other_user = create_test_user(&connection, &config);

            // Try creating a new category that has a parent category belonging to a different user.
            // This should result in an error.
            let other_user_cat = create(&connection, &other_user, "Utilities", None, None).unwrap();
            let cat = create(
                &connection,
                &user,
                "Telecommunication",
                Some("Internet and telephone"),
                Some(&other_user_cat),
            )
            .unwrap_err();

            assert_eq!(CategoryErrorKind::ParentCategoryHasWrongUser, cat);

            Ok(())
        });
    }

    // Tests super::read().
    #[test]
    fn test_read() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // When no category with the given ID exists, `None` should be returned.
            assert!(read(&conn, 1).is_none());

            // Create a root category and assert that the `read()` function returns it.
            let user = create_test_user(&conn, &config);
            let name = "Groceries";
            let result = create(&conn, &user, name, None, None).unwrap();
            let cat = read(&conn, result.id).unwrap();
            assert_category(&cat, Some(result.id), name, None, user.id, None);

            // Delete the category. Now the `read()` function should return `None` again.
            assert!(delete(&conn, cat.id).is_ok());
            assert!(read(&conn, cat.id).is_none());

            Ok(())
        });
    }

    // Tests super::delete().
    #[test]
    fn test_delete() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // Initially there should not be any categories.
            assert_category_count(&conn, 0);

            // Create a root category. Now there should be one category.
            let user = create_test_user(&conn, &config);
            let name = "Healthcare";
            let cat = create(&conn, &user, name, None, None).unwrap();
            assert_category_count(&conn, 1);

            // Delete the category. This should not result in any errors, and there should again be
            // 0 categories in the database.
            assert!(delete(&conn, cat.id).is_ok());
            assert_category_count(&conn, 0);

            // Try deleting the category again.
            let result = delete(&conn, cat.id);
            assert!(result.is_err());
            assert_eq!(CategoryErrorKind::NotFound(cat.id), result.unwrap_err());

            Ok(())
        });
    }

    // Tests that a category which has a child category cannot be deleted.
    #[test]
    fn test_delete_with_child() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // Create a root category.
            let user = create_test_user(&conn, &config);
            let name = "Lifestyle";
            let parent_cat = create(&conn, &user, name, None, None).unwrap();

            // Create a child category.
            let child_name = "Haircuts";
            create(&conn, &user, child_name, None, Some(&parent_cat)).unwrap();

            // Delete to delete the parent category. This should result in an error.
            let result = delete(&conn, parent_cat.id);
            assert!(result.is_err());
            assert_eq!(
                CategoryErrorKind::HasChildren(parent_cat.id, "category".to_string()),
                result.unwrap_err()
            );

            Ok(())
        });
    }

    // Tests that a category which contains an expense cannot be deleted.
    #[test]
    fn test_delete_category_containing_expense() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // Create a category which contains an expense.
            let user = create_test_user(&conn, &config);
            let cat = create_test_category(&conn, &user);
            create_test_expense(&conn, &user, &cat);

            // Delete to delete the category. This should result in an error.
            let result = delete(&conn, cat.id);
            assert!(result.is_err());
            assert_eq!(
                crate::category::CategoryErrorKind::HasChildren(cat.id, "expense".to_string()),
                result.unwrap_err()
            );

            Ok(())
        });
    }

    // Tests that an error is returned if default categories are created for a user that already has
    // categories.
    #[test]
    fn test_populate_categories_with_existing_categories() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            // Create a test user which has a category.
            let user = create_test_user(&conn, &config);
            create_test_category(&conn, &user);
            assert_eq!(
                CategoryErrorKind::AlreadyPopulated(user.email.clone()),
                populate_categories(&conn, &user, &config).unwrap_err()
            );

            Ok(())
        });
    }

    #[test]
    // Tests that an error is returned when populating categories using a malformed JSON file.
    fn test_populate_categories_using_malformed_json() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let mut config = AppConfig::from_test_defaults();

        let test_files = vec![
            "../resources/fixtures/malformed-default-categories.json",
            "../resources/fixtures/malformed-default-categories2.json",
        ];

        for test_file in test_files {
            conn.test_transaction::<_, Error, _>(|| {
                config.set_default_categories_json_path(test_file.to_string());
                let user = create_test_user(&conn, &config);
                let result = populate_categories(&conn, &user, &config);

                // An error should be returned.
                assert_eq!(result, Err(CategoryErrorKind::MalformedCategoryList));

                // No categories should have been created.
                assert_category_count(&conn, 0);

                Ok(())
            });
        }
    }

    #[test]
    // Tests super::populate_categories().
    fn test_populate_categories() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            let user = create_test_user(&conn, &config);
            let result = populate_categories(&conn, &user, &config);

            // No error should be returned.
            assert_eq!(result, Ok(()));

            // The test file contains 8 categories. All should be created.
            assert_category_count(&conn, 8);

            // Verify that the categories were created with the correct parents.
            let expected_parent_cat_names: Vec<(&str, Option<&str>)> = vec![
                ("Food", None),
                ("Utilities", None),
                ("Alcohol", Some("Food")),
                ("Rakia", Some("Alcohol")),
                ("Groceries", Some("Food")),
                ("Electricity", Some("Utilities")),
                ("Internet", Some("Utilities")),
                ("Water", Some("Utilities")),
            ];

            let cats = get_categories(&conn, &user).unwrap();
            for (cat_name, expected_parent_cat_name) in expected_parent_cat_names {
                // Check that there is exactly 1 category with the expected category name.
                let cats_with_cat_name = cats
                    .iter()
                    .filter(|c| c.name.eq(cat_name))
                    .collect::<Vec<&Category>>();
                assert_eq!(cats_with_cat_name.len(), 1);
                let cat = *cats_with_cat_name.first().unwrap();

                // Check that the parent category matches.
                match cat.parent_id {
                    None => assert!(expected_parent_cat_name.is_none()),
                    Some(id) => {
                        let parent_cat = cats
                            .iter()
                            .filter(|c| c.id.eq(&id))
                            .collect::<Vec<&Category>>();
                        let parent_cat = parent_cat.first().unwrap();
                        assert_eq!(parent_cat.name, expected_parent_cat_name.unwrap());
                    }
                }
            }

            Ok(())
        });
    }

    // Tests super::has_categories().
    #[test]
    fn test_has_categories() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            let user1 = create_test_user(&conn, &config);
            let user2 = create_test_user(&conn, &config);
            assert_eq!(false, has_categories(&conn, &user1).unwrap());
            assert_eq!(false, has_categories(&conn, &user2).unwrap());
            create_test_category(&conn, &user1);
            assert_eq!(true, has_categories(&conn, &user1).unwrap());
            assert_eq!(false, has_categories(&conn, &user2).unwrap());
            create_test_category(&conn, &user2);
            assert_eq!(true, has_categories(&conn, &user1).unwrap());
            assert_eq!(true, has_categories(&conn, &user2).unwrap());

            Ok(())
        });
    }

    // Tests super::get_categories().
    #[test]
    fn test_get_categories() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            let user1 = create_test_user(&conn, &config);
            let user2 = create_test_user(&conn, &config);

            // Initially both users don't have any categories.
            let no_cats: Vec<Category> = vec![];
            assert_eq!(no_cats, get_categories(&conn, &user1).unwrap());
            assert_eq!(no_cats, get_categories(&conn, &user2).unwrap());

            // Create a root category for user 1 and check that it is returned correctly.
            let user1_cat1 = create_test_category(&conn, &user1);
            assert_eq!(
                vec![user1_cat1.clone()],
                get_categories(&conn, &user1).unwrap()
            );
            assert_eq!(no_cats, get_categories(&conn, &user2).unwrap());

            // Create a root category for user 2.
            let user2_cat1 = create_test_category(&conn, &user2);
            assert_eq!(
                vec![user1_cat1.clone()],
                get_categories(&conn, &user1).unwrap()
            );
            assert_eq!(
                vec![user2_cat1.clone()],
                get_categories(&conn, &user2).unwrap()
            );

            // Create a child category for user 1.
            let user1_cat2 = create_test_category_with_parent(&conn, &user1, Some(&user1_cat1));
            assert_eq!(
                vec![user1_cat1.clone(), user1_cat2.clone()],
                get_categories(&conn, &user1).unwrap()
            );
            assert_eq!(
                vec![user2_cat1.clone()],
                get_categories(&conn, &user2).unwrap()
            );

            // Create some more root and child categories for user 1.
            let user1_cat3 = create_test_category(&conn, &user1);
            assert_eq!(
                vec![user1_cat1.clone(), user1_cat2.clone(), user1_cat3.clone()],
                get_categories(&conn, &user1).unwrap()
            );
            assert_eq!(
                vec![user2_cat1.clone()],
                get_categories(&conn, &user2).unwrap()
            );

            let user1_cat4 = create_test_category_with_parent(&conn, &user1, Some(&user1_cat2));
            assert_eq!(
                vec![user1_cat1, user1_cat2, user1_cat3, user1_cat4],
                get_categories(&conn, &user1).unwrap()
            );
            assert_eq!(vec![user2_cat1], get_categories(&conn, &user2).unwrap());

            Ok(())
        });
    }

    // Simplified version of the Categories struct, used for testing.
    struct ExpectedCategories {
        pub category: Option<String>,
        pub children: Vec<ExpectedCategories>,
    }

    #[test]
    // Tests super::get_categories_tree.
    fn test_get_categories_tree() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        conn.test_transaction::<_, Error, _>(|| {
            let user = create_test_user(&conn, &config);
            populate_categories(&conn, &user, &config).unwrap();

            let expected_categories = ExpectedCategories {
                category: None,
                children: vec![
                    ExpectedCategories {
                        category: Some("Food".to_string()),
                        children: vec![
                            ExpectedCategories {
                                category: Some("Alcohol".to_string()),
                                children: vec![ExpectedCategories {
                                    category: Some("Rakia".to_string()),
                                    children: vec![],
                                }],
                            },
                            ExpectedCategories {
                                category: Some("Groceries".to_string()),
                                children: vec![],
                            },
                        ],
                    },
                    ExpectedCategories {
                        category: Some("Utilities".to_string()),
                        children: vec![
                            // Child categories are defined with an arbitrary sorting order in the
                            // fixtures file but they should be sorted in alphabetical order.
                            ExpectedCategories {
                                category: Some("Electricity".to_string()),
                                children: vec![],
                            },
                            ExpectedCategories {
                                category: Some("Internet".to_string()),
                                children: vec![],
                            },
                            ExpectedCategories {
                                category: Some("Water".to_string()),
                                children: vec![],
                            },
                        ],
                    },
                ],
            };

            let cat_tree = get_categories_tree(&conn, &user).unwrap();
            assert_category_tree(&expected_categories, &cat_tree, user.id, None);

            Ok(())
        });
    }

    // Checks recursively that the passed in Categories tree matches the ExpectedCategories tree.
    // Each category is checked that it belongs to the correct user and has the expected parent ID.
    fn assert_category_tree(
        expected_categories: &ExpectedCategories,
        categories: &Categories,
        expected_user_id: i32,
        expected_parent_id: Option<i32>,
    ) {
        assert_eq!(
            expected_categories.category,
            categories.category.as_ref().map(|c| c.name.clone())
        );
        if let Some(cat) = categories.category.clone() {
            assert_eq!(expected_user_id, cat.user_id);
            assert_eq!(expected_parent_id, cat.parent_id);
        }

        // Check that the child categories are in the expected order.
        let expected_child_count = expected_categories.children.len();
        assert_eq!(expected_child_count, categories.children.len());
        if expected_child_count > 0 {
            // Pass on the ID of the current category when recursing, so that we can check that the
            // children have the parent ID set correctly.
            let parent_id = match &categories.category {
                None => None,
                Some(c) => Some(c.id),
            };

            for i in 0..expected_child_count {
                let expected_child_cat = &expected_categories.children[i];
                let actual_child_cat = &categories.children[i];
                assert_eq!(
                    expected_child_cat.category,
                    actual_child_cat.category.as_ref().map(|c| c.name.clone())
                );
                assert_category_tree(
                    expected_child_cat,
                    actual_child_cat,
                    expected_user_id,
                    parent_id,
                );
            }
        }
    }

    #[test]
    // Tests that an error is returned when trying to populate default categories using malformed
    // JSON.
    fn test_populate_categories_from_malformed_json() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        let test_cases = vec![
            json!("a string"),
            json!(1.2),
            json!(0),
            json!(null),
            json!(true),
            json!(["an", "array", 0, "containing", "a", "non-string", "value"]),
            json!({"Object with invalid value": "string"}),
            json!({"Object with invalid value": 0}),
            json!({"Object with invalid value": 1.2}),
            json!({ "Object with invalid value": null }),
            json!({"Object with invalid value": true}),
            json!({"Object with invalid nested value": {"deeper": {"deeper": null}}}),
            json!({
                "Nested": ["object", "that"],
                "Somewhere": {
                    "Inside": ["it", "has"],
                    "An": {
                        "Invalid": ["value", null]
                    }
                },
            }),
        ];

        for test_case in test_cases {
            conn.test_transaction::<_, Error, _>(|| {
                let user = create_test_user(&conn, &config);
                let result = populate_categories_from_json(&conn, user.id, &test_case, None);
                assert_eq!(
                    result.unwrap_err(),
                    CategoryErrorKind::MalformedCategoryList
                );

                Ok(())
            });
        }
    }

    #[test]
    // Tests super::populate_categories_from_json().
    fn test_populate_categories_from_json() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        // Each test case consists of a tuple, with the first element a JSON value, the second an
        // integer representing the expected number of root categories, and the last element an
        // integer representing the total number of expected categories.
        let test_cases = vec![
            (json!([]), 0, 0),
            (json!({}), 0, 0),
            (json!({"Education": []}), 1, 1),
            (json!(["Books"]), 1, 1),
            (json!({"Entertainment": ["Concerts", "Dining"]}), 1, 3),
            (json!({"Financial": [], "Food": []}), 2, 2),
            (
                json!({
                    "Food and drink": {
                        "Drinks": ["Alcohol", "Water", "Coffee"],
                        "Eating out": {
                            "Restaurants": ["Italian", "Japanese"],
                            "Quick bites": {
                                "Breakfast": ["Coffee"],
                                "Lunch": ["Coffee"],
                            },
                        }
                    }
                }),
                1,
                14,
            ),
        ];

        for (test_case, expected_root_count, expected_total_count) in test_cases {
            conn.test_transaction::<_, Error, _>(|| {
                let user = create_test_user(&conn, &config);
                let result = populate_categories_from_json(&conn, user.id, &test_case, None);
                assert_eq!(result, Ok(()));
                assert_root_category_count(&conn, expected_root_count);
                assert_category_count(&conn, expected_total_count);

                Ok(())
            });
        }
    }

    #[test]
    // Tests super::insert_child_categories().
    fn test_insert_child_categories() {
        let conn = establish_connection(&get_database_url()).unwrap();
        let config = AppConfig::from_test_defaults();

        // Define a custom assertion for validating the categories created in the test.
        let assert_cats = |cats: Vec<(&str, Option<&str>)>,
                           parent_id: Option<i32>,
                           result: Vec<i32>,
                           user_id: i32| {
            // We should get back the 2 IDs of the created categories.
            assert_eq!(2, result.len());

            // Check that the categories contain the right data.
            for i in 0..2 {
                let id = result.get(i).unwrap();
                let (name, description) = cats.get(i).unwrap();
                let category = read(&conn, *id).unwrap();
                assert_category(&category, Some(*id), name, *description, user_id, parent_id);
            }
        };

        conn.test_transaction::<_, Error, _>(|| {
            let user = create_test_user(&conn, &config);

            // Initially there are no categories in the database.
            assert_category_count(&conn, 0);

            // Try creating two root categories, one with a description and one without.
            let root_cats = vec![
                ("Healthcare", None),
                ("Housing", Some("Expenses related to a residence")),
            ];
            let result = insert_child_categories(&conn, user.id, None, root_cats.clone()).unwrap();

            // There should be 2 categories in the database now.
            assert_category_count(&conn, 2);
            assert_cats(root_cats, None, result.clone(), user.id);

            // Create 2 child categories, one with a description and one without.
            let parent_id = result.get(0).unwrap();

            let child_cats = vec![
                ("Dentist", None),
                ("Doctor", Some("Visiting a general practitioner")),
            ];
            let result =
                insert_child_categories(&conn, user.id, Some(*parent_id), child_cats.clone())
                    .unwrap();

            // There should be 4 categories in the database now.
            assert_category_count(&conn, 4);
            assert_cats(child_cats.clone(), Some(*parent_id), result, user.id);

            // Inserting the same categories again should result in an error.
            let result =
                insert_child_categories(&conn, user.id, Some(*parent_id), child_cats.clone());
            assert!(result.is_err());

            Ok(())
        });
    }

    // Checks that the given category matches the given values.
    fn assert_category(
        // The category to check.
        category: &Category,
        // The expected category ID. If None this will not be checked.
        id: Option<i32>,
        // The expected category name.
        name: &str,
        // The expected description.
        description: Option<&str>,
        // The expected user ID of the category owner.
        user_id: i32,
        // The expected parent category ID.
        parent_id: Option<i32>,
    ) {
        if let Some(id) = id {
            assert_eq!(id, category.id);
        }
        assert_eq!(name, category.name);
        assert_eq!(description.map(|d| d.to_string()), category.description);
        assert_eq!(user_id, category.user_id);
        assert_eq!(parent_id, category.parent_id);
    }

    // Checks that the number of categories stored in the database matches the expected count.
    fn assert_category_count(connection: &PgConnection, expected_count: i64) {
        let actual_count: i64 = dsl::categories
            .select(diesel::dsl::count_star())
            .first(connection)
            .unwrap();
        assert_eq!(expected_count, actual_count);
    }

    // Checks that the number of root categories stored in the database matches the expected count.
    fn assert_root_category_count(connection: &PgConnection, expected_count: i64) {
        let actual_count: i64 = dsl::categories
            .select(diesel::dsl::count_star())
            .filter(dsl::parent_id.is_null())
            .first(connection)
            .unwrap();
        assert_eq!(expected_count, actual_count);
    }

    // Checks that the given error is an CategoryErrorKind::CategoryAlreadyExists error.
    fn assert_category_exists_err(error: CategoryErrorKind, name: &str, parent: Option<&Category>) {
        assert_eq!(
            error,
            CategoryErrorKind::CategoryAlreadyExists {
                name: name.to_string(),
                parent: parent.map(|p| p.name.clone())
            }
        );
    }
}
