use super::schema::activation_codes;
use super::schema::activation_codes::dsl;
use super::user::{User, UserErrorKind};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rand::{thread_rng, Rng};
use std::fmt;

// The minimum and maximum values for a random activation code.
const MIN_VALUE: i32 = 100_000;
const MAX_VALUE: i32 = 999_999;

// The maximum number of activations that can be attempted in 30 minutes.
const MAX_ATTEMPTS: i16 = 5;

#[derive(Associations, Clone, Debug, PartialEq, Queryable)]
#[belongs_to(User, foreign_key = "email")]
pub struct ActivationCode {
    pub email: String,
    pub code: i32,
    pub expiration_time: chrono::NaiveDateTime,
    pub attempts: i16,
}

impl ActivationCode {
    /// Returns whether or not the activation code is expired.
    ///
    /// # Example
    ///
    /// ```
    /// # use db::activation_code::ActivationCode;
    /// #
    /// let mut activation_code = ActivationCode {
    ///     email: "test@example.com".to_string(),
    ///     code: 123456,
    ///     expiration_time: chrono::Local::now().checked_add_signed(time::Duration::minutes(30)).unwrap().naive_local(),
    ///     attempts: 0,
    /// };
    /// assert_eq!(activation_code.is_expired(), false);
    /// #
    /// # activation_code.expiration_time = chrono::Local::now().checked_sub_signed(time::Duration::seconds(1)).unwrap().naive_local();
    /// # assert_eq!(activation_code.is_expired(), true);
    /// # activation_code.expiration_time = chrono::Local::now().checked_add_signed(time::Duration::seconds(1)).unwrap().naive_local();
    /// # assert_eq!(activation_code.is_expired(), false);
    /// ```
    pub fn is_expired(&self) -> bool {
        self.expiration_time.lt(&chrono::Local::now().naive_local())
    }

    /// Returns whether or not the maximum number of activation attempts have been exceeded.
    ///
    /// # Example
    ///
    /// ```
    /// # use db::activation_code::ActivationCode;
    /// #
    /// let mut activation_code = ActivationCode {
    ///     email: "test@example.com".to_string(),
    ///     code: 123456,
    ///     expiration_time: chrono::Local::now().checked_add_signed(time::Duration::minutes(30)).unwrap().naive_local(),
    ///     attempts: 0,
    /// };
    ///
    /// for i in 0..5 {
    ///     activation_code.attempts = i;
    ///     assert_eq!(activation_code.attempts_exceeded(), false);
    /// }
    ///
    /// for i in 6..10 {
    ///     activation_code.attempts = i;
    ///     assert_eq!(activation_code.attempts_exceeded(), true);
    /// }
    /// ```
    pub fn attempts_exceeded(&self) -> bool {
        self.attempts.gt(&MAX_ATTEMPTS)
    }
}

// Possible errors thrown when handling activation codes.
#[derive(Debug, PartialEq)]
pub enum ActivationCodeErrorKind {
    // A user could not be activated due to a database error.
    ActivationFailed(UserErrorKind),
    // A new activation code could not be created due to a database error.
    CreationFailed(diesel::result::Error),
    // An activation code could not be deleted due to a database error.
    DeletionFailed(diesel::result::Error),
    // The expiration time overflowed. Not expected to occur before the end of the year 262143.
    ExpirationTimeOverflow,
    // The activation code has expired.
    Expired,
    // The activation code is invalid.
    InvalidCode,
    // The maximum number of attempts to retrieve or validate an activation code has been exceeded.
    MaxAttemptsExceeded,
    // Expired activation codes could not be purged due to a database error.
    PurgingFailed(diesel::result::Error),
    // An existing activation code could not be updated due to a database error.
    UpdateFailed(diesel::result::Error),
    // No activation code needs to be generated because the user has already been activated.
    UserAlreadyActivated(String),
}

impl fmt::Display for ActivationCodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ActivationCodeErrorKind::ActivationFailed(ref err) => {
                write!(f, "Database error when activating user: {}", err)
            }
            ActivationCodeErrorKind::CreationFailed(ref err) => {
                write!(f, "Database error when creating activation code: {}", err)
            }
            ActivationCodeErrorKind::DeletionFailed(ref err) => {
                write!(f, "Database error when deleting activation code: {}", err)
            }
            ActivationCodeErrorKind::Expired => {
                write!(f, "The activation code has expired")
            }
            ActivationCodeErrorKind::ExpirationTimeOverflow => {
                write!(f, "Expiration time overflow")
            }
            ActivationCodeErrorKind::InvalidCode => {
                write!(f, "Invalid activation code")
            }
            ActivationCodeErrorKind::MaxAttemptsExceeded => {
                write!(f, "The maximum number of allowed attempts to retrieve or validate an activation code has been exceeded. Please wait 30 minutes before requesting a new activation code.")
            }
            ActivationCodeErrorKind::PurgingFailed(ref err) => {
                write!(f, "Database error when purging expired activation codes: {}", err)
            }
            ActivationCodeErrorKind::UpdateFailed(ref err) => {
                write!(f, "Database error when updating activation code: {}", err)
            }
            ActivationCodeErrorKind::UserAlreadyActivated(ref email) => {
                write!(f, "The user with email {} is already activated", email)
            }
        }
    }
}

/// Returns an activation code for the given user.
pub fn get(
    connection: &PgConnection,
    user: &User,
) -> Result<ActivationCode, ActivationCodeErrorKind> {
    assert_not_activated(user)?;

    let email = user.email.as_str();
    match read(connection, email) {
        Some(c) => {
            if c.is_expired() {
                create(connection, email)
            } else {
                // If the activation code already exists, increase the attempts counter before
                // returning the code. This prevents an attacker flooding the user's inbox with
                // activation messages. Possibly returns a MaxAttemptsExceeded error.
                increase_attempt_counter(connection, c)
            }
        }
        None => create(connection, email),
    }
}

/// Activates the given user if the given activation code is valid.
pub fn activate_user(
    connection: &PgConnection,
    user: User,
    activation_code: i32,
) -> Result<User, ActivationCodeErrorKind> {
    assert_not_activated(&user)?;
    match read(connection, user.email.as_str()) {
        Some(c) => {
            if c.is_expired() {
                return Err(ActivationCodeErrorKind::Expired);
            }
            if c.attempts_exceeded() {
                return Err(ActivationCodeErrorKind::MaxAttemptsExceeded);
            }
            if c.code == activation_code {
                let user = super::user::activate(connection, user)
                    .map_err(ActivationCodeErrorKind::ActivationFailed)?;
                return Ok(user);
            }
            increase_attempt_counter(connection, c)?;
            Err(ActivationCodeErrorKind::InvalidCode)
        }
        // In normal usage (registering a user through the web interface) an activation code is
        // always generated. If none is returned then the code has expired and has been purged from
        // the database, so return an `Expired` error.
        None => Err(ActivationCodeErrorKind::Expired),
    }
}

/// Purges all expired activation codes.
pub fn purge(connection: &PgConnection) -> Result<(), ActivationCodeErrorKind> {
    let expiration_time = chrono::Local::now().naive_local();
    diesel::delete(dsl::activation_codes.filter(dsl::expiration_time.lt(expiration_time)))
        .execute(connection)
        .map_err(ActivationCodeErrorKind::PurgingFailed)?;
    Ok(())
}

/// Deletes the activation code for the given user.
pub fn delete(connection: &PgConnection, user: &User) -> Result<(), ActivationCodeErrorKind> {
    diesel::delete(dsl::activation_codes.filter(dsl::email.eq(user.email.as_str())))
        .execute(connection)
        .map_err(ActivationCodeErrorKind::DeletionFailed)?;
    Ok(())
}

// Retrieves the activation code for the user with the given email address.
//
// Returns raw data from the database which may be stale. Use `get()` instead, this is guaranteed to
// return a valid activation code when possible, and has protection against brute force attacks.
fn read(connection: &PgConnection, email: &str) -> Option<ActivationCode> {
    let activation_code = dsl::activation_codes
        .find(email)
        .first::<ActivationCode>(connection);
    match activation_code {
        Ok(c) => Some(c),
        Err(_) => None,
    }
}

// Creates an activation code.
//
// Creates a new activation code database record for the user with the given email address with the
// following data:
// - email: the user's email address.
// - code: a random number between 100000 and 999999.
// - expiration_time: a timestamp 30 minutes from now.
//
// If an existing record already exists for the given user it will be overwritten. It is recommended
// to use `get()` instead of this function; it will check if an existing non-expired activation code
// exists and return it if possible. It will create a new activation code only if no previous record
// exists, or it is expired.
fn create(
    connection: &PgConnection,
    email: &str,
) -> Result<ActivationCode, ActivationCodeErrorKind> {
    // Create a new activation code.
    let random_code = thread_rng().gen_range(MIN_VALUE, MAX_VALUE);
    let expiration_time =
        match chrono::Local::now().checked_add_signed(time::Duration::minutes(30)) {
            Some(t) => t,
            None => return Err(ActivationCodeErrorKind::ExpirationTimeOverflow),
        }
        .naive_local();

    // There can only be one activation code per user. Insert a new record or update an existing
    // record.
    diesel::insert_into(dsl::activation_codes)
        .values((
            dsl::email.eq(email),
            dsl::code.eq(random_code),
            dsl::expiration_time.eq(expiration_time),
            dsl::attempts.eq(0),
        ))
        .on_conflict(dsl::email)
        .do_update()
        .set((
            dsl::code.eq(random_code),
            dsl::expiration_time.eq(expiration_time),
            dsl::attempts.eq(0),
        ))
        .returning((dsl::email, dsl::code, dsl::expiration_time, dsl::attempts))
        .get_result(connection)
        .map_err(ActivationCodeErrorKind::CreationFailed)
}

// Increases the attempt counter.
//
// To prevent compromising a user account by brute forcing the activation code we only allow a
// limited number of validation attempts.
fn increase_attempt_counter(
    connection: &PgConnection,
    activation_code: ActivationCode,
) -> Result<ActivationCode, ActivationCodeErrorKind> {
    // If the number of attempts have already exceeded the limit previously, don't bother to
    // increase the counter but exit early.
    if activation_code.attempts_exceeded() {
        return Err(ActivationCodeErrorKind::MaxAttemptsExceeded);
    }

    let activation_code =
        diesel::update(dsl::activation_codes.filter(dsl::email.eq(activation_code.email.as_str())))
            .set(dsl::attempts.eq(dsl::attempts + 1))
            .returning((dsl::email, dsl::code, dsl::expiration_time, dsl::attempts))
            .get_result::<ActivationCode>(connection)
            .map_err(ActivationCodeErrorKind::UpdateFailed)?;

    if activation_code.attempts_exceeded() {
        return Err(ActivationCodeErrorKind::MaxAttemptsExceeded);
    }

    Ok(activation_code)
}

// Asserts that the given user is not activated.
fn assert_not_activated(user: &User) -> Result<(), ActivationCodeErrorKind> {
    if user.activated {
        return Err(ActivationCodeErrorKind::UserAlreadyActivated(
            user.email.clone(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{establish_connection, get_database_url, user};
    use app::AppConfig;
    use diesel::result::Error;

    // Tests super::get().
    #[test]
    fn test_get() {
        let connection = establish_connection(&get_database_url());
        let email = "test@example.com";
        let password = "mypass";
        let config = AppConfig::from_test_defaults();
        connection.test_transaction::<_, Error, _>(|| {
            // Create a test user.
            let user = user::create(&connection, email, password, &config).unwrap();

            // Initially, an activation code should not be present in the database.
            assert!(read(&connection, email).is_none());

            // Generate an activation code and check that it contains correct values.
            let activation_code = get(&connection, &user).unwrap();
            assert_activation_code(&activation_code, email, None, None, 0);

            // Check that a record now exists in the database.
            assert!(read(&connection, email).is_some());

            // We should be allowed to retrieve the activation code 5 more times, but any more
            // attempts should return an error.
            for attempt_count in 1..6 {
                // Check that the data in the newly retrieved activation code matches the original.
                let newly_retrieved = get(&connection, &user).unwrap();
                assert_activation_code(
                    &newly_retrieved,
                    &activation_code.email,
                    Some(activation_code.code),
                    Some(activation_code.expiration_time),
                    attempt_count,
                );
            }

            for _failed_attempt_count in 0..10 {
                assert_eq!(
                    ActivationCodeErrorKind::MaxAttemptsExceeded,
                    get(&connection, &user).unwrap_err()
                );
            }

            // Expire the activation code by updating the expired time.
            expire_activation_code(&connection, email);

            // Check that the activation code is now effectively expired, by reading the data
            // directly from the database.
            assert!(read(&connection, email).unwrap().is_expired());

            // When an activation code is expired and is again requested, a new activation code
            // should be generated and the attempts counter should be reset to 0.
            let fresh_activation_code = get(&connection, &user).unwrap();
            assert_activation_code(&fresh_activation_code, email, None, None, 0);
            assert_ne!(activation_code.code, fresh_activation_code.code);

            // Activate the user and request a new activation code. This should result in an
            // `UserAlreadyActivated` error.
            let user = user::activate(&connection, user).unwrap();
            assert_eq!(
                ActivationCodeErrorKind::UserAlreadyActivated(user.email.clone()),
                get(&connection, &user).unwrap_err()
            );

            // Request an activation code for a user that has not been saved in the database. This
            // should result in an error.
            let user = User {
                activated: false,
                email: "non-existing-user@example.com".to_string(),
                created: chrono::Local::now().naive_local(),
                password: "hunter2".to_string(),
            };
            // Todo: Check that this returns an `ActivationCodeErrorKind::CreationFailed()`.
            assert!(get(&connection, &user).is_err());

            Ok(())
        });
    }

    // Tests super::activate_user().
    #[test]
    fn test_activate_user() {
        let connection = establish_connection(&get_database_url());
        let email = "test@example.com";
        let password = "mypass";
        let config = AppConfig::from_test_defaults();
        connection.test_transaction::<_, Error, _>(|| {
            // Create a test user through the API. This does not cause an activation code to be
            // generated, as is happening in normal usage (i.e. through the web UI).
            let user = user::create(&connection, email, password, &config).unwrap();
            assert!(read(&connection, email).is_none());

            // In normal usage if an activation code is not present in the database for a non-
            // activated user this means that the activation code has expired and been purged from
            // the database. Check that calling `activate_user()` returns an `Expired` error.
            assert_eq!(
                ActivationCodeErrorKind::Expired,
                activate_user(&connection, user.clone(), 0).unwrap_err()
            );

            // Generate an activation code. It should initially have 0 attempts.
            let activation_code = get(&connection, &user).unwrap();
            assert_activation_code(&activation_code, email, None, None, 0);

            // Try activating using the wrong code. This should result 5 times in an `InvalidCode`
            // error, and any subsequent attempts should activate the brute force protection and
            // result in an `AttemptsExceeded` error, even if the correct code is passed.
            let wrong_code = activation_code.code + 1;

            for _i in 0..5 {
                assert_eq!(
                    ActivationCodeErrorKind::InvalidCode,
                    activate_user(&connection, user.clone(), wrong_code).unwrap_err()
                );
            }
            for _i in 5..10 {
                assert_eq!(
                    ActivationCodeErrorKind::MaxAttemptsExceeded,
                    activate_user(&connection, user.clone(), wrong_code).unwrap_err()
                );
            }

            // Once the brute force protection has been triggered an error should always be
            // returned, even when passing the correct activation code.
            assert_eq!(
                ActivationCodeErrorKind::MaxAttemptsExceeded,
                activate_user(&connection, user.clone(), activation_code.code).unwrap_err()
            );

            // Expire the activation code. It should then return an `Expired` error when trying to
            // activate, regardless of whether the correct or wrong code is passed.
            expire_activation_code(&connection, email);
            assert_eq!(
                ActivationCodeErrorKind::Expired,
                activate_user(&connection, user.clone(), activation_code.code).unwrap_err()
            );
            assert_eq!(
                ActivationCodeErrorKind::Expired,
                activate_user(&connection, user.clone(), wrong_code).unwrap_err()
            );

            // Get a fresh activation code, and activate the user using the correct code. This is
            // expected to return the activated user.
            let fresh_activation_code = get(&connection, &user).unwrap();
            let activated_user =
                activate_user(&connection, user, fresh_activation_code.code).unwrap();
            assert!(activated_user.activated);

            // Try to re-activate the user. We should now get a `UserAlreadyActivated` error.
            assert_eq!(
                ActivationCodeErrorKind::UserAlreadyActivated(activated_user.email.clone()),
                activate_user(&connection, activated_user, fresh_activation_code.code).unwrap_err()
            );

            Ok(())
        });
    }

    // Tests super::purge().
    #[test]
    fn test_purge() {
        let connection = establish_connection(&get_database_url());
        let password = "mypass";
        let config = AppConfig::from_test_defaults();
        connection.test_transaction::<_, Error, _>(|| {
            // Create some test users and activation codes.
            for i in 0..10 {
                let email = format!("test{}@example.com", i);
                user::create(&connection, email.as_str(), password, &config).unwrap();
                create(&connection, email.as_str()).unwrap();

                // The first 5 users will have a fresh activation code, while the last 5 have an
                // expired code.
                if i >= 5 {
                    expire_activation_code(&connection, email.as_str());
                }
            }

            // Before purging, all activation codes should be present in the database.
            for i in 0..10 {
                let email = format!("test{}@example.com", i);
                assert!(read(&connection, email.as_str()).is_some());
            }

            // After purging, the last 5 activation codes should no longer be present.
            assert!(purge(&connection).is_ok());
            for i in 0..5 {
                let email = format!("test{}@example.com", i);
                assert!(read(&connection, email.as_str()).is_some());
            }
            for i in 5..10 {
                let email = format!("test{}@example.com", i);
                assert!(read(&connection, email.as_str()).is_none());
            }

            Ok(())
        });
    }

    // Tests super::delete().
    #[test]
    fn test_delete() {
        let connection = establish_connection(&get_database_url());
        let email = "test@example.com";
        let password = "mypass";
        let config = AppConfig::from_test_defaults();
        connection.test_transaction::<_, Error, _>(|| {
            // Initially we do not expect to have a record for an activation code present in the
            // database for a user that is created through the API.
            let user = user::create(&connection, email, password, &config).unwrap();
            assert!(read(&connection, email).is_none());

            // Generate an activation code. Now there should be a record.
            assert!(get(&connection, &user).is_ok());
            assert!(read(&connection, email).is_some());

            // Delete the activation code. This should not result in an error, and the record should
            // no longer be present.
            assert!(delete(&connection, &user).is_ok());
            assert!(read(&connection, email).is_none());

            Ok(())
        });
    }

    // Tests super::read().
    #[test]
    fn test_read() {
        let connection = establish_connection(&get_database_url());
        let email = "test@example.com";
        let password = "mypass";
        let config = AppConfig::from_test_defaults();
        connection.test_transaction::<_, Error, _>(|| {
            // When no activation code is present yet, the `read()` function should return `None`.
            let user = user::create(&connection, email, password, &config).unwrap();
            assert!(read(&connection, email).is_none());

            // Generate an activation code and assert that the `read()` function returns it.
            assert!(get(&connection, &user).is_ok());
            let activation_code = read(&connection, email).unwrap();
            assert_activation_code(&activation_code, email, None, None, 0);

            // Expire the activation code. It should still be returned.
            expire_activation_code(&connection, email);
            let activation_code = read(&connection, email).unwrap();
            let expiration_time = chrono::Local::now().naive_local();
            assert_activation_code(&activation_code, email, None, Some(expiration_time), 0);

            // Delete the activation code. Now the `read()` function should return `None` again.
            assert!(delete(&connection, &user).is_ok());
            assert!(read(&connection, email).is_none());

            Ok(())
        });
    }

    // Tests super::create().
    #[test]
    fn test_create() {
        let connection = establish_connection(&get_database_url());
        let email1 = "test-user-1@example.com";
        let email2 = "test-user-2@example.com";
        let password1 = "mypass";
        let password2 = "abc123";
        let config = AppConfig::from_test_defaults();
        connection.test_transaction::<_, Error, _>(|| {
            // Create two test users.
            user::create(&connection, email1, password1, &config).unwrap();
            user::create(&connection, email2, password2, &config).unwrap();

            // Initially there should be no activation codes for the test users.
            assert!(read(&connection, email1).is_none());
            assert!(read(&connection, email2).is_none());

            // Create activation codes for the users and check that valid objects are returned.
            let activation_code_for_user_1 = create(&connection, email1).unwrap();
            assert_activation_code(&activation_code_for_user_1, email1, None, None, 0);
            let activation_code_for_user_2 = create(&connection, email2).unwrap();
            assert_activation_code(&activation_code_for_user_2, email2, None, None, 0);

            // Check that the activation codes are different for both users.
            // Todo: there is a 1/900000 chance that both activation codes are equal, so this might
            // cause a random failure.
            assert_ne!(activation_code_for_user_1.code, activation_code_for_user_2.code);

            // There should now be a database record for both activation codes that can be read from
            // the database. The object that is retrieved from the database should be identical to
            // the one returned by `create()`.
            assert_eq!(activation_code_for_user_1, read(&connection, email1).unwrap());
            assert_eq!(activation_code_for_user_2, read(&connection, email2).unwrap());

            // When a new activation code is created for a user it should overwrite the existing
            // one. It should have a different code than the previous one.
            // Todo: there is a 1/900000 chance that both activation codes are equal, so this might
            // cause a random failure.
            let new_activation_code_for_user_1 = create(&connection, email1).unwrap();
            assert_activation_code(&new_activation_code_for_user_1, email1, None, None, 0);
            assert_ne!(activation_code_for_user_1.code, new_activation_code_for_user_1.code);
            let new_activation_code_for_user_2 = create(&connection, email2).unwrap();
            assert_activation_code(&new_activation_code_for_user_2, email2, None, None, 0);
            assert_ne!(activation_code_for_user_2.code, new_activation_code_for_user_2.code);

            Ok(())
        });
    }

    // Tests super::increase_attempt_counter().
    #[test]
    fn test_increase_attempt_counter() {
        let connection = establish_connection(&get_database_url());
        let email = "test@example.com";
        let password = "mypass";
        let config = AppConfig::from_test_defaults();
        connection.test_transaction::<_, Error, _>(|| {
            // If we try to increase the attempt counter for an activation code that does not have a
            // matching database record then we should get an error.
            let user = user::create(&connection, email, password, &config).unwrap();
            let unsaved_activation_code = ActivationCode {
                email: email.to_string(),
                code: 123456,
                expiration_time: chrono::Local::now().checked_add_signed(time::Duration::minutes(30)).unwrap().naive_local(),
                attempts: 0,
            };
            assert!(increase_attempt_counter(&connection, unsaved_activation_code).is_err());

            // Generate an activation code. We should be able to increase the attempts counter 5
            // times, but all attempts after that should return an error.
            let mut activation_code = get(&connection, &user).unwrap();
            assert_eq!(0, activation_code.attempts);

            for i in 1..6 {
                activation_code = increase_attempt_counter(&connection, activation_code).unwrap();
                assert_eq!(i, activation_code.attempts);
                assert!(!activation_code.attempts_exceeded());
            }

            for _i in 6..99 {
                assert_eq!(
                    ActivationCodeErrorKind::MaxAttemptsExceeded,
                    increase_attempt_counter(&connection, activation_code.clone()).unwrap_err()
                );
            }

            // Tampering with the attempt counter of an activation code that has exceeded the number
            // of attempts should not be possible.
            activation_code.attempts = 0;
            assert_eq!(
                ActivationCodeErrorKind::MaxAttemptsExceeded,
                increase_attempt_counter(&connection, activation_code).unwrap_err()
            );

            Ok(())
        });
    }

    // Checks that the given activation code matches the given values.
    fn assert_activation_code(
        // The activation code to check.
        activation_code: &ActivationCode,
        // The expected email address.
        email: &str,
        // The expected activation code. If omitted the code will only be checked to see if it is
        // between MIN_VALUE and MAX_VALUE.
        code: Option<i32>,
        // The expected expiration time. If omitted this will default to 30 minutes in the future.
        // This will verify that the expiration time is within an interval of the given time and 2
        // seconds earlier, to account for the elapsed time between the creation of the database
        // record and the assertion.
        expiration_time: Option<chrono::NaiveDateTime>,
        // The expected value of the retry attempts counter.
        attempts: i16,
    ) {
        // Check the email address.
        assert_eq!(email.to_string(), activation_code.email);

        // Check the activation code.
        match code {
            Some(c) => {
                assert_eq!(c, activation_code.code);
            }
            None => {
                assert!(MIN_VALUE <= activation_code.code);
                assert!(activation_code.code <= MAX_VALUE);
            }
        }

        // Check the expiration time. If no expiration time is passed, default to to 30 minutes in
        // the future.
        let expiration_time = expiration_time.unwrap_or_else(|| {
            chrono::Local::now()
                .checked_add_signed(time::Duration::minutes(30))
                .unwrap()
                .naive_local()
        });

        // Check within a time interval of a few seconds, to account for elapsed time before the
        // assertion is called.
        let two_seconds_earlier = expiration_time
            .checked_sub_signed(time::Duration::seconds(2))
            .unwrap();
        assert!(activation_code.expiration_time <= expiration_time);
        assert!(activation_code.expiration_time > two_seconds_earlier);

        // Check the attempts counter.
        assert_eq!(attempts, activation_code.attempts);
    }

    // Expire the activation code for the given user by updating the expired time in the database.
    fn expire_activation_code(connection: &PgConnection, email: &str) {
        diesel::update(dsl::activation_codes.filter(dsl::email.eq(email)))
            .set(dsl::expiration_time.eq(chrono::Local::now().naive_local()))
            .execute(connection)
            .unwrap();
    }
}
