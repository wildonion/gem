// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "userrole"))]
    pub struct Userrole;
}

diesel::table! {
    clp_events (id) {
        id -> Int4,
        contract_address -> Varchar,
        nfts -> Nullable<Jsonb>,
        event_name -> Varchar,
        symbol -> Varchar,
        max_supply -> Int4,
        team_reserve -> Int4,
        mint_price -> Int8,
        presale_mint_price -> Int8,
        tokens_per_mint -> Int4,
        owner_screen_cid -> Varchar,
        treasury_address -> Varchar,
        public_mint_start_date -> Varchar,
        metadata_updatable -> Nullable<Bool>,
        freeze_metadata -> Nullable<Bool>,
        base_uri -> Varchar,
        presale_mint_start_date -> Varchar,
        presale_whitelisted_addresses -> Nullable<Array<Nullable<Text>>>,
        prereveal_token_uri -> Varchar,
        royalties_share -> Int4,
        royalties_address_screen_cid -> Varchar,
        event_background -> Varchar,
        extra -> Nullable<Jsonb>,
        event_description -> Varchar,
        contract_tx_hash -> Nullable<Varchar>,
        start_at -> Int8,
        expire_at -> Int8,
        is_locked -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
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
    use super::sql_types::Userrole;

    users (id) {
        id -> Int4,
        region -> Nullable<Varchar>,
        username -> Varchar,
        bio -> Nullable<Varchar>,
        avatar -> Nullable<Varchar>,
        banner -> Nullable<Varchar>,
        wallet_background -> Nullable<Varchar>,
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
        balance -> Nullable<Int8>,
        last_login -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_chats (id) {
        id -> Int4,
        clp_event_id -> Int4,
        user_id -> Int4,
        content -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_checkouts (id) {
        id -> Int4,
        user_cid -> Varchar,
        product_id -> Varchar,
        price_id -> Varchar,
        payment_status -> Varchar,
        payment_intent -> Varchar,
        c_status -> Varchar,
        checkout_session_url -> Varchar,
        checkout_session_id -> Varchar,
        checkout_session_expires_at -> Int8,
        tokens -> Int8,
        usd_token_price -> Int8,
        tx_signature -> Varchar,
        iat -> Timestamptz,
    }
}

diesel::table! {
    users_clps (id) {
        id -> Int4,
        clp_event_id -> Int4,
        user_id -> Int4,
        entry_amount -> Nullable<Int8>,
        registered_at -> Timestamptz,
        joined_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_collections (id) {
        id -> Int4,
        contract_address -> Varchar,
        nfts -> Nullable<Jsonb>,
        col_name -> Varchar,
        symbol -> Varchar,
        owner_screen_cid -> Varchar,
        metadata_updatable -> Nullable<Bool>,
        freeze_metadata -> Nullable<Bool>,
        base_uri -> Varchar,
        royalties_share -> Int4,
        royalties_address_screen_cid -> Varchar,
        collection_background -> Varchar,
        extra -> Nullable<Jsonb>,
        col_description -> Varchar,
        contract_tx_hash -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_deposits (id) {
        id -> Int4,
        mint_tx_hash -> Varchar,
        nft_id -> Varchar,
        nft_img_url -> Varchar,
        from_cid -> Varchar,
        recipient_screen_cid -> Varchar,
        is_claimed -> Bool,
        amount -> Int8,
        tx_signature -> Varchar,
        iat -> Timestamptz,
    }
}

diesel::table! {
    users_fans (id, user_screen_cid) {
        id -> Int4,
        user_screen_cid -> Varchar,
        friends -> Nullable<Jsonb>,
        invitation_requests -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_galleries (id) {
        id -> Int4,
        owner_screen_cid -> Varchar,
        collections -> Nullable<Jsonb>,
        gal_name -> Varchar,
        gal_description -> Varchar,
        invited_friends -> Nullable<Array<Nullable<Text>>>,
        extra -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
    users_nfts (id) {
        id -> Int4,
        contract_address -> Varchar,
        current_owner_screen_cid -> Varchar,
        metadata_uri -> Varchar,
        onchain_id -> Nullable<Varchar>,
        nft_name -> Varchar,
        nft_description -> Varchar,
        is_minted -> Nullable<Bool>,
        current_price -> Nullable<Int8>,
        is_listed -> Nullable<Bool>,
        freeze_metadata -> Nullable<Bool>,
        extra -> Nullable<Jsonb>,
        attributes -> Nullable<Jsonb>,
        comments -> Nullable<Jsonb>,
        likes -> Nullable<Jsonb>,
        tx_hash -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
        transfer_tx_hash -> Varchar,
        recipient_cid -> Varchar,
        tx_signature -> Varchar,
        wat -> Timestamptz,
    }
}

diesel::joinable!(tasks -> users (admin_id));
diesel::joinable!(users_chats -> clp_events (clp_event_id));
diesel::joinable!(users_chats -> users (user_id));
diesel::joinable!(users_clps -> clp_events (clp_event_id));
diesel::joinable!(users_clps -> users (user_id));
diesel::joinable!(users_tasks -> tasks (task_id));
diesel::joinable!(users_tasks -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    clp_events,
    tasks,
    users,
    users_chats,
    users_checkouts,
    users_clps,
    users_collections,
    users_deposits,
    users_fans,
    users_galleries,
    users_mails,
    users_nfts,
    users_phones,
    users_tasks,
    users_withdrawals,
);
