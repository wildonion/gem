



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
    config.service(apis::admin::exports::update_event_img);
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
    config.service(apis::user::exports::verify_twitter_account);
    config.service(apis::user::exports::tasks_report);
    config.service(apis::user::exports::make_cid);
    config.service(apis::user::exports::deposit);
    config.service(apis::user::exports::withdraw);
    config.service(apis::user::exports::get_all_user_withdrawals);
    config.service(apis::user::exports::get_all_user_deposits);
    config.service(apis::user::exports::get_recipient_unclaimed_deposits);
    config.service(apis::user::exports::request_mail_code);
    config.service(apis::user::exports::verify_mail_code);
    config.service(apis::user::exports::request_phone_code);
    config.service(apis::user::exports::verify_phone_code);
    config.service(apis::user::exports::buy_token);
    config.service(apis::user::exports::burn_token);
    
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
    config.service(apis::public::exports::check_users_tassk);

    // other routs maybe ?
    // ...


}