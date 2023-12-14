


use crate::*;
use crate::adapters::stripe::StripeWebhookPayload;
use crate::models::users_checkouts::UserCheckout;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use s3req::Storage;
use passport::Passport;
use crate::models::users::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::models::{tasks::*, users_tasks::*};




/*
     ------------------------
    |        SCHEMAS
    | ------------------------
    |
    |

*/
#[derive(Serialize, Deserialize, Clone)]
pub struct Health{
    pub status: String,
}


/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/
#[get("/check-server")]
#[passport(admin, user, dev)]
async fn index(
        req: HttpRequest,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

        let iam_healthy = Health{
            status: "🥞 Alive".to_string()
        };
    
        resp!{
            Health, // the data type
            iam_healthy, // response data
            IAM_HEALTHY, // response message
            StatusCode::OK, // status code
            None::<Cookie<'_>>,
        }

}

#[get("/check-token")]
#[passport(admin, user, dev)]
async fn check_token(
        req: HttpRequest,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let single_user = users
                        .filter(id.eq(_id))
                        .first::<User>(connection);


                    let Ok(user) = single_user else{
                        resp!{
                            i32, // the data type
                            _id, // response data
                            USER_NOT_FOUND, // response message
                            StatusCode::NOT_FOUND, // status code
                            None::<Cookie<'_>>,
                        } 
                    };

                    let user_data = UserData { 
                        id: user.id, 
                        region: user.region.clone(),
                        username: user.username, 
                        bio: user.bio.clone(),
                        avatar: user.avatar.clone(),
                        banner: user.banner.clone(),
                        wallet_background: user.wallet_background.clone(),
                        activity_code: user.activity_code,
                        twitter_username: user.twitter_username, 
                        facebook_username: user.facebook_username, 
                        discord_username: user.discord_username, 
                        identifier: user.identifier, 
                        user_role: {
                            match user.user_role.clone(){
                                UserRole::Admin => "Admin".to_string(),
                                UserRole::User => "User".to_string(),
                                _ => "Dev".to_string(),
                            }
                        },
                        token_time: user.token_time,
                        balance: user.balance,
                        last_login: { 
                            if user.last_login.is_some(){
                                Some(user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: user.created_at.to_string(),
                        updated_at: user.updated_at.to_string(),
                        mail: user.mail,
                        google_id: user.google_id,
                        microsoft_id: user.microsoft_id,
                        is_mail_verified: user.is_mail_verified,
                        is_phone_verified: user.is_phone_verified,
                        phone_number: user.phone_number,
                        paypal_id: user.paypal_id,
                        account_number: user.account_number,
                        device_id: user.device_id,
                        social_id: user.social_id,
                        cid: user.cid,
                        screen_cid: user.screen_cid,
                        snowflake_id: user.snowflake_id,
                        stars: user.stars,
                        extra: user.extra,
                    };


                    /* sending pg response */
                    resp!{
                        UserData, // the data type
                        user_data, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>,
                    }
                    

                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>,
            }
        }
    }

}

#[post("/logout")]
#[passport(admin, user, dev)]
async fn logout(
        req: HttpRequest,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {


    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
        /* 
             ------------------------------------- 
            | --------- PASSPORT CHECKING --------- 
            | ------------------------------------- 
            | granted_role has been injected into this 
            | api body using #[passport()] proc macro 
            | at compile time thus we're checking it
            | at runtime
            |
        */
        let granted_role = 
            if granted_roles.len() == 3{ /* everyone can pass */
                None /* no access is required perhaps it's an public route! */
            } else if granted_roles.len() == 1{
                match granted_roles[0]{ /* the first one is the right access */
                    "admin" => Some(UserRole::Admin),
                    "user" => Some(UserRole::User),
                    _ => Some(UserRole::Dev)
                }
            } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
                resp!{
                    &[u8], // the data type
                    &[], // response data
                    ACCESS_DENIED, // response message
                    StatusCode::FORBIDDEN, // status code
                    None::<Cookie<'_>>, // cookie
                }
            };

            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* 
                        🔐 logout supports also for jwt only, it sets the token time field 
                        inside the users table related to the logged in user to 0, this wiill 
                        be checked inside the **passport** function to see that the token time 
                        inside the passed in jwt to the request header must be the one 
                        inside the users table
                    */
                    match User::logout(_id, redis_client.to_owned(), redis_actix_actor, connection).await{
                        Ok(_) => {
                            
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                LOGOUT, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>,
                            }
        
                        },
                        Err(resp) => {
            
                            /* DIESEL UPDATE ERROR RESPONSE */
                            resp
        
                        }
                    }

                },
                Err(resp) => {
                    
                    /* 
                        🥝 based on the flow response can be one of the following:
                        
                        - NOT_FOUND_TOKEN
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND 
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                        - NOT_FOUND_JWT_VALUE
                    */
                    resp
                }
            }

        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }

        }
    }
        

}

#[get("/get-tasks/")]
#[passport(admin, user, dev)]
async fn get_tasks(
        req: HttpRequest,  
        limit: web::Query<Limit>,
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            

            /* 
                 ------------------------------------- 
                | --------- PASSPORT CHECKING --------- 
                | ------------------------------------- 
                | granted_role has been injected into this 
                | api body using #[passport()] proc macro 
                | at compile time thus we're checking it
                | at runtime
                |
            */
            let granted_role = 
                if granted_roles.len() == 3{ /* everyone can pass */
                    None /* no access is required perhaps it's an public route! */
                } else if granted_roles.len() == 1{
                    match granted_roles[0]{ /* the first one is the right access */
                        "admin" => Some(UserRole::Admin),
                        "user" => Some(UserRole::User),
                        _ => Some(UserRole::Dev)
                    }
                } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
                    resp!{
                        &[u8], // the data type
                        &[], // response data
                        ACCESS_DENIED, // response message
                        StatusCode::FORBIDDEN, // status code
                        None::<Cookie<'_>>, // cookie
                    }
                };


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match Task::get_all(limit, connection).await{
                        Ok(all_tasks) => {

                            resp!{
                                Vec<TaskData>, // the data type
                                all_tasks, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        },
                        Err(resp) => {

                            /* DIESEL FETCH ERROR RESPONSE */
                            resp
                        }
                    }
                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        
        }, 
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }         


}

#[post("/cid/wallet/stripe/update/balance/webhook/{session_id}/{payment_intent}")]
#[passport(admin, user, dev)]
async fn update_user_balance_webhook(
        req: HttpRequest,
        params: web::Path<(String, String)>,
        storage: web::Data<Option<Arc<Storage>>>
    ) -> PanelHttpResponse{

    /* 
        stripe event handler and webhook subscriber to the new success checkout session event
        webhook means once an event gets triggered an api call will be invoked to notify (it's 
        like a notification to the server) server about the event happend as a result of handling 
        another process in some where like a payment result in which server subscribes to incoming 
        event type and can publish it to redispubsub so other app, threads and scopes can also 
        subscribe to it or charge an in-app token balance of a user like the following logic
    */

    /* extracting shared state data */
    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().unwrap().get_redis().await.unwrap();
    let async_redis_client = storage.as_ref().unwrap().get_async_redis_pubsub_conn().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();

            let session_id = params.clone().0;
            let payment_intent = params.clone().1;
            let stripe_webhook_signature = env::var("STRIPE_WEBHOOK_SIGNATURE").unwrap();
            let webhook_event_signature = req.headers().get("stripe-signature").unwrap().to_str().unwrap();
            if &stripe_webhook_signature != webhook_event_signature{

                resp!{
                    &[u8], // the data type
                    &[], // response data
                    STRIPE_INVALID_WEBHOOK_SIGNATURE, // response message
                    StatusCode::EXPECTATION_FAILED, // status code
                    None::<Cookie<'_>>, // cookie
                }
            }

            match UserCheckout::update(&session_id, &payment_intent, connection).await{
                Ok(updated_user_checkout) => {
                    
                    /* update the user balance */
                    let find_user_screen_cid = User::find_by_screen_cid(&walletreq::evm::get_keccak256_from(updated_user_checkout.user_cid.clone()), connection).await;
                        let Ok(user_info) = find_user_screen_cid else{
                            
                            resp!{
                                String, // the data type
                                updated_user_checkout.user_cid, // response data
                                &USER_SCREEN_CID_NOT_FOUND, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

                    let new_balance = if user_info.balance.is_none(){0 + updated_user_checkout.tokens} else{user_info.balance.unwrap() + updated_user_checkout.tokens};
                    match User::update_balance(user_info.id, new_balance, redis_client.to_owned(), redis_actix_actor, connection).await{

                        Ok(updated_user_data) => {

                            resp!{
                                UserData, // the data type
                                updated_user_data, // response data
                                PAID_SUCCESSFULLY, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        },
                        Err(resp) => {
                            resp
                        }
                    }

                },
                Err(resp) => {
                    resp
                }
            }

        },
        None => {
            
            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}

#[post("/am-i-kyced")]
#[passport(admin, user, dev)]
async fn is_user_kyced(
        req: HttpRequest,  
        check_kyc_request: web::Json<CheckKycRequest>,
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    

    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();

            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let check_kyc_request = check_kyc_request.to_owned();
                        
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*
                        followings are the param 
                        must be passed to do the 
                        kyc process on request data
                        @params:
                            - _id              : user id
                            - from_cid         : user crypto id
                            - tx_signature     : tx signature signed
                            - hash_data        : sha256 hash of data generated in client app
                            - deposited_amount : the amount of token must be deposited for this call
                    */
                    let is_request_verified = kyced::verify_request(
                        _id, 
                        &check_kyc_request.caller_cid, 
                        &check_kyc_request.tx_signature, 
                        &check_kyc_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };
                    
                    let user_data = UserWalletInfoResponse{
                        username: user.username,
                        avatar: user.avatar,
                        mail: user.mail,
                        screen_cid: user.screen_cid,
                        stars: user.stars,
                        created_at: user.created_at.to_string(),
                        bio: user.bio,
                        banner: user.banner,
                        extra: user.extra,
                    };

                    /* sending user data */
                    resp!{
                        UserWalletInfoResponse, // the data type
                        user_data, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>,
                    }
                    

                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }
        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>,
            }
        }
    }

}



pub mod exports{
    pub use super::index;
    pub use super::check_token;
    pub use super::logout;
    pub use super::get_tasks;
    pub use super::update_user_balance_webhook;
    pub use super::is_user_kyced;
}