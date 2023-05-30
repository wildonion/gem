// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "userrole"))]
    pub struct Userrole;
}

diesel::table! {
    tasks (id) {
        id -> Int4,
        task_name -> Varchar,
        task_description -> Nullable<Varchar>,
        task_score -> Int4,
        admin_id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
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
        token_time -> Nullable<Int8>,
        last_login -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_tasks (user_id, task_id) {
        user_id -> Int4,
        task_id -> Int4,
        done_at -> Timestamptz,
    }
}

diesel::joinable!(tasks -> users (admin_id));
diesel::joinable!(users_tasks -> tasks (task_id));
diesel::joinable!(users_tasks -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    tasks,
    users,
    users_tasks,
);
