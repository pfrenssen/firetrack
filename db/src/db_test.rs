use super::*;

use crate::user::User;
use app::AppConfig;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::iter;

/// Creates a test user using a random email address.
pub fn create_test_user(connection: &PgConnection, config: &AppConfig) -> User {
    let mut rng = thread_rng();
    let username: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(10)
        .collect();
    user::create(
        &connection,
        format!("{}@example.com", username).as_str(),
        "letmein",
        &config,
    )
    .unwrap()
}
