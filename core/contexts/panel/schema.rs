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
        twitter_username -> Nullable<Varchar>,
        facebook_username -> Nullable<Varchar>,
        discord_username -> Nullable<Varchar>,
        wallet_address -> Nullable<Varchar>,
        user_role -> Userrole,
        pswd -> Varchar,
        last_login -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
