table! {
    activation_codes (id) {
        id -> Int4,
        code -> Int4,
        expiration_time -> Timestamp,
        attempts -> Int2,
    }
}

table! {
    categories (id) {
        id -> Int4,
        name -> Varchar,
        description -> Nullable<Varchar>,
        user_id -> Int4,
        parent_id -> Nullable<Int4>,
    }
}

table! {
    expenses (id) {
        id -> Int4,
        amount -> Numeric,
        description -> Nullable<Varchar>,
        category_id -> Int4,
        user_id -> Int4,
        date -> Timestamp,
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
joinable!(categories -> users (user_id));
joinable!(expenses -> categories (category_id));
joinable!(expenses -> users (user_id));

allow_tables_to_appear_in_same_query!(
    activation_codes,
    categories,
    expenses,
    users,
);
