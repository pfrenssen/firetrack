table! {
    activation_codes (id) {
        id -> Int4,
        code -> Int4,
        expiration_time -> Timestamp,
        attempts -> Int2,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        password -> Varchar,
        created -> Timestamp,
        activated -> Bool,
    }
}

joinable!(activation_codes -> users (id));

allow_tables_to_appear_in_same_query!(activation_codes, users,);
