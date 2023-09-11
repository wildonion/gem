// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "userregion"))]
    pub struct Userregion;

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
        task_priority -> Int4,
        hashtag -> Varchar,
        tweet_content -> Varchar,
        retweet_id -> Varchar,
        like_tweet_id -> Varchar,
        admin_id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Userregion;
    use super::sql_types::Userrole;

    users (id) {
        id -> Int4,
        region -> Userregion,
        username -> Varchar,
        activity_code -> Varchar,
        twitter_username -> Nullable<Varchar>,
        facebook_username -> Nullable<Varchar>,
        discord_username -> Nullable<Varchar>,
        identifier -> Nullable<Varchar>,
        mail -> Nullable<Varchar>,
        is_mail_verified -> Bool,
        is_phone_verified -> Bool,
        phone_number -> Nullable<Varchar>,
        paypal_id -> Nullable<Varchar>,
        account_number -> Nullable<Varchar>,
        device_id -> Nullable<Varchar>,
        social_id -> Nullable<Varchar>,
        cid -> Nullable<Varchar>,
        screen_cid -> Nullable<Varchar>,
        snowflake_id -> Nullable<Int8>,
        stars -> Nullable<Int8>,
        user_role -> Userrole,
        pswd -> Varchar,
        token_time -> Nullable<Int8>,
        last_login -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_deposits (id) {
        id -> Int4,
        mint_tx_hash -> Varchar,
        nft_id -> Varchar,
        from_cid -> Varchar,
        recipient_screen_cid -> Varchar,
        is_claimed -> Bool,
        amount -> Int8,
        tx_signature -> Varchar,
        iat -> Timestamptz,
    }
}

diesel::table! {
    users_mails (id) {
        id -> Int4,
        user_id -> Int4,
        mail -> Varchar,
        code -> Varchar,
        exp -> Int8,
        vat -> Int8,
    }
}

diesel::table! {
    users_phones (id) {
        id -> Int4,
        user_id -> Int4,
        phone -> Varchar,
        code -> Varchar,
        exp -> Int8,
        vat -> Int8,
    }
}

diesel::table! {
    users_tasks (user_id, task_id) {
        user_id -> Int4,
        task_id -> Int4,
        done_at -> Timestamptz,
    }
}

diesel::table! {
    users_withdrawals (id) {
        id -> Int4,
        deposit_id -> Int4,
        burn_tx_hash -> Varchar,
        recipient_cid -> Varchar,
        tx_signature -> Varchar,
        wat -> Timestamptz,
    }
}

diesel::joinable!(tasks -> users (admin_id));
diesel::joinable!(users_tasks -> tasks (task_id));
diesel::joinable!(users_tasks -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    tasks,
    users,
    users_deposits,
    users_mails,
    users_phones,
    users_tasks,
    users_withdrawals,
);
