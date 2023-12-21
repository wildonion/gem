


use crate::*;
use crate::adapters::stripe::{create_product, create_price, create_session, StripeCreateCheckoutSessionData};
use crate::events::subscribers::handlers::actors::notif::pg::{PgListenerActor};
use crate::events::subscribers::handlers::actors::notif::system::{SystemActor, GetSystemUsersMap};
use crate::models::clp_events::ClpEventData;
use crate::models::users_checkouts::{UserCheckoutData, UserCheckout, NewUserCheckout};
use crate::models::users_clps::UserClp;
use crate::models::users_collections::{UserCollection, UserCollectionData, NewUserCollectionRequest, UpdateUserCollectionRequest};
use crate::models::users_deposits::UserDepositData;
use crate::models::users_fans::{InvitationRequestDataResponse, AcceptInvitationRequest, UserFanData, UserFan, AcceptFriendRequest, InvitationRequestData, SendFriendRequest, FriendData, UserRelations, EnterPrivateGalleryRequest, RemoveFriend, RemoveFollower};
use crate::models::users_galleries::{UserPrivateGalleryInfoDataInvited, NewUserPrivateGalleryRequest, UpdateUserPrivateGalleryRequest, UserPrivateGallery, UserPrivateGalleryData, RemoveInvitedFriendFromPrivateGalleryRequest, SendInvitationRequest, UserPrivateGalleryInfoData, ExitFromPrivateGalleryRequest};
use crate::models::users_nfts::{UserNftData, NewUserNftRequest, UpdateUserNftRequest, UserNft, UserReactionData, NftReactionData, AddReactionRequest, CreateNftMetadataUriRequest};
use crate::models::users_withdrawals::{UserWithdrawal, UserWithdrawalData};
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::passport::Passport; /* loading Passport macro to use get_user() method on HttpRequest object */
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use actix::Addr;
use s3req::Storage;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use futures_util::TryStreamExt;
use crate::*;
use crate::models::users::UserRole;
use crate::constants::*;
use crate::misc::*;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use models::users::{Id, NewIdRequest, UserIdResponse};
use models::users_deposits::{NewUserDepositRequest, UserDeposit};
use models::users_withdrawals::NewUserWithdrawRequest;
 
use crate::adapters::nftport::*;





/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/

/* 
    >-------------------------------------------------------------------------
    There are access and refresh tokens in cookie response in form of 
        /accesstoken={access_token:}&accesstoken_time={time_hash_hex_string:}&refrestoken={refresh_token:} 
    once the access token gets expired we can pass refresh token into 
    the request header in place of access token to get a new set of 
    keys on behalf of user, instead of redirecting client to the 
    login page again.
*/
#[post("/login/{identifier}")]
async fn login(
        req: HttpRequest, 
        login_identifier: web::Path<String>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();


            let check_refresh_token = req.check_refresh_token(connection);
            let Ok(user) = check_refresh_token else{
                let err_resp = check_refresh_token.unwrap_err();
                return err_resp;
            };

            if user.id != 0{

                info!("generating new set of token with refresh token for user with id: {}", user.id);
                return user.get_user_data_response_with_cookie(redis_client.clone(), redis_actix_actor, connection).await.unwrap();

            }

            match User::find_by_identifier(&login_identifier.to_owned(), connection).await{
                Ok(user) => user.get_user_data_response_with_cookie(redis_client.clone(), redis_actix_actor, connection).await.unwrap(),
                Err(resp) => {

                    /* USER NOT FOUND response */
                    // resp
                    
                    /* gently, we'll insert this user into table */
                    match User::insert(login_identifier.to_owned(), connection).await{
                        Ok((user_login_data, cookie)) => {

                            resp!{
                                UserData, // the data type
                                user_login_data, // response data
                                REGISTERED, // response message
                                StatusCode::CREATED, // status code,
                                Some(cookie), // cookie 
                            } 

                        },
                        Err(resp) => {
                            
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                                - CANT_GENERATE_COOKIE
                            */
                            resp
                        }
                    }

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

#[post("/request-mail-code/{mail}")]
#[passport(user)]
async fn request_mail_code(
    req: HttpRequest,
    user_mail: web::Path<String>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
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


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-request-mail-code", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "request_mail_code");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        match User::send_mail_verification_code_to(_id, user_mail.to_owned(), redis_client.to_owned(), redis_actix_actor, connection).await{
                            
                            Ok(updated_user) => {
    
                                resp!{
                                    UserData, // the data type
                                    updated_user, // response data
                                    MAIL_VERIFICATION_CODE_SENT, // response message
                                    StatusCode::OK, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
    
                            },
                            Err(resp) => {
    
                                /* 
                                    🥝 response can be one of the following:
    
                                    - USER NOT FOUND RESPONE
                                    - MAIL CLIENT ERROR
                                */
                                resp
    
                            }
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

#[post("/verify-mail-code")]
#[passport(user)]
async fn verify_mail_code(
    req: HttpRequest,
    check_user_verification_request: web::Json<CheckUserMailVerificationRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
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


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match User::check_mail_verification_code(check_user_verification_request.to_owned(), _id, redis_actix_actor, connection).await{
                        
                        Ok(updated_user) => {

                            resp!{
                                UserData, // the data type
                                updated_user, // response data
                                MAIL_VERIFIED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        },
                        Err(resp) => {

                            /* 
                                🥝 response can be one of the following:

                                - USER NOT FOUND RESPONE
                                - MAIL CLIENT ERROR
                            */
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

#[post("/request-phone-code/{phone}")]
#[passport(user)]
async fn request_phone_code(
    req: HttpRequest,
    user_phone: web::Path<String>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();
    
    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            let mut user_ip = "".to_string();

            /* ---------------------------------------------------------------------------------
                if we're getting 127.0.0.1 for client ip addr from the incoming request means
                the address 127.0.0.1 is the loopback address, which means the request is 
                coming from the same machine where the server is running. if we're running 
                both the server and the browser on the same computer and we're connecting 
                to localhost or 127.0.0.1 in the browser, then this behavior is expected.
                if Actix application is behind a reverse proxy like Nginx or Apache, the proxy 
                may be forwarding requests to your application in such a way that all client 
                connections appear to come from the loopback address. to fix this issue and get 
                the original client's IP address, you can use the X-Forwarded-For or X-Real-IP 
                headers. These headers are typically set by the reverse proxy to indicate the 
                original IP address of the client, also we have to make sure that these are set
                inside the nginx config file:

                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
               ---------------------------------------------------------------------------------
            */
            if let Some(header) = req.headers().get("X-Forwarded-For") {
                if let Ok(ip_str) = header.to_str() {
                    user_ip = ip_str.to_string();
                }
            }


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

                    /* we need rate limit in this api since otp providers have rate limits */
                    let identifier_key = format!("{}-request-phone-code", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "request_phone_code");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                        
                        let get_user = User::find_by_id(_id, connection).await;
                        let Ok(user) = get_user else{
                            let get_user_err = get_user.unwrap_err();
                            return get_user_err; /* user not found response */
                        };
                        
                      
                        match User::send_phone_verification_code_to(_id, user_phone.to_owned(), user_ip.clone(), redis_actix_actor, connection).await{
                            
                            Ok(updated_user) => {
    
                                resp!{
                                    UserData, // the data type
                                    updated_user, // response data
                                    PHONE_VERIFICATION_CODE_SENT, // response message
                                    StatusCode::OK, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
    
                            },
                            Err(resp) => {
    
                                /* 
                                    🥝 response can be one of the following:
    
                                    - USER NOT FOUND RESPONE
                                    - MAIL CLIENT ERROR
                                */
                                resp
    
                            }
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

#[post("/verify-phone-code")]
#[passport(user)]
async fn verify_phone_code(
    req: HttpRequest,
    check_user_verification_request: web::Json<CheckUserPhoneVerificationRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
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


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match User::check_phone_verification_code(check_user_verification_request.to_owned(), _id, redis_actix_actor, connection).await{
                        
                        Ok(updated_user) => {

                            resp!{
                                UserData, // the data type
                                updated_user, // response data
                                PHONE_VERIFIED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        },
                        Err(resp) => {

                            /* 
                                🥝 response can be one of the following:

                                - USER NOT FOUND RESPONE
                                - MAIL CLIENT ERROR
                            */
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

/* 
    >-------------------------------------------------------------------------
    There are access and refresh tokens in cookie response in form of 
        /accesstoken={access_token:}&accesstoken_time={time_hash_hex_string:}&refrestoken={refresh_token:} 
    once the access token gets expired we can pass refresh token into 
    the request header in place of access token to get a new set of 
    keys on behalf of user, instead of redirecting client to the 
    login page again.
*/
#[post("/login")]
async fn login_with_identifier_and_password(
        req: HttpRequest, 
        user_login_info: web::Json<UserLoginInfoRequest>,
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            let check_refresh_token = req.check_refresh_token(connection);
            let Ok(user) = check_refresh_token else{
                let err_resp = check_refresh_token.unwrap_err();
                return err_resp;
            };

            if user.id != 0{

                info!("generating new set of token with refresh token for admin with id: {}", user.id);
                return user.get_user_data_response_with_cookie(redis_client.clone(), redis_actix_actor, connection).await.unwrap();

            }

            let login_info = user_login_info.to_owned();

            match User::find_by_identifier(&login_info.identifier.to_owned(), connection).await{
                Ok(user) => {

                    let pswd_verification = user.verify_pswd(&login_info.password); 
                    let Ok(pswd_flag) = pswd_verification else{
                        let err_msg = pswd_verification.unwrap_err();
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            &err_msg.to_string(), // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }
                    };

                    if !pswd_flag{
                        resp!{
                            String, // the data type
                            login_info.identifier, // response data
                            WRONG_PASSWORD, // response message
                            StatusCode::FORBIDDEN, // status code
                            None::<Cookie<'_>>, // cookie
                        }
                    }
        
                    user.get_user_data_response_with_cookie(redis_client.clone(), redis_actix_actor, connection).await.unwrap()
                },
                Err(resp) => {

                    /* USER NOT FOUND response */
                    // resp
                    
                    /* gently, we'll insert this user into table */
                    match User::insert_by_identifier_password(login_info.identifier, login_info.password, connection).await{
                        Ok((user_login_data, cookie)) => {

                            resp!{
                                UserData, // the data type
                                user_login_data, // response data
                                REGISTERED, // response message
                                StatusCode::CREATED, // status code,
                                Some(cookie), // cookie 
                            } 

                        },
                        Err(resp) => {
                            
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                                - CANT_GENERATE_COOKIE
                            */
                            resp
                        }
                    }

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

#[post("/login/gmail")]
async fn login_with_gmail(
        req: HttpRequest,
        user_gmail_request: web::Json<UserLoginWithGmailRequest>,
        storage: web::Data<Option<Arc<Storage>>>,
    ) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned();
    match storage.unwrap().clone().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();

            let user_gmail_request = user_gmail_request.to_owned();

            
            // User::login_by_gmail_info();
            todo!()

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

#[post("/login/microsoft")]
async fn login_with_microsoft(
        req: HttpRequest,
        user_microsoft_request: web::Json<UserLoginWithMicrosoftRequest>,
        storage: web::Data<Option<Arc<Storage>>>,
    ) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned();
    match storage.unwrap().clone().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();

            let user_microsoft_request = user_microsoft_request.to_owned();
            
            // User::login_by_microsoft_info();
            todo!()

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

#[post("/verify-twitter-account/{account_name}")]
#[passport(user)]
async fn verify_twitter_account(
        req: HttpRequest,
        account_name: web::Path<String>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

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

                    /* rate limiter based on doer id */
                    let chill_zone_duration = 30_000u64; //// 30 seconds chillzone
                    let now = chrono::Local::now().timestamp_millis() as u64;
                    let mut is_rate_limited = false;
                    
                    let redis_result_verify_x_username_rate_limiter: RedisResult<String> = redis_conn.get("verify_x_username_rate_limiter").await;
                    let mut redis_verify_x_username_rate_limiter = match redis_result_verify_x_username_rate_limiter{
                        Ok(data) => {
                            let rl_data = serde_json::from_str::<HashMap<u64, u64>>(data.as_str()).unwrap();
                            rl_data
                        },
                        Err(e) => {
                            let empty_verify_x_username_rate_limiter = HashMap::<u64, u64>::new();
                            let rl_data = serde_json::to_string(&empty_verify_x_username_rate_limiter).unwrap();
                            let _: () = redis_conn.set("verify_x_username_rate_limiter", rl_data).await.unwrap();
                            HashMap::new()
                        }
                    };

                    if let Some(last_used) = redis_verify_x_username_rate_limiter.get(&(_id as u64)){
                        if now - *last_used < chill_zone_duration{
                            is_rate_limited = true;
                        }
                    }
                    
                    if is_rate_limited{
                    
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            TWITTER_VERIFICATION_RATE_LIMIT, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else{

                        /* updating the last rquest time */
                        //// this will be used to handle shared state between clusters
                        redis_verify_x_username_rate_limiter.insert(_id as u64, now); //// updating the redis rate limiter map
                        let rl_data = serde_json::to_string(&redis_verify_x_username_rate_limiter).unwrap();
                        let _: () = redis_conn.set("verify_x_username_rate_limiter", rl_data).await.unwrap(); //// writing to redis ram


        
                        match User::update_social_account(_id, &account_name.to_owned(), connection).await{
                            Ok(updated_user) => {
                    
                                resp!{
                                    UserData, // the data type
                                    updated_user, // response data
                                    UPDATED, // response message
                                    StatusCode::OK, // status code,
                                    None::<Cookie<'_>>, // cookie 
                                } 
                            },
                            Err(resp) => {
    
                                /* USER NOT FOUND response */
                                resp
                            }
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

#[get("/report-tasks/{user_id}/")]
#[passport(user)]
pub async fn tasks_report(
        req: HttpRequest,
        limit: web::Query<Limit>,
        user_id: web::Path<i32>,  
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


                    match UserTask::reports(user_id.to_owned(), limit, connection).await{
                        Ok(user_stask_reports) => {

                            resp!{
                                FetchUserTaskReport, // the data type
                                user_stask_reports, // response data
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

#[post("/cid/wallet/stripe/charge")]
#[passport(user)]
async fn charge_wallet_request(
    req: HttpRequest,
    charge_wallet_request: web::Json<ChargeWalletRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-charge-wallet-request", _id);

                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "charge_wallet_request");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {

                        let charge_wallet_request_object = charge_wallet_request.to_owned();
                        
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
                            &charge_wallet_request_object.buyer_cid, 
                            &charge_wallet_request_object.tx_signature, 
                            &charge_wallet_request_object.hash_data, 
                            None, /* no need to charge the user for this call */
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        if charge_wallet_request_object.tokens < 0 &&
                            charge_wallet_request_object.tokens < 5{

                                resp!{
                                    i32, // the data type
                                    _id, // response data
                                    INVALID_TOKEN_AMOUNT, // response message
                                    StatusCode::NOT_ACCEPTABLE, // status code
                                    None::<Cookie<'_>>, // cookie
                                }

                            }

                        if user.region.is_none(){

                            resp!{
                                i32, // the data type
                                _id, // response data
                                REGION_IS_NONE, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        }

                        let u_region = user.region.as_ref().unwrap();
                        let token_price = gastracker::calculate_token_value(charge_wallet_request_object.tokens, redis_client.clone()).await;

                        match u_region.as_str(){
                            "ir" => {

                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    NOT_IMPLEMENTED, // response message
                                    StatusCode::NOT_IMPLEMENTED, // status code
                                    None::<Cookie<'_>>, // cookie
                                }

                            },
                            _ => {

                                /* means that the api call can't return prices */
                                if token_price.0 == 0{
                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        CANT_GET_TOKEN_VALUE, // response message
                                        StatusCode::EXPECTATION_FAILED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
                                }
                                /* 
                                    stripe will divide the amount by 100 in checkout page to get the precision 
                                    like if we have 50000 this will show $500 in checkout page as the default price
                                */
                                let usd_token_price = token_price.0; 

                                /*  -------------------------------------------------------------
                                    note that we don't store the product, price and session data
                                    response came from stripe in db cause later on we can fetch 
                                    a single data of either product, price or checkout session 
                                    using stripe api or can be viewable in stripe dashboard
                                    ------------------------------------------------------------- */
                                let product = create_product(
                                    redis_client.clone(), 
                                    usd_token_price, 
                                    charge_wallet_request.tokens, 
                                    &charge_wallet_request.buyer_cid
                                ).await;

                                if product.id.is_none(){
                                    
                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        STRIPE_PRODUCT_OBJECT_ISSUE, // response message
                                        StatusCode::EXPECTATION_FAILED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
                                }

                                let price_id = create_price(
                                    redis_client.clone(), 
                                    usd_token_price, 
                                    &product.id.as_ref().unwrap(),
                                ).await;

                                if price_id.is_empty(){

                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        STRIPE_PRICE_OBJECT_ISSUE, // response message
                                        StatusCode::EXPECTATION_FAILED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
                                }

                                let checkout_session_data = create_session(
                                    redis_client.clone(), 
                                    &price_id, 
                                    charge_wallet_request.tokens,
                                    /*  
                                        since calling unwrap() takes the ownership of the object
                                        and the type will be dropped from the ram, thus if the type
                                        is being used in other scopes it's better to borrow it 
                                        or clone it which we've used as_ref() to borrow it also if 
                                        the user is here means that he has definitely the cid cause 
                                        to build cid we need verified mail
                                    */
                                    user.region.as_ref().unwrap(),
                                    user.mail.as_ref().unwrap() 
                                ).await;

                                if checkout_session_data.session_id.is_empty() || 
                                    checkout_session_data.session_url.is_empty() ||
                                    checkout_session_data.expires_at == 0{

                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        STRIPE_SESSION_OBJECT_ISSUE, // response message
                                        StatusCode::EXPECTATION_FAILED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
                                }

                                /* store users_checkouts data */
                                let new_user_checkout = NewUserCheckout{
                                    user_cid: charge_wallet_request_object.buyer_cid,
                                    product_id: product.id.unwrap(),
                                    price_id,
                                    payment_status: checkout_session_data.payment_status,
                                    payment_intent: checkout_session_data.payment_intent,
                                    c_status: checkout_session_data.status,
                                    checkout_session_url: checkout_session_data.session_url,
                                    checkout_session_id: checkout_session_data.session_id,
                                    checkout_session_expires_at: checkout_session_data.expires_at,
                                    tokens: charge_wallet_request_object.tokens,
                                    usd_token_price,
                                    tx_signature: charge_wallet_request_object.tx_signature,
                                };
                                match UserCheckout::insert(new_user_checkout, connection).await{
                                    Ok(user_checkout_data) => {

                                        resp!{
                                            UserCheckoutData, // the data type
                                            user_checkout_data, // response data
                                            STRIPE_STARTED_PAYAMENT, // response message
                                            StatusCode::OK, // status code
                                            None::<Cookie<'_>>, // cookie
                                        }
                                    },
                                    Err(resp) => {
                                        resp
                                    }
                                }

                            }
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

#[post("/cid/build")]
#[passport(user)]
async fn make_cid(
    req: HttpRequest,
    id_: web::Json<NewIdRequest>,
    storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let new_object_id_request = id_.0;
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();
    
    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            let mut user_ip = "".to_string();

            /* ---------------------------------------------------------------------------------
                if we're getting 127.0.0.1 for client ip addr inside the incoming request means
                the address 127.0.0.1 is the loopback address, which means the request is 
                coming from the same machine where the server is running. if we're running 
                both the server and the browser on the same computer and we're connecting 
                to localhost or 127.0.0.1 in the browser, then this behavior is expected.
                if Actix application is behind a reverse proxy like Nginx or Apache, the proxy 
                may be forwarding requests to your application in such a way that all client 
                connections appear to come from the loopback address. to fix this issue and get 
                the original client's IP address, you can use the X-Forwarded-For or X-Real-IP 
                headers. These headers are typically set by the reverse proxy to indicate the 
                original IP address of the client, also we have to make sure that these are set
                inside the nginx config file:

                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
               ---------------------------------------------------------------------------------
            */
            if let Some(header) = req.headers().get("X-Forwarded-For") {
                if let Ok(ip_str) = header.to_str() {
                    user_ip = ip_str.to_string();
                }
            }

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

                    /* check that the user has a verified mail or not */
                    let get_user = User::find_by_id(_id, connection).await;
                    let Ok(user) = get_user else{
                        
                        let err_resp = get_user.unwrap_err();
                        return err_resp;
                    };

                    /* if the mail wasn't verified user can't create id */
                    if user.mail.is_none() || 
                        !user.is_mail_verified{
                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            NOT_VERIFIED_MAIL, // response message
                            StatusCode::NOT_ACCEPTABLE, // status code
                            None::<Cookie<'_>>, // cookie
                        }
                    }

                    /* ------------------------------------------------------------ */
                    /* ------------------ NO NEED TO BE VERIFIED ------------------ */
                    /* ------------------------------------------------------------ */
                    /* if the phone wasn't verified user can't create id */
                    // if user.phone_number.is_none() || 
                    //     !user.is_phone_verified{
                    //     resp!{
                    //         &[u8], // the date type
                    //         &[], // the data itself
                    //         NOT_VERIFIED_PHONE, // response message
                    //         StatusCode::NOT_ACCEPTABLE, // status code
                    //         None::<Cookie<'_>>, // cookie
                    //     }
                    // }
                    /* ------------------------------------------------------------ */
                    /* ------------------------------------------------------------ */
                    /* ------------------------------------------------------------ */

                    let identifier_key = format!("{}-make-cid", _id);

                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "make_cid");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                        
                        /* building new id contains the public and private key and the snowflake id */
                        let get_new_id = Id::new_or_update(
                            new_object_id_request.clone(), 
                            _id, 
                            /* 
                                we're using new_object_id_request username since 
                                the username inside the JWT might be empty
                            */
                            new_object_id_request.username.clone().to_lowercase(),
                            user_ip,
                            redis_client.to_owned(),
                            redis_actix_actor.clone(),
                            connection
                        ).await;

                        /* 
                            if we found a user simply we'll update all its fields with 
                            new one inside the body of NewIdRequest object except cid and 
                            the snowflake id, then return the updated data as the err response 
                            of the new_or_update method.
                        */
                        let Ok(mut new_id) = get_new_id else{
                            let resp = get_new_id.unwrap_err();
                            return resp;
                        };


                        /* 
                            saveing the new Id into db, also if we're here means 
                            that we're creating a new Id for the user since on the 
                            second request it'll return the founded user info
                        */
                        let save_user_data = new_id.save(redis_client.to_owned(), redis_actix_actor, connection).await;
                        let Ok(user_data) = save_user_data else{
                            let resp = save_user_data.unwrap_err();
                            return resp;
                        };
                        
                        resp!{
                            UserIdResponse, // the data type
                            user_data, // response data
                            ID_BUILT, // response message
                            StatusCode::CREATED, // status code
                            None::<Cookie<'_>>, // cookie
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

#[post("/deposit/to/{contract_address}")]
#[passport(user)]
async fn deposit(
    req: HttpRequest,
    contract_address: web::Path<String>,
    deposit: web::Json<NewUserDepositRequest>,
    storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-deposit", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "deposit");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {

                        let deposit_object = deposit.to_owned();

                        let find_recipient_screen_cid = User::find_by_username_or_mail_or_scid(&deposit_object.recipient, connection).await;
                        let Ok(recipient_info) = find_recipient_screen_cid else{
                            
                            resp!{
                                String, // the data type
                                deposit_object.recipient, // response data
                                &RECIPIENT_NOT_FOUND, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

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
                            &deposit_object.from_cid, 
                            &deposit_object.tx_signature, 
                            &deposit_object.hash_data, 
                            Some(deposit_object.amount), 
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };



                        let new_balance = user.balance.unwrap() - deposit_object.amount;
                        let mut mint_tx_hash = String::from("");
                        let mut token_id = String::from("");

                        /* deposit_object.recipient_screen_cid must be the keccak256 of the recipient public key */
                        if recipient_info.screen_cid.is_none(){
                            resp!{
                                String, // the date type
                                deposit_object.recipient, // the data itself
                                RECIPIENT_SCREEN_CID_NOT_FOUND, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        if recipient_info.cid.clone().unwrap() == deposit_object.from_cid{

                            resp!{
                                String, // the date type
                                deposit_object.from_cid, // the data itself
                                SENDER_CANT_BE_RECEIVER, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        let polygon_recipient_address = recipient_info.clone().screen_cid.unwrap();
                        let contract_owner = env::var("GIFT_CARD_POLYGON_NFT_OWNER_ADDRESS").unwrap();
                        let contract_address_env = env::var("GIFT_CARD_POLYGON_NFT_CONTRACT_ADDRESS").unwrap();

                        if contract_address.to_owned() != contract_address_env{
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_CONTRACT_ADDRESS, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        let (tx_hash, tid, res_mint_status) = start_minting_card_process(
                            user.screen_cid.unwrap(),
                            deposit_object.clone(),  
                            contract_address.clone(),
                            contract_owner.clone(),
                            polygon_recipient_address.clone(),
                            deposit_object.nft_img_url.clone(),
                            deposit_object.nft_name,
                            deposit_object.nft_desc,
                            redis_client.clone()
                        ).await;
                        
                        if res_mint_status == 1{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                CANT_MINT_CARD, // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        mint_tx_hash = tx_hash; // moving into another type
                        token_id = tid;
                        
                        if !mint_tx_hash.is_empty(){
                            
                            match UserDeposit::insert(deposit.to_owned(), mint_tx_hash, token_id, polygon_recipient_address, deposit_object.nft_img_url, connection).await{
                                Ok(user_deposit_data) => {

                                    let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor, connection).await;
                                    let Ok(updated_user_data) = update_user_balance else{

                                        let err_resp = update_user_balance.unwrap_err();
                                        return err_resp;
                                        
                                    };

                                    resp!{
                                        UserDepositData, // the data type
                                        user_deposit_data, // response data
                                        DEPOSITED_SUCCESSFULLY, // response message
                                        StatusCode::CREATED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }

                                },
                                Err(resp) => {
                                    /* 
                                        🥝 response can be one of the following:
                                        
                                        - DIESEL INSERT ERROR RESPONSE
                                    */
                                    resp
                                }
                            }                                    

                            
                        } else{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                CANT_MINT_CARD, // response message
                                StatusCode::FAILED_DEPENDENCY, // status code
                                None::<Cookie<'_>>, // cookie
                            }
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

#[get("/deposit/get/all/")]
#[passport(user)]
async fn get_all_user_deposits(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserDeposit::get_all_for(user.cid.unwrap().to_string(), limit, connection).await{
                        Ok(user_deposits) => {

                            resp!{
                                Vec<UserDepositData>, // the data type
                                user_deposits, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[post("/withdraw/from/{contract_address}")]
#[passport(user)]
async fn withdraw(
    req: HttpRequest,
    contract_address: web::Path<String>,
    withdraw: web::Json<NewUserWithdrawRequest>,
    storage: web::Data<Option<Arc<Storage>>>,
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-withdraw", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "withdraw");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{ /* chill 30 seconds */
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else { /* not rate limited, we're ok to go */

                        let withdraw_object = withdraw.to_owned();
                    
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
                            &withdraw_object.recipient_cid, 
                            &withdraw_object.tx_signature, 
                            &withdraw_object.hash_data, 
                            None, /* no need to charge the user for this call */
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        let get_deposit_info = UserDeposit::find_by_id(withdraw_object.deposit_id, connection).await;
                        let Ok(deposit_info) = get_deposit_info else{

                            let error = get_deposit_info.unwrap_err();
                            return error;
                        };


                        /* generate keccak256 from recipient_cid to check aginst the one in db */
                        let polygon_recipient_address = walletreq::evm::get_keccak256_from(withdraw_object.recipient_cid.to_owned().clone());
                        if deposit_info.recipient_screen_cid != polygon_recipient_address ||
                        withdraw_object.recipient_cid != user.cid.unwrap(){
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                NO_DEPOSIT_FOR_THIS_RECIPIENT, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        let token_id = deposit_info.nft_id;
                        let mut transfer_tx_hash = String::from("");
                        
                        let res_transfer = start_transferring_card_process(
                            contract_address.to_owned(), 
                            token_id,
                            polygon_recipient_address,
                            redis_client.clone()
                        ).await;

                        if res_transfer.1 == 1{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                CANT_TRANSFER_CARD, // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        transfer_tx_hash = res_transfer.0; // moving into another type
                        
                        if !transfer_tx_hash.is_empty(){

                            match UserWithdrawal::insert(withdraw.to_owned(), transfer_tx_hash, connection).await{
                                Ok(user_withdrawal_data) => {
                                    
                                    let new_balance = if user.balance.is_none(){0 + deposit_info.amount} else{user.balance.unwrap() + deposit_info.amount};
                                    let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor, connection).await;
                                    let Ok(updated_user_data) = update_user_balance else{

                                        let err_resp = update_user_balance.unwrap_err();
                                        return err_resp;
                                        
                                    };
                                    
                                    resp!{
                                        UserWithdrawalData, // the data type
                                        user_withdrawal_data, // response data
                                        WITHDRAWN_SUCCESSFULLY, // response message
                                        StatusCode::CREATED, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }

                                },
                                Err(resp) => {
                                    /* 
                                        🥝 response can be one of the following:
                                        
                                        - DIESEL INSERT ERROR RESPONSE
                                        - DEPOSIT OBJECT NOT FOUND
                                        - ALREADY_WITHDRAWN
                                    */
                                    resp
                                }
                            }
                                

                        } else{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                CANT_TRANSFER_CARD, // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }
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

#[get("/withdraw/get/all/")]
#[passport(user)]
async fn get_all_user_withdrawals(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
    
                    match UserWithdrawal::get_all_for(user.cid.unwrap().to_string(), limit, connection).await{
                        Ok(user_withdrawals) => {

                            resp!{
                                Vec<UserWithdrawalData>, // the data type
                                user_withdrawals, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[get("/checkout/get/all/unpaid/")]
#[passport(user)]
async fn get_all_user_unpaid_checkouts(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
    
                    match UserCheckout::get_all_unpaid_for(&user.cid.unwrap(), limit, connection).await{
                        Ok(user_checkouts) => {

                            resp!{
                                Vec<UserCheckoutData>, // the data type
                                user_checkouts, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[get("/checkout/get/all/paid/")]
#[passport(user)]
async fn get_all_user_paid_checkouts(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
    
                    match UserCheckout::get_all_paid_for(&user.cid.unwrap(), limit, connection).await{
                        Ok(user_checkouts) => {

                            resp!{
                                Vec<UserCheckoutData>, // the data type
                                user_checkouts, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[get("/deposit/get/all/unclaimed/")]
#[passport(user)]
async fn get_recipient_unclaimed_deposits(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    /* generate keccak256 from recipient_cid to mint nft to */
                    let polygon_recipient_address = walletreq::evm::get_keccak256_from(user.cid.unwrap().to_owned().clone());

                    match UserDeposit::get_unclaimeds_for(polygon_recipient_address, limit, connection).await{
                        Ok(user_unclaimeds) => {

                            resp!{
                                Vec<UserDepositData>, // the data type
                                user_unclaimeds, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                            */
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

#[post("/profile/update/bio")]
#[passport(user)]
async fn edit_bio(
    req: HttpRequest,
    update_bio_request: web::Json<UpdateBioRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let new_bio = update_bio_request.to_owned().bio;

                    match User::update_bio(_id, &new_bio, redis_client.to_owned(), redis_actix_actor, connection).await{
                        Ok(updated_user) => {
                            resp!{
                                UserData, // the data type
                                updated_user, // response data
                                UPDATED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        },
                        Err(resp) => {
                            
                            /* USER NOT FOUND response */
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

#[post("/profile/update/extra")]
#[passport(user)]
async fn edit_extra(
    req: HttpRequest,
    update_extra_request: web::Json<UpdateExtraRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let new_extra = update_extra_request.to_owned().extra;

                    match User::update_extra(_id, new_extra, redis_client.to_owned(), redis_actix_actor, connection).await{
                        Ok(updated_user) => {
                            resp!{
                                UserData, // the data type
                                updated_user, // response data
                                UPDATED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        },
                        Err(resp) => {
                            
                            /* USER NOT FOUND response */
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

#[post("/profile/update/wallet-back")]
#[passport(user)]
async fn upload_wallet_back(
    req: HttpRequest,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart,
) -> PanelHttpResponse{

    /* extracting shared storage data */
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

        match storage.clone().unwrap().as_ref().get_pgdb().await{

            Some(pg_pool) => {
    
                let connection = &mut pg_pool.get().unwrap();
    
    
                /* ------ ONLY USER CAN DO THIS LOGIC ------ */
                match req.get_user(granted_role, connection){
                    Ok(token_data) => {
                        
                        let _id = token_data._id;
                        let role = token_data.user_role;
    
                        
                        match User::update_wallet_back(_id, img, redis_client.to_owned(), redis_actix_actor, connection).await{
                            Ok(updated_user) => {
                                
                                resp!{
                                    UserData, // the data type
                                    updated_user, // response data
                                    UPDATED, // response message
                                    StatusCode::OK, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            },
                            Err(resp) => {
                                
                                /* USER NOT FOUND response */
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

#[post("/profile/update/avatar")]
#[passport(user)]
async fn upload_avatar(
    req: HttpRequest,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart, /* form-data implementation to receive stream of byte fields */
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    
                    match User::update_avatar(_id, img, redis_client.to_owned(), redis_actix_actor, connection).await{
                        Ok(updated_user) => {
                            
                            resp!{
                                UserData, // the data type
                                updated_user, // response data
                                UPDATED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        },
                        Err(resp) => {
                            
                            /* USER NOT FOUND response */
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

#[post("/profile/update/banner")]
#[passport(user)]
async fn upload_banner(
    req: HttpRequest,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart, /* form-data implementation to receive stream of byte fields */
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match User::update_banner(_id, img, redis_client.to_owned(), redis_actix_actor, connection).await{
                        Ok(updated_user) => {
                            
                            resp!{
                                UserData, // the data type
                                updated_user, // response data
                                UPDATED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        },
                        Err(resp) => {
                            
                            /* USER NOT FOUND response */
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

#[post("/profile/update/password")]
#[passport(user)]
async fn update_password(
    req: HttpRequest,
    new_password_request: web::Json<NewPasswordRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-update-password", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "update_password");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let new_password_request = new_password_request.to_owned();
                        
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
                            &new_password_request.owner_cid, 
                            &new_password_request.tx_signature, 
                            &new_password_request.hash_data, 
                            None, /* no need to charge the user for this call */
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        todo!()

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

#[post("/profile/forgot-password")]
#[passport(user)]
async fn forgot_password(
    req: HttpRequest,
    forgot_password_request: web::Json<ForgotPasswordRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-forgot-password", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "forgot_password");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let forgot_password_request = forgot_password_request.to_owned();
                        
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
                            &forgot_password_request.owner_cid, 
                            &forgot_password_request.tx_signature, 
                            &forgot_password_request.hash_data, 
                            None, /* no need to charge the user for this call */
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        todo!()

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

/* 
    this api must gets called by player with his conse rendezvous hyper server JWT 
    passed in to the request header 
*/
#[post("/rendezvous/player/{player_id}/upload/avatar")]
async fn upload_rendezvous_player_avatar(
    req: HttpRequest, 
    player_id: web::Path<String>, // mongodb objectid
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart, /* form-data implementation to receive stream of byte fields */
    ) -> PanelHttpResponse{


        if let Some(header_value) = req.headers().get("Authorization"){

            let token = header_value.to_str().unwrap();
            
            /*
                @params: 
                    - @token          → JWT
    
                note that this token must be taken from the conse rendezvous hyper server
            */
            match rendezvous_passport!{ token }{
                true => {
    
                    // -------------------------------------------------------------------------------------
                    // ------------------------------- ACCESS GRANTED REGION -------------------------------
                    // -------------------------------------------------------------------------------------
                    /*  
                        this route requires the player access token from the conse 
                        rendezvous hyper server to update avatar image, we'll send a request
                        to the conse rendezvous hyper server to verify the passed in JWT of the
                        player and if it was verified we'll allow the user to update the image
                    */
    
                    let storage = storage.as_ref().to_owned();
                    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                    let player_id_img_key = format!("{player_id:}-img");

                    let get_redis_conn = redis_client.get_async_connection().await;
                    let Ok(mut redis_conn) = get_redis_conn else{

                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "update_event_img");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    let img = std::sync::Arc::new(tokio::sync::Mutex::new(img));
                    let get_player_img_path = multipartreq::store_file(
                        AVATAR_UPLOAD_PATH, &format!("{}", player_id), 
                        "player", 
                        img).await;
                    let Ok(player_img_filepath) = get_player_img_path else{
            
                        let err_res = get_player_img_path.unwrap_err();
                        return err_res;
                    };

                    
                    /* 
                        writing the avatar image filename to redis ram, by doing this we can 
                        retrieve the value from redis in conse hyper rendezvous server when we call 
                        the check token api
                    */
                    let _: () = redis_conn.set(player_id_img_key.as_str(), player_img_filepath.as_str()).await.unwrap();
                
                    resp!{
                        &[u8], // the date type
                        &[], // the data itself
                        RENDEZVOUS_PLAYER_AVATAR_IMG_UPDATED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
                    }
                    
    
                    // -------------------------------------------------------------------------------------
                    // -------------------------------------------------------------------------------------
                    // -------------------------------------------------------------------------------------
    
                },
                false => {
                    
                    resp!{
                        &[u8], // the date type
                        &[], // the data itself
                        INVALID_TOKEN, // response message
                        StatusCode::FORBIDDEN, // status code
                        None::<Cookie<'_>>, // cookie
                    }
                }
            }
    
        } else{
            
            resp!{
                &[u8], // the date type
                &[], // the data itself
                NOT_AUTH_HEADER, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        }

}

#[post("/gallery/create")]
#[passport(user)]
async fn create_private_gallery(
    req: HttpRequest,
    new_private_gallery_request: web::Json<NewUserPrivateGalleryRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-create-private-gallery", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "create_private_gallery");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let create_private_gallery_request_object = new_private_gallery_request.to_owned();
                        
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
                            &create_private_gallery_request_object.owner_cid, 
                            &create_private_gallery_request_object.tx_signature, 
                            &create_private_gallery_request_object.hash_data, 
                            None, /* no need to charge the user for this call */
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        match UserPrivateGallery::insert(create_private_gallery_request_object, connection).await{
                            Ok(gallery_data) => {

                                resp!{
                                    UserPrivateGalleryData, //// the data type
                                    gallery_data, //// response data
                                    CREATED, //// response message
                                    StatusCode::CREATED, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            },
                            Err(resp) => {
                                resp
                            }
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

#[post("/gallery/{gal_id}/update")]
#[passport(user)]
async fn update_private_gallery(
    req: HttpRequest,
    gal_id: web::Path<i32>,
    update_private_gallery_request: web::Json<UpdateUserPrivateGalleryRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-update-private-gallery", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "update_private_gallery");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        let update_private_gallery_request_object = update_private_gallery_request.to_owned();
                        
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
                            &update_private_gallery_request_object.owner_cid, 
                            &update_private_gallery_request_object.tx_signature, 
                            &update_private_gallery_request_object.hash_data, 
                            None, /* no need to charge the user for this call */
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        match UserPrivateGallery::update(&walletreq::evm::get_keccak256_from(
                            update_private_gallery_request_object.clone().owner_cid), 
                            update_private_gallery_request_object, 
                            gal_id.to_owned(), 
                            connection).await{
                           
                            Ok(updated_gal_data) => {

                                resp!{
                                    UserPrivateGalleryData, //// the data type
                                    updated_gal_data, //// response data
                                    UPDATED, //// response message
                                    StatusCode::OK, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            },
                            Err(resp) => {
                                resp
                            }
                        
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

#[post("/gallery/remove/invited-friend")]
#[passport(user)]
async fn remove_invited_friend_from_gallery(
    req: HttpRequest,
    remove_invited_friend_request: web::Json<RemoveInvitedFriendFromPrivateGalleryRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let remove_invited_friend_request = remove_invited_friend_request.to_owned();
                    
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
                        &remove_invited_friend_request.caller_cid, 
                        &remove_invited_friend_request.tx_signature, 
                        &remove_invited_friend_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserPrivateGallery::remove_invited_friend_from(remove_invited_friend_request, redis_client.clone(), redis_actix_actor, connection).await{
                        Ok(gallery_data) => {

                            resp!{
                                UserPrivateGalleryData, //// the data type
                                gallery_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/gallery/exit")]
#[passport(user)]
async fn exit_from_private_gallery(
    req: HttpRequest,
    exit_from_private_gallery: web::Json<ExitFromPrivateGalleryRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let exit_from_private_gallery = exit_from_private_gallery.to_owned();
                    
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
                        &exit_from_private_gallery.caller_cid, 
                        &exit_from_private_gallery.tx_signature, 
                        &exit_from_private_gallery.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserPrivateGallery::exit_from_private_gallery(exit_from_private_gallery, connection).await{
                        Ok(gallery_data) => {

                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/gallery/send/invitation-request")]
#[passport(user)]
async fn send_private_gallery_invitation_request_to(
    req: HttpRequest,
    send_invitation_request: web::Json<SendInvitationRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    let send_invitation_request = send_invitation_request.to_owned();

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
                        &send_invitation_request.gallery_owner_cid, 
                        &send_invitation_request.tx_signature, 
                        &send_invitation_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserPrivateGallery::send_invitation_request_to(send_invitation_request, connection).await{
                        Ok(invitation_data) => {

                            resp!{
                                InvitationRequestDataResponse, //// the data type
                                invitation_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/gallery/get/all/")]
#[passport(user)]
async fn get_all_private_galleries_for(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserPrivateGallery::get_all_for(&user.screen_cid.unwrap(), limit, connection).await{
                        
                        Ok(galleries) => {

                            resp!{
                                Vec<UserPrivateGalleryData>, //// the data type
                                galleries, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/gallery/get/all/i-invited-to/")]
#[passport(user)]
async fn get_all_galleries_invited_to(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserPrivateGallery::get_all_galleries_invited_to(
                        &user.screen_cid.unwrap(), 
                        limit, connection).await{
                        
                        Ok(galleries) => {

                            resp!{
                                Vec<Option<UserPrivateGalleryInfoDataInvited>>, //// the data type
                                galleries, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/gallery/{gal_id}/get/invited-friends/")]
#[passport(user)]
async fn get_invited_friends_wallet_data_of_gallery(
    req: HttpRequest,
    gal_id: web::Path<i32>,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserPrivateGallery::get_invited_friends_wallet_data_of_gallery(
                        &user.screen_cid.unwrap(),
                        gal_id.to_owned(),
                        limit, connection).await{
                        
                        Ok(friends_wallet_data) => {

                            resp!{
                                Vec<Option<UserWalletInfoResponse>>, //// the data type
                                friends_wallet_data, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/gallery/get/unaccepted/invitation-requests/")]
#[passport(user)]
async fn get_user_unaccpeted_invitation_requests(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_user_unaccpeted_invitation_requests(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        
                        Ok(unaccepted_requests) => {

                            resp!{
                                Vec<Option<InvitationRequestData>>, //// the data type
                                unaccepted_requests, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/fan/get/unaccepted/friend-requests/")]
#[passport(user)]
async fn get_user_unaccpeted_friend_requests(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_user_unaccpeted_friend_requests(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        
                        Ok(unaccepted_requests) => {

                            resp!{
                                Vec<Option<FriendData>>, //// the data type
                                unaccepted_requests, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/gallery/accept/invitation-request")]
#[passport(user)]
async fn accept_invitation_request(
    req: HttpRequest,
    accept_invitation_request: web::Json<AcceptInvitationRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let accept_invitation_request = accept_invitation_request.to_owned();
                    
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
                        &accept_invitation_request.owner_cid, 
                        &accept_invitation_request.tx_signature, 
                        &accept_invitation_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::accept_invitation_request(accept_invitation_request, redis_client.clone(), redis_actix_actor, connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/gallery/enter")]
#[passport(user)]
async fn enter_private_gallery(
    req: HttpRequest,
    enter_private_gallery_request: web::Json<EnterPrivateGalleryRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let enter_private_gallery_request = enter_private_gallery_request.to_owned();
                    
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
                        &enter_private_gallery_request.caller_cid, 
                        &enter_private_gallery_request.tx_signature, 
                        &enter_private_gallery_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::enter_private_gallery_request(enter_private_gallery_request, redis_client.clone(), redis_actix_actor, connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/fan/get/relations/for/{who}/")]
#[passport(user)]
async fn get_all_user_relations(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_user_relations(
                        &who_screen_cid.to_owned(), 
                        limit, connection).await{
                        
                        Ok(user_relations) => {

                            resp!{
                                UserRelations, //// the data type
                                user_relations, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/fan/accept/friend-request")]
#[passport(user)]
async fn accept_friend_request(
    req: HttpRequest,
    accept_friend_request: web::Json<AcceptFriendRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let accept_friend_request = accept_friend_request.to_owned();
                    
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
                        &accept_friend_request.owner_cid, 
                        &accept_friend_request.tx_signature, 
                        &accept_friend_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::accept_friend_request(accept_friend_request, connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/fan/send/friend-request")]
#[passport(user)]
async fn send_friend_request_to(
    req: HttpRequest,
    send_friend_request_to: web::Json<SendFriendRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let send_friend_request_to = send_friend_request_to.to_owned();
                    
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
                        &send_friend_request_to.owner_cid, 
                        &send_friend_request_to.tx_signature, 
                        &send_friend_request_to.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::send_friend_request_to(send_friend_request_to, connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/fan/remove/follower")]
#[passport(user)]
async fn remove_user_from_follower(
    req: HttpRequest,
    remove_follower_request: web::Json<RemoveFollower>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let remove_follower_request = remove_follower_request.to_owned();
                    
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
                        &remove_follower_request.owner_cid, 
                        &remove_follower_request.tx_signature, 
                        &remove_follower_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::remove_follower(remove_follower_request, connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/fan/remove/friend")]
#[passport(user)]
async fn remove_user_from_freind(
    req: HttpRequest,
    remove_friend_request: web::Json<RemoveFriend>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let remove_friend_request = remove_friend_request.to_owned();
                    
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
                        &remove_friend_request.owner_cid, 
                        &remove_friend_request.tx_signature, 
                        &remove_friend_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::remove_freind(remove_friend_request, connection).await{
                        Ok(user_fan_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fan_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/fan/get/all/friends/")]
#[passport(user)]
async fn get_all_my_friends(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_all_my_friends(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        Ok(user_fans_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fans_data, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/fan/get/all/followings/")]
#[passport(user)]
async fn get_all_my_followings(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_all_my_followings(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        Ok(followings) => {

                            resp!{
                                Vec<UserFanData>, //// the data type
                                followings, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/fan/get/all/followers/")]
#[passport(user)]
async fn get_all_my_followers(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserFan::get_all_my_followers(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        Ok(user_fans_data) => {

                            resp!{
                                UserFanData, //// the data type
                                user_fans_data, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/fan/get/suggestions/for/")]
#[passport(user)]
async fn get_friend_suggestions_for_owner(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match User::suggest_user_to_owner(
                        limit, 
                        &user.screen_cid.unwrap(),
                        connection).await{
                        Ok(suggestions) => {

                            resp!{
                                Vec<UserWalletInfoResponse>, //// the data type
                                suggestions, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/collection/get/all/for/{who}/")]
#[passport(user)]
async fn get_all_public_collections_for(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserCollection::get_all_public_collections_for(
                        &who_screen_cid.to_owned(), 
                        limit, &user.screen_cid.unwrap(), connection).await{
                        
                        Ok(collections) => {

                            resp!{
                                Vec<Option<UserCollectionData>>, //// the data type
                                collections, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/gallery/get/all/for/{who}/")]
#[passport(user)]
async fn get_all_private_galleries_general_info_for(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    // get all general infos of all private galleries for who_screen_cid
                    match UserPrivateGallery::get_all_general_info_for(
                        &who_screen_cid.to_owned(), 
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        
                        Ok(private_galleries) => {

                            resp!{
                                Vec<UserPrivateGalleryInfoData>, //// the data type
                                private_galleries, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/collection/{col_id}/get/all/minted-nfts/")]
#[passport(user)]
async fn get_all_public_collection_nfts(
    req: HttpRequest,
    col_id: web::Path<i32>,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserCollection::get_all_minted_nfts_of_collection(
                        col_id.to_owned(), 
                        limit, &user.screen_cid.unwrap(), connection).await{
                        
                        Ok(nfts) => {

                            resp!{
                                Vec<Option<UserNftData>>, //// the data type
                                nfts, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/collection/get/all/private/in-gallery/{gal_id}/")]
#[passport(user)]
async fn get_all_private_collections_for(
    req: HttpRequest,
    gal_id: web::Path<i32>,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    let gal_id = gal_id.to_owned();
                    match UserCollection::get_all_private_collections_for(
                        &user.screen_cid.unwrap(),
                        gal_id,
                        limit, connection).await{
                        
                        Ok(collections) => {

                            resp!{
                                Vec<UserCollectionData>, //// the data type
                                collections, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/collection/create")]
#[passport(user)]
async fn create_collection(
    req: HttpRequest,
    new_user_collection_request: web::Json<NewUserCollectionRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-create-collection", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "create_collection");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let new_user_collection_request = new_user_collection_request.to_owned();
                        
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
                            &new_user_collection_request.owner_cid, 
                            &new_user_collection_request.tx_signature, 
                            &new_user_collection_request.hash_data, 
                            Some(new_user_collection_request.amount),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        match UserCollection::insert(
                            new_user_collection_request, 
                            redis_actix_actor.clone(),
                            redis_client.clone(), 
                            connection).await{
                            Ok(user_collection_data) => {

                                resp!{
                                    UserCollectionData, //// the data type
                                    user_collection_data, //// response data
                                    CREATED, //// response message
                                    StatusCode::CREATED, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            },
                            Err(resp) => {
                                resp
                            }
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

#[post("/collection/update")]
#[passport(user)]
async fn update_collection(
    req: HttpRequest,
    update_user_collection_request: web::Json<UpdateUserCollectionRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-update-collection", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "updated_collection");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let update_user_collection_request = update_user_collection_request.to_owned();
                        
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
                            &update_user_collection_request.owner_cid, 
                            &update_user_collection_request.tx_signature, 
                            &update_user_collection_request.hash_data, 
                            Some(update_user_collection_request.amount),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        match UserCollection::update(
                            update_user_collection_request,
                            redis_actix_actor,
                            redis_client.clone(), 
                            connection).await{
                            Ok(user_collection_data) => {

                                resp!{
                                    UserCollectionData, //// the data type
                                    user_collection_data, //// response data
                                    UPDATED, //// response message
                                    StatusCode::OK, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            },
                            Err(resp) => {
                                resp
                            }
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

#[post("/nft/create")]
#[passport(user)]
async fn create_nft(
    req: HttpRequest,
    new_user_nft_request: web::Json<NewUserNftRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-create-nft", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "create_nft");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let new_user_nft_request = new_user_nft_request.to_owned();
                        
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
                            &new_user_nft_request.caller_cid, 
                            &new_user_nft_request.tx_signature, 
                            &new_user_nft_request.hash_data, 
                            Some(new_user_nft_request.amount),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        match UserNft::insert(
                            new_user_nft_request, 
                            redis_client.clone(), 
                            redis_actix_actor,
                            connection).await{
                            Ok(user_nft_data) => {

                                resp!{
                                    UserNftData, //// the data type
                                    user_nft_data, //// response data
                                    CREATED, //// response message
                                    StatusCode::CREATED, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            },
                            Err(resp) => {
                                resp
                            }
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

#[post("/nft/create/metadata-uri")]
#[passport(user)]
async fn create_nft_metadata_uri(
    req: HttpRequest,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut payload: Multipart,
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-create-nft-metadata-uri", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "create_nft_metadata_uri");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                        

                        let arced_payload = std::sync::Arc::new(tokio::sync::Mutex::new(payload));
                        let get_json_data = multipartreq::extract(
                            arced_payload.clone()
                        ).await;

                        let Ok((json_data, files)) = get_json_data else{
                            let err_resp = get_json_data.unwrap_err();
                            return err_resp;
                        };

                        /* json_data can by any content, here we're trying to map it into the CreateNftMetadataUriRequest struct */
                        let create_nft_metadata_uri_request = serde_json::from_value::<CreateNftMetadataUriRequest>(json_data).unwrap();


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
                            &create_nft_metadata_uri_request.caller_cid, 
                            &create_nft_metadata_uri_request.tx_signature, 
                            &create_nft_metadata_uri_request.hash_data, 
                            Some(create_nft_metadata_uri_request.amount.parse::<i64>().unwrap()),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };


                        match UserNft::create_nft_metadata_uri(
                            create_nft_metadata_uri_request,
                            files,
                            redis_client.clone(), 
                            connection).await{
                            Ok(user_nft_data) => {
    
                                resp!{
                                    UserNftData, //// the data type
                                    user_nft_data, //// response data
                                    UPDATED, //// response message
                                    StatusCode::OK, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }
    
                            },
                            Err(resp) => {
                                resp
                            }
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

#[post("/collection/{col_id}/upload/background")]
#[passport(user)]
async fn upload_collection_banner(
    req: HttpRequest,
    col_id: web::Path<i32>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart,
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserCollection::upload_collection_img(
                        col_id.to_owned(), 
                        &user.screen_cid.unwrap(),
                        img,
                        connection).await{
                        Ok(user_collection_data) => {

                            resp!{
                                UserCollectionData, //// the data type
                                user_collection_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/gallery/{gal_id}/upload/background")]
#[passport(user)]
async fn upload_private_gallery_back(
    req: HttpRequest,
    gal_id: web::Path<i32>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart,
) -> PanelHttpResponse{

    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserPrivateGallery::upload_background(
                        gal_id.to_owned(), 
                        &user.screen_cid.unwrap(), // this must be the gallery owner
                        img,
                        connection).await{
                        Ok(user_private_gallery_data) => {

                            resp!{
                                UserPrivateGalleryData, //// the data type
                                user_private_gallery_data, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/nft/update")]
#[passport(user)]
async fn update_nft(
    req: HttpRequest,
    update_user_nft_request: web::Json<UpdateUserNftRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-update-nft", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "updated_nft");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let update_user_nft_request = update_user_nft_request.to_owned();
                        
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
                            &update_user_nft_request.caller_cid, 
                            &update_user_nft_request.tx_signature, 
                            &update_user_nft_request.hash_data, 
                            Some(update_user_nft_request.amount),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        match UserNft::update(
                            update_user_nft_request,
                            redis_client.clone(), 
                            redis_actix_actor,
                            connection).await{
                            Ok(user_nft_data) => {

                                resp!{
                                    UserNftData, //// the data type
                                    user_nft_data, //// response data
                                    UPDATED, //// response message
                                    StatusCode::OK, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            },
                            Err(resp) => {
                                resp
                            }
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

#[post("/nft/add/reaction")]
#[passport(user)]
async fn add_reaction_to_nft(
    req: HttpRequest,
    user_add_nft_reaction: web::Json<AddReactionRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-add-reaction-to-nft", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "add_reaction_to_nft");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let user_add_nft_reaction = user_add_nft_reaction.to_owned();
                        
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
                            &user_add_nft_reaction.caller_cid, 
                            &user_add_nft_reaction.tx_signature, 
                            &user_add_nft_reaction.hash_data, 
                            None,
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        match UserNft::add_reaction_to_nft(
                            user_add_nft_reaction,
                            connection).await{
                            Ok(user_nft_data) => {

                                resp!{
                                    UserNftData, //// the data type
                                    user_nft_data, //// response data
                                    UPDATED, //// response message
                                    StatusCode::OK, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            },
                            Err(resp) => {
                                resp
                            }
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

#[post("/nft/buy")]
#[passport(user)]
async fn buy_nft(
    req: HttpRequest,
    user_buy_nft_request: web::Json<UpdateUserNftRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-buy-nft", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "buy_nft");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let user_buy_nft_request = user_buy_nft_request.to_owned();
                        
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
                            &user_buy_nft_request.caller_cid, 
                            &user_buy_nft_request.tx_signature, 
                            &user_buy_nft_request.hash_data, 
                            Some(user_buy_nft_request.amount),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        match UserNft::buy_nft(
                            user_buy_nft_request,
                            redis_client.clone(),
                            redis_actix_actor,
                            connection).await{
                            Ok(user_nft_data) => {

                                resp!{
                                    UserNftData, //// the data type
                                    user_nft_data, //// response data
                                    UPDATED, //// response message
                                    StatusCode::OK, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            },
                            Err(resp) => {
                                resp
                            }
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

#[post("/nft/mint")]
#[passport(user)]
async fn mint_nft(
    req: HttpRequest,
    user_mint_nft_request: web::Json<UpdateUserNftRequest>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-mint-nft", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "mint_nft");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                    
                        
                        let user_mint_nft_request = user_mint_nft_request.to_owned();
                        
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
                            &user_mint_nft_request.caller_cid, 
                            &user_mint_nft_request.tx_signature, 
                            &user_mint_nft_request.hash_data, 
                            Some(user_mint_nft_request.amount),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        match UserNft::mint_nft(
                            user_mint_nft_request,
                            redis_client.clone(),
                            redis_actix_actor,
                            connection).await{
                            Ok(user_nft_data) => {

                                resp!{
                                    UserNftData, //// the data type
                                    user_nft_data, //// response data
                                    UPDATED, //// response message
                                    StatusCode::OK, //// status code
                                    None::<Cookie<'_>>, //// cookie
                                }

                            },
                            Err(resp) => {
                                resp
                            }
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

#[get("/reaction/get/all/")]
#[passport(user)]
async fn get_all_user_reactions(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserNft::get_all_user_reactions(
                        &user.screen_cid.unwrap(),
                        limit, connection).await{
                        Ok(user_reaction_data) => {

                            resp!{
                                Vec<Option<UserReactionData>>, //// the data type
                                user_reaction_data, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/nft/get/all/onchain/")]
#[passport(user)]
async fn get_all_nfts_owned_by(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{
    
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    match UserNft::get_all_nfts_owned_by(
                        &user.screen_cid.unwrap(),
                        limit).await{
                        Ok(user_nfts) => {

                            resp!{
                                OnchainNfts, //// the data type
                                user_nfts, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[get("/nft/{nft_onchain_id}/reaction/get/all")]
#[passport(user)]
async fn get_all_nft_reactions(
    req: HttpRequest,
    nft_onchain_id: web::Path<String>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    let nft_onchain_id = nft_onchain_id.to_owned();
                    match UserNft::get_all_nft_reactions(
                        &nft_onchain_id,
                        connection).await{
                        Ok(nft_reaction_data) => {

                            resp!{
                                NftReactionData, //// the data type
                                nft_reaction_data, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => {
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

#[post("/clp/register")]
#[passport(user)]
async fn register_clp_event(
    req: HttpRequest,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-register-clp-event", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "register_clp_event");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    /* 
                        checking that the incoming request is already rate limited or not,
                        since there is no global storage setup we have to pass the storage 
                        data like redis_conn to the macro call 
                    */
                    if is_rate_limited!{
                        redis_conn,
                        identifier_key.clone(), /* identifier */
                        String, /* the type of identifier */
                        "fin_rate_limiter" /* redis key */
                    }{

                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            RATE_LIMITED, //// response message
                            StatusCode::TOO_MANY_REQUESTS, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }

                    } else {
                        

                        // kyc with deposited amount
                        // user balance must be higher than the entry amount of the event
                        // update users_clps schema
                        // ...
                        
                        todo!()

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


#[get("/clp/get/new")]
#[passport(user)]
async fn get_new_clp_event_info(
    req: HttpRequest,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{
    
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    // clp_events schema
                    // return the latest clp in the stack
                    // ...
                    todo!()
                    

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

#[get("/clp/get/all/")]
#[passport(user)]
async fn get_all_user_clp_events_info(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{
    
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;

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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();

            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    let get_all_my_events = UserClp::get_all_user_events(limit, _id, connection).await;
                    let Ok(all_my_events) = get_all_my_events else{
                        let err_resp = get_all_my_events.unwrap_err();
                        return err_resp;
                    };
                    
                    resp!{
                        Vec<ClpEventData>, //// the data type
                        all_my_events, //// response data
                        FETCHED, //// response message
                        StatusCode::OK, //// status code
                        None::<Cookie<'_>>, //// cookie
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

#[get("/get-token-value/{tokens}")]
#[passport(user)]
async fn get_token_value(
        req: HttpRequest,  
        tokens: web::Path<i64>,
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let async_redis_client = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();

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


    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    /* 
                        Note: The usd field in response need to gets divided by 10000000 to extract 
                        the exact amount of token based in USD.
                    */
                    let value = gastracker::calculate_token_value(tokens.to_owned(), redis_client.clone()).await;
                    resp!{
                        GetTokenValueResponse, // the data type
                        GetTokenValueResponse{
                            usd: value.0,
                            irr: value.1,
                        }, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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

#[get("/get-gas-fee")]
#[passport(user)]
async fn get_gas_fee(
        req: HttpRequest,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let async_redis_client = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();

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


    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.screen_cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }

                    let get_gas_fee = gastracker::calculate_gas_in_token(redis_client.clone()).await;
                    let Ok(gas) = get_gas_fee else{
                        let resp_err = get_gas_fee.unwrap_err();
                        return resp_err;
                    };

                    resp!{
                        i64, // the data type
                        gas, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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

pub mod exports{
    pub use super::upload_rendezvous_player_avatar; // `<---rendezvous jwt--->` rendezvous hyper server
    pub use super::tasks_report;
    pub use super::get_token_value;
    pub use super::get_gas_fee;
    pub use super::get_all_user_withdrawals;
    pub use super::get_all_user_deposits;
    pub use super::get_recipient_unclaimed_deposits;
    pub use super::get_all_user_unpaid_checkouts;
    pub use super::get_all_user_paid_checkouts;
    pub use super::get_all_private_galleries_for;
    pub use super::get_all_private_galleries_general_info_for;
    pub use super::get_all_private_collections_for;
    pub use super::get_all_public_collections_for;
    pub use super::get_all_public_collection_nfts;
    pub use super::get_all_galleries_invited_to;
    pub use super::get_invited_friends_wallet_data_of_gallery;
    pub use super::get_user_unaccpeted_invitation_requests;
    pub use super::get_user_unaccpeted_friend_requests;
    pub use super::get_all_my_friends;
    pub use super::get_all_my_followers;
    pub use super::get_all_my_followings;
    pub use super::get_all_user_relations;
    pub use super::get_all_user_reactions; /**** all user comments, likes and dislikes ****/
    pub use super::get_all_nft_reactions; /**** all nft comments, likes and dislikes ****/
    pub use super::get_all_nfts_owned_by;
    pub use super::get_new_clp_event_info;
    pub use super::get_all_user_clp_events_info;
    pub use super::get_friend_suggestions_for_owner;
    pub use super::login;
    pub use super::login_with_identifier_and_password;
    pub use super::login_with_gmail;
    pub use super::login_with_microsoft;
    pub use super::verify_twitter_account;
    pub use super::edit_bio;
    pub use super::edit_extra;
    pub use super::upload_avatar;
    pub use super::upload_banner;
    pub use super::update_password;
    pub use super::forgot_password;
    pub use super::upload_wallet_back;
    pub use super::upload_collection_banner;
    pub use super::upload_private_gallery_back;
    pub use super::make_cid;
    pub use super::request_mail_code;
    pub use super::verify_mail_code;
    pub use super::request_phone_code;
    pub use super::verify_phone_code;
    pub use super::send_private_gallery_invitation_request_to;
    pub use super::send_friend_request_to;
    pub use super::accept_invitation_request;
    pub use super::enter_private_gallery;
    pub use super::accept_friend_request;
    pub use super::remove_user_from_follower;
    pub use super::remove_user_from_freind;
    pub use super::remove_invited_friend_from_gallery;
    pub use super::exit_from_private_gallery;
    pub use super::create_private_gallery;
    pub use super::update_private_gallery;
    pub use super::create_collection;
    pub use super::update_collection;
    pub use super::create_nft;
    pub use super::update_nft; 
    pub use super::create_nft_metadata_uri;
    pub use super::buy_nft; 
    pub use super::mint_nft; 
    pub use super::add_reaction_to_nft;
    pub use super::deposit; /**** gift card money transfer ****/
    pub use super::withdraw; /**** gift card money claim ****/
    pub use super::charge_wallet_request; /**** buy in-app token ****/
    pub use super::register_clp_event;
    // pub use super::request_advieh_collection;
    // ...
}