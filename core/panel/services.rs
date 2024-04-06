



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

    config.service(apis::rom::subscribe::exports::sub_to_rom);
    config.service(apis::clp::chat::exports::chatroomlp);


}

/*
     --------------------------------
    |        REGISTER DEV ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_dev(config: &mut web::ServiceConfig){

    config.service(apis::dev::get_data::exports::get_admin_data);
    config.service(apis::dev::get_data::exports::get_user_data);


}

/*
     --------------------------------
    |      REGISTER ADMIN ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_admin(config: &mut web::ServiceConfig){
    
    config.service(apis::admin::rendezvous::exports::reveal_role);
    config.service(apis::admin::rendezvous::exports::update_rendezvous_event_img);
    config.service(apis::admin::auth::exports::login);
    config.service(apis::admin::user::exports::register_new_user);
    config.service(apis::admin::task::exports::register_new_task);
    config.service(apis::admin::task::exports::delete_task);
    config.service(apis::admin::task::exports::edit_task);
    config.service(apis::admin::user::exports::edit_user);
    config.service(apis::admin::user::exports::delete_user);
    config.service(apis::admin::user::exports::get_users);
    config.service(apis::admin::task::exports::get_admin_tasks);
    config.service(apis::admin::task::exports::get_users_tasks);
    config.service(apis::admin::x::exports::add_twitter_account);
    config.service(apis::admin::wallet::exports::get_all_users_deposits);
    config.service(apis::admin::wallet::exports::get_all_users_checkouts);
    config.service(apis::admin::wallet::exports::get_all_users_withdrawals);
    config.service(apis::admin::clp::exports::get_clp_event);
    config.service(apis::admin::clp::exports::start_new_clp_event);
    config.service(apis::admin::clp::exports::update_clp_event);
    config.service(apis::admin::clp::exports::get_all_clp_event);
    config.service(apis::admin::clp::exports::update_clp_event_back);
    config.service(apis::admin::mail::exports::send_mail);
    config.service(apis::admin::ticket::exports::get_all);
    config.service(apis::admin::token::exports::get_all);


}

/*
     --------------------------------
    |       REGISTER USER ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_user(config: &mut web::ServiceConfig){
    
    config.service(apis::user::wallet::exports::get_token_value);
    config.service(apis::user::wallet::exports::get_gas_fee);
    config.service(apis::user::auth::exports::login_with_identifier_and_password);
    config.service(apis::user::auth::exports::signup_with_identifier_and_password);
    config.service(apis::user::x::exports::verify_twitter_account);
    config.service(apis::user::task::exports::tasks_report);
    config.service(apis::user::wallet::exports::make_cid);
    config.service(apis::user::wallet::exports::deposit);
    config.service(apis::user::wallet::exports::withdraw);
    config.service(apis::user::wallet::exports::get_all_user_withdrawals);
    config.service(apis::user::wallet::exports::get_all_user_deposits);
    config.service(apis::user::wallet::exports::get_recipient_unclaimed_deposits);
    config.service(apis::user::wallet::exports::get_all_user_paid_checkouts);
    config.service(apis::user::wallet::exports::get_all_user_unpaid_checkouts);
    config.service(apis::user::auth::exports::request_mail_code);
    config.service(apis::user::auth::exports::verify_mail_code);
    config.service(apis::user::auth::exports::request_phone_code);
    config.service(apis::user::auth::exports::verify_phone_code);
    config.service(apis::user::wallet::exports::charge_wallet_request);
    config.service(apis::user::profile::exports::edit_bio);
    config.service(apis::user::profile::exports::edit_extra);
    config.service(apis::user::profile::exports::upload_avatar);
    config.service(apis::user::profile::exports::update_password);
    config.service(apis::user::profile::exports::upload_banner);
    config.service(apis::user::profile::exports::upload_wallet_back);
    config.service(apis::user::gallery::exports::upload_private_gallery_back);
    config.service(apis::user::rendezvous::exports::upload_rendezvous_player_avatar);
    config.service(apis::user::gallery::exports::create_private_gallery);
    config.service(apis::user::gallery::exports::update_private_gallery);
    config.service(apis::user::gallery::exports::get_all_private_galleries_for);
    config.service(apis::user::gallery::exports::get_all_private_galleries_general_info_for);
    config.service(apis::user::gallery::exports::get_all_galleries_invited_to);
    config.service(apis::user::gallery::exports::get_all_public_collections_for);
    config.service(apis::user::gallery::exports::get_all_private_collections_for);
    config.service(apis::user::gallery::exports::get_all_private_collections_for_invited_friends);
    config.service(apis::user::friend::exports::get_friend_suggestions_for_owner);
    config.service(apis::user::gallery::exports::get_invited_friends_wallet_data_of_gallery);
    config.service(apis::user::gallery::exports::send_private_gallery_invitation_request_to);
    config.service(apis::user::gallery::exports::remove_invited_friend_from_gallery);
    config.service(apis::user::gallery::exports::exit_from_private_gallery);
    config.service(apis::user::gallery::exports::get_all_public_collection_nfts);
    config.service(apis::user::gallery::exports::accept_invitation_request);
    config.service(apis::user::gallery::exports::enter_private_gallery);
    config.service(apis::user::gallery::exports::search_in_invited_friends_wallet_data_of_gallery);
    config.service(apis::user::gallery::exports::search_in_all_galleries_invited_to);
    config.service(apis::user::gallery::exports::get_user_unaccepted_invitation_requests);
    config.service(apis::user::friend::exports::get_user_unaccepted_friend_requests);
    config.service(apis::user::friend::exports::accept_friend_request);
    config.service(apis::user::friend::exports::send_friend_request_to);
    config.service(apis::user::friend::exports::remove_user_from_follower);
    config.service(apis::user::friend::exports::remove_user_from_friend);
    config.service(apis::user::friend::exports::remove_user_from_following);
    config.service(apis::user::friend::exports::search_in_followers);
    config.service(apis::user::friend::exports::search_in_followings);
    config.service(apis::user::friend::exports::search_in_friends);
    config.service(apis::user::friend::exports::search_in_unaccepted_friend_request);
    config.service(apis::user::friend::exports::search_in_friend_suggestions_for_owner);
    config.service(apis::user::friend::exports::search_in_all_user_relations);
    config.service(apis::user::friend::exports::get_all_my_friends);
    config.service(apis::user::friend::exports::get_all_my_followers);
    config.service(apis::user::friend::exports::get_all_my_followings);
    config.service(apis::user::gallery::exports::create_nft);
    config.service(apis::user::gallery::exports::create_nft_with_pic);
    config.service(apis::user::gallery::exports::update_nft);
    config.service(apis::user::gallery::exports::buy_nft);
    config.service(apis::user::gallery::exports::mint_nft);
    config.service(apis::user::gallery::exports::add_reaction_to_nft);
    config.service(apis::user::friend::exports::get_all_user_relations);
    config.service(apis::user::gallery::exports::get_all_nft_reactions);
    config.service(apis::user::gallery::exports::get_all_nfts_owned_by);
    config.service(apis::user::gallery::exports::get_all_collections_owned_by);
    config.service(apis::user::gallery::exports::create_nft_metadata_uri);
    config.service(apis::user::clp::exports::get_new_clp_event_info);
    config.service(apis::user::clp::exports::get_all_user_clp_events_info);
    config.service(apis::user::clp::exports::register_clp_event);
    config.service(apis::user::clp::exports::cancel_clp_event);
    config.service(apis::user::profile::exports::get_notifications);
    config.service(apis::user::auth::exports::session_oauth_google);
    config.service(apis::user::leaderboard::exports::get_top_users);
    config.service(apis::user::gallery::exports::create_collection);
    config.service(apis::user::gallery::exports::update_collection);
    config.service(apis::user::gallery::exports::upload_collection_banner);
    config.service(apis::user::gallery::exports::get_nfts_owned_by);
}

/*
     --------------------------------
    |     REGISTER HEALTH ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_health(config: &mut web::ServiceConfig){

    config.service(apis::health::index::exports::index);
    config.service(apis::health::check::exports::check_token);
    config.service(apis::health::task::exports::get_tasks);
    config.service(apis::health::logout::exports::logout);
    config.service(apis::health::webhook::exports::update_user_balance_webhook);
    config.service(apis::health::check::exports::is_user_kyced);
    config.service(apis::health::password::exports::forgot_password);

}

/*
     --------------------------------
    |     REGISTER PUBLIC ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_public(config: &mut web::ServiceConfig){

    config.service(apis::public::x::exports::verify_twitter_task);
    config.service(apis::public::task::exports::check_users_task);
    config.service(apis::public::x::exports::get_x_requests);
    config.service(apis::public::task::exports::tasks_leaderboard);
    config.service(apis::public::wallet::exports::get_user_wallet_info);
    config.service(apis::public::wallet::exports::get_users_wallet_info);
    config.service(apis::public::search::exports::search);
    config.service(apis::public::search::exports::search_in_top_nfts);
    config.service(apis::public::blockchain::exports::get_top_nfts);
    config.service(apis::public::blockchain::exports::get_all_nfts);
    config.service(apis::public::blockchain::exports::get_public_collection);
    config.service(apis::public::blockchain::exports::get_single_nft);
    config.service(apis::public::blockchain::exports::get_nft_product_collections);
    config.service(apis::public::stream::exports::test_stream);
    config.service(apis::public::ticket::exports::send);

}