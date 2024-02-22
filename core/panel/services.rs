



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

    config.service(apis::rom::exports::sub_to_rom);
    config.service(apis::clp::exports::chatroomlp);


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
    config.service(apis::admin::exports::get_users);
    config.service(apis::admin::exports::get_admin_tasks);
    config.service(apis::admin::exports::get_users_tasks);
    config.service(apis::admin::exports::add_twitter_account);
    config.service(apis::admin::exports::get_all_users_deposits);
    config.service(apis::admin::exports::get_all_users_checkouts);
    config.service(apis::admin::exports::get_all_users_withdrawals);
    config.service(apis::admin::exports::get_clp_event);
    config.service(apis::admin::exports::start_new_clp_event);
    config.service(apis::admin::exports::update_clp_event);
    config.service(apis::admin::exports::get_all_clp_event);
    config.service(apis::admin::exports::update_clp_event_back);
    config.service(apis::admin::exports::send_mail);


}

/*
     --------------------------------
    |       REGISTER USER ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_user(config: &mut web::ServiceConfig){
    
    config.service(apis::user::exports::get_token_value);
    config.service(apis::user::exports::get_gas_fee);
    config.service(apis::user::exports::login_with_identifier_and_password);
    config.service(apis::user::exports::signup_with_identifier_and_password);
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
    config.service(apis::user::exports::edit_extra);
    config.service(apis::user::exports::upload_avatar);
    config.service(apis::user::exports::update_password);
    config.service(apis::user::exports::upload_banner);
    config.service(apis::user::exports::upload_wallet_back);
    config.service(apis::user::exports::upload_private_gallery_back);
    config.service(apis::user::exports::upload_rendezvous_player_avatar);
    config.service(apis::user::exports::create_private_gallery);
    config.service(apis::user::exports::update_private_gallery);
    config.service(apis::user::exports::get_all_private_galleries_for);
    config.service(apis::user::exports::get_all_private_galleries_general_info_for);
    config.service(apis::user::exports::get_all_galleries_invited_to);
    config.service(apis::user::exports::get_all_public_collections_for);
    config.service(apis::user::exports::get_all_private_collections_for);
    config.service(apis::user::exports::get_all_private_collections_for_invited_friends);
    config.service(apis::user::exports::get_friend_suggestions_for_owner);
    config.service(apis::user::exports::get_invited_friends_wallet_data_of_gallery);
    config.service(apis::user::exports::send_private_gallery_invitation_request_to);
    config.service(apis::user::exports::remove_invited_friend_from_gallery);
    config.service(apis::user::exports::exit_from_private_gallery);
    config.service(apis::user::exports::get_all_public_collection_nfts);
    config.service(apis::user::exports::accept_invitation_request);
    config.service(apis::user::exports::enter_private_gallery);
    config.service(apis::user::exports::get_user_unaccepted_invitation_requests);
    config.service(apis::user::exports::get_user_unaccepted_friend_requests);
    config.service(apis::user::exports::accept_friend_request);
    config.service(apis::user::exports::send_friend_request_to);
    config.service(apis::user::exports::remove_user_from_follower);
    config.service(apis::user::exports::remove_user_from_friend);
    config.service(apis::user::exports::remove_user_from_following);
    config.service(apis::user::exports::get_all_my_friends);
    config.service(apis::user::exports::get_all_my_followers);
    config.service(apis::user::exports::get_all_my_followings);
    config.service(apis::user::exports::create_nft);
    config.service(apis::user::exports::update_nft);
    config.service(apis::user::exports::buy_nft);
    config.service(apis::user::exports::mint_nft);
    config.service(apis::user::exports::add_reaction_to_nft);
    config.service(apis::user::exports::get_all_user_relations);
    config.service(apis::user::exports::get_all_nft_reactions);
    config.service(apis::user::exports::get_all_nfts_owned_by);
    config.service(apis::user::exports::get_all_collections_owned_by);
    config.service(apis::user::exports::create_nft_metadata_uri);
    config.service(apis::user::exports::get_new_clp_event_info);
    config.service(apis::user::exports::get_all_user_clp_events_info);
    config.service(apis::user::exports::register_clp_event);
    config.service(apis::user::exports::cancel_clp_event);
    config.service(apis::user::exports::get_notifications);
    config.service(apis::user::exports::session_oauth_google);
    config.service(apis::user::exports::get_top_users);
    config.service(apis::user::exports::create_collection);
    config.service(apis::user::exports::update_collection);
    config.service(apis::user::exports::upload_collection_banner);
    config.service(apis::user::exports::get_nfts_owned_by);
    config.service(apis::user::exports::get_single_nft);


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
    config.service(apis::health::exports::update_user_balance_webhook);
    config.service(apis::health::exports::is_user_kyced);
    config.service(apis::health::exports::forgot_password);

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
    config.service(apis::public::exports::get_x_requests);
    config.service(apis::public::exports::tasks_leaderboard);
    config.service(apis::public::exports::get_user_wallet_info);
    config.service(apis::public::exports::get_users_wallet_info);
    config.service(apis::public::exports::search);
    config.service(apis::public::exports::get_top_nfts);
    config.service(apis::public::exports::get_all_nfts);
    config.service(apis::public::exports::get_nft_product_collections);
    config.service(apis::public::exports::test_stream);

}