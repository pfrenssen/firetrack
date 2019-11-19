table! {
    activation_code (email) {
        email -> Varchar,
        activation_code -> Int4,
        expiration_time -> Timestamp,
        attempts -> Nullable<Int2>,
    }
}

table! {
    users (email) {
        email -> Varchar,
        password -> Varchar,
        created -> Timestamp,
        validated -> Bool,
    }
}

joinable!(activation_code -> users (email));

allow_tables_to_appear_in_same_query!(
    activation_code,
    users,
);
