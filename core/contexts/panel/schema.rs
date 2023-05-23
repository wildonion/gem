// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "userrole"))]
    pub struct Userrole;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Userrole;

    users (id) {
        id -> Int4,
        username -> Varchar,
        twitter_username -> Varchar,
        facebook_username -> Varchar,
        discord_username -> Varchar,
        wallet_address -> Varchar,
        user_role -> Userrole,
        pswd -> Varchar,
        last_login -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
