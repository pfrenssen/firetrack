use super::schema::activation_codes;
use super::schema::activation_codes::dsl;
use super::user::User;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rand::{thread_rng, Rng};
use std::fmt;

// The minimum and maximum values for a random activation code.
const MIN_VALUE: i32 = 100000;
const MAX_VALUE: i32 = 999999;

// The maximum number of activations that can be attempted in 30 minutes.
const MAX_ATTEMPTS: i16 = 5;

#[derive(Associations, Debug, Queryable)]
#[belongs_to(User, foreign_key = "email")]
pub struct ActivationCode {
    pub email: String,
    pub code: i32,
    pub expiration_time: chrono::NaiveDateTime,
    pub attempts: i16,
}

impl ActivationCode {
    /// Returns whether or not the activation code is expired.
    /// Todo: unit test.
    pub fn is_expired(&self) -> bool {
        self.expiration_time.lt(&chrono::Local::now().naive_local())
    }

    /// Returns whether or not the maximum number of activation attempts have been exceeded.
    /// Todo: unit test.
    pub fn attempts_exceeded(&self) -> bool {
        self.attempts.gt(&MAX_ATTEMPTS)
    }
}

// Possible errors thrown when handling activation codes.
#[derive(Debug)]
pub enum ActivationCodeErrorKind {
    // A new activation code could not be created due to a database error.
    ActivationCodeCreationFailed(diesel::result::Error),
    // A new activation code could not be deleted due to a database error.
    ActivationCodeDeletionFailed(diesel::result::Error),
    // An existing activation code could not be updated due to a database error.
    ActivationCodeUpdateFailed(diesel::result::Error),
    // The expiration time overflowed. Not expected to occur before the end of the year 262143.
    ExpirationTimeOverflow,
    // The maximum number of attempts to retrieve or validate an activation code has been exceeded.
    MaxAttemptsExceeded,
    // No activation code needs to be generated because the user has already been activated.
    UserAlreadyActivated(String),
}

impl fmt::Display for ActivationCodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ActivationCodeErrorKind::ActivationCodeCreationFailed(ref err) => {
                write!(f, "Database error when creating activation code: {}", err)
            }
            ActivationCodeErrorKind::ActivationCodeDeletionFailed(ref err) => {
                write!(f, "Database error when deleting activation code: {}", err)
            }
            ActivationCodeErrorKind::ActivationCodeUpdateFailed(ref err) => {
                write!(f, "Database error when updating activation code: {}", err)
            }
            ActivationCodeErrorKind::ExpirationTimeOverflow => {
                write!(f, "Expiration time overflow")
            }
            ActivationCodeErrorKind::MaxAttemptsExceeded => {
                write!(f, "The maximum number of allowed attempts to retrieve or validate an activation code has been exceeded. Please wait 30 minutes before requesting a new activation code.")
            }
            ActivationCodeErrorKind::UserAlreadyActivated(ref email) => {
                write!(f, "The user with email {} is already activated", email)
            }
        }
    }
}

/// Returns an activation code for the given user.
/// Todo: rename the `validated` column in the users table to `activated`.
pub fn get(
    connection: &PgConnection,
    user: &User,
) -> Result<ActivationCode, ActivationCodeErrorKind> {
    if user.validated {
        return Err(ActivationCodeErrorKind::UserAlreadyActivated(
            user.email.clone(),
        ));
    }
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
/// Todo: increase the attempts on failure.
pub fn activate_user(user: &User, activation_code: u32) -> Result<(), ActivationCodeErrorKind> {
    Ok(())
}

/// Purges all expired activation codes.
pub fn purge() -> Result<(), ActivationCodeErrorKind> {
    Ok(())
}

/// Deletes the activation code for the given user.
pub fn delete(connection: &PgConnection, user: &User) -> Result<(), ActivationCodeErrorKind> {
    diesel::delete(dsl::activation_codes.filter(dsl::email.eq(user.email.as_str())))
        .execute(connection)
        .map_err(ActivationCodeErrorKind::ActivationCodeDeletionFailed);
    Ok(())
}

// Retrieves the activation code for the user with the given email address.
//
// Returns raw data from the database which may be stale. Use get_activation_code() instead, this is
// guaranteed to return a valid activation code when possible.
fn read(connection: &PgConnection, email: &str) -> Option<ActivationCode> {
    // Check if a non-expired activation code already exists.
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
// A valid record might already exist. Use
// get_activation_code() instead, this will return an existing activation code if a non-expired one
// exists, and will create a new one otherwise.
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
        .map_err(ActivationCodeErrorKind::ActivationCodeCreationFailed)
}

// Increases the attempt counter.
//
// To prevent compromising a user account by brute forcing the activation code we only allow a
// limited number of validation attempts.
fn increase_attempt_counter(
    connection: &PgConnection,
    activation_code: ActivationCode,
) -> Result<ActivationCode, ActivationCodeErrorKind> {
    let activation_code =
        diesel::update(dsl::activation_codes.filter(dsl::email.eq(activation_code.email.as_str())))
            .set((dsl::attempts.eq(activation_code.attempts + 1),))
            .returning((dsl::email, dsl::code, dsl::expiration_time, dsl::attempts))
            .get_result::<ActivationCode>(connection)
            .map_err(ActivationCodeErrorKind::ActivationCodeUpdateFailed)?;

    if activation_code.attempts_exceeded() {
        return Err(ActivationCodeErrorKind::MaxAttemptsExceeded);
    }

    Ok(activation_code)
}
