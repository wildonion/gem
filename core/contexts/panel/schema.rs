// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        twitter_username -> Varchar,
        wallet_address -> Varchar,
        user_role -> Varchar,
        last_login -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
