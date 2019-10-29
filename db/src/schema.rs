table! {
    users (email) {
        email -> Varchar,
        password -> Varchar,
        created -> Timestamp,
        validated -> Bool,
    }
}
