



/*
    
    all APIs in here are based on the access levels 
    which are defined for this app, thus based on 
    those access levels we must have three services 
    including admin, user and dev with registered APIs
    
*/



use crate::*;



/*
     --------------------------------
    |        REGISTER WS ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_ws_notif(config: &mut web::ServiceConfig){

    config.service(apis::notifs::exports::notif_subs);
    config.service(apis::notifs::exports::chatroomlp);
    
    // other routs maybe ?
    // ...


}

/*
     --------------------------------
    |        REGISTER DEV ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_dev(config: &mut web::ServiceConfig){

    config.service(apis::dev::exports::get_admin_data);
    config.service(apis::dev::exports::get_user_data);
    
    // other routs maybe ?
    // ...


}

/*
     --------------------------------
    |      REGISTER ADMIN ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_admin(config: &mut web::ServiceConfig){
    
    config.service(apis::admin::exports::reveal_role);
    config.service(apis::admin::exports::update_rendezvous_event_img);
    config.service(apis::admin::exports::login);
    config.service(apis::admin::exports::register_new_user);
    config.service(apis::admin::exports::register_new_task);
    config.service(apis::admin::exports::delete_task);
    config.service(apis::admin::exports::edit_task);
    config.service(apis::admin::exports::edit_user);
    config.service(apis::admin::exports::delete_user);
    config.service(apis::admin::exports::start_tcp_server);
    config.service(apis::admin::exports::get_users);
    config.service(apis::admin::exports::get_admin_tasks);
    config.service(apis::admin::exports::get_users_tasks);
    config.service(apis::admin::exports::add_twitter_account);
    config.service(apis::admin::exports::get_all_users_deposits);
    config.service(apis::admin::exports::get_all_users_checkouts);
    config.service(apis::admin::exports::get_all_users_withdrawals);

    // other routs maybe ?
    // ...


}

/*
     --------------------------------
    |       REGISTER USER ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_user(config: &mut web::ServiceConfig){
    
    config.service(apis::user::exports::login);
    config.service(apis::user::exports::login_with_identifier_and_password);
    config.service(apis::user::exports::login_with_gmail);
    config.service(apis::user::exports::login_with_microsoft);
    config.service(apis::user::exports::verify_twitter_account);
    config.service(apis::user::exports::tasks_report);
    config.service(apis::user::exports::make_cid);
    config.service(apis::user::exports::deposit);
    config.service(apis::user::exports::withdraw);
    config.service(apis::user::exports::get_all_user_withdrawals);
    config.service(apis::user::exports::get_all_user_deposits);
    config.service(apis::user::exports::get_recipient_unclaimed_deposits);
    config.service(apis::user::exports::get_all_user_paid_checkouts);
    config.service(apis::user::exports::get_all_user_unpaid_checkouts);
    config.service(apis::user::exports::request_mail_code);
    config.service(apis::user::exports::verify_mail_code);
    config.service(apis::user::exports::request_phone_code);
    config.service(apis::user::exports::verify_phone_code);
    config.service(apis::user::exports::charge_wallet_request);
    config.service(apis::user::exports::edit_bio);
    config.service(apis::user::exports::upload_avatar);
    config.service(apis::user::exports::upload_banner);
    config.service(apis::user::exports::upload_wallet_back);
    config.service(apis::user::exports::upload_rendezvous_player_avatar);
    config.service(apis::user::exports::create_private_gallery);
    config.service(apis::user::exports::update_private_gallery);
    config.service(apis::user::exports::get_all_private_galleries_for);
    config.service(apis::user::exports::get_all_galleries_invited_to);
    config.service(apis::user::exports::get_all_public_collections_for);
    config.service(apis::user::exports::get_all_private_collections_for);
    config.service(apis::user::exports::get_invited_friends_wallet_data_of_gallery);
    config.service(apis::user::exports::send_private_gallery_invitation_request_to);
    config.service(apis::user::exports::remove_invited_friend_from_gallery);
    config.service(apis::user::exports::get_all_public_collection_nfts);
    config.service(apis::user::exports::accept_invitation_request);
    config.service(apis::user::exports::get_user_unaccpeted_invitation_requests);
    config.service(apis::user::exports::get_user_unaccpeted_friend_requests);
    config.service(apis::user::exports::accept_friend_request);
    config.service(apis::user::exports::send_friend_request_to);
    config.service(apis::user::exports::remove_user_from_friend);
    config.service(apis::user::exports::get_all_user_fans_data_for);
    config.service(apis::user::exports::create_collection);
    config.service(apis::user::exports::update_collection);
    config.service(apis::user::exports::create_nft);
    config.service(apis::user::exports::update_nft);
    config.service(apis::user::exports::buy_nft);
    config.service(apis::user::exports::mint_nft);
    config.service(apis::user::exports::add_reaction_to_nft);
    config.service(apis::user::exports::get_all_user_reactions);
    config.service(apis::user::exports::get_all_nft_reactions);
    config.service(apis::user::exports::get_all_nfts_owned_by);
    config.service(apis::user::exports::upload_collection_banner);
    config.service(apis::user::exports::create_nft_metadata_uri);
    
    // other routs maybe ?
    // ...


}

/*
     --------------------------------
    |     REGISTER HEALTH ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_health(config: &mut web::ServiceConfig){

    config.service(apis::health::exports::index);
    config.service(apis::health::exports::check_token);
    config.service(apis::health::exports::get_tasks);
    config.service(apis::health::exports::logout);
    config.service(apis::health::update_user_balance_webhook);
    config.service(apis::health::is_user_kyced);

    // other routs maybe ?
    // ...


}

/*
     --------------------------------
    |     REGISTER PUBLIC ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_public(config: &mut web::ServiceConfig){

    config.service(apis::public::exports::verify_twitter_task);
    config.service(apis::public::exports::check_users_task);
    config.service(apis::public::exports::get_token_value);
    config.service(apis::public::exports::get_x_requests);
    config.service(apis::public::tasks_leaderboard);
    config.service(apis::public::get_user_wallet_info);

    // other routs maybe ?
    // ...


}