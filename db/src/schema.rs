table! {
    activation_codes (email) {
        email -> Varchar,
        code -> Int4,
        expiration_time -> Timestamp,
        attempts -> Int2,
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

joinable!(activation_codes -> users (email));

allow_tables_to_appear_in_same_query!(activation_codes, users,);
