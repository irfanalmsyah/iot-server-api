// @generated automatically by Diesel CLI.

diesel::table! {
    hardwares (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[sql_name = "type"]
        #[max_length = 255]
        type_ -> Varchar,
        description -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        status -> Nullable<Bool>,
        isadmin -> Nullable<Bool>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(hardwares, users,);
