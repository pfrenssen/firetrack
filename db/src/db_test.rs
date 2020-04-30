use super::*;

use crate::category::Category;
use crate::user::User;
use app::AppConfig;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::iter;

/// Creates a test user using a random email address.
pub fn create_test_user(connection: &PgConnection, config: &AppConfig) -> User {
    let username = random_string(10);
    user::create(
        &connection,
        format!("{}@example.com", username).as_str(),
        "letmein",
        &config,
    )
    .unwrap()
}

/// Creates a test category using a random name.
pub fn create_test_category(connection: &PgConnection, user: &User) -> Category {
    crate::category::create(&connection, &user, random_string(10).as_str(), None, None).unwrap()
}

// Returns a random alphanumeric string of the given length.
fn random_string(length: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(length)
        .collect()
}
