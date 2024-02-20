


use crate::*;
use crate::adapters::stripe::{create_product, create_price, create_session, StripeCreateCheckoutSessionData};
use crate::events::publishers::action::{UserNotif, NotifExt};
use crate::events::subscribers::handlers::actors::notif::user::{UserListenerActor};
use crate::events::subscribers::handlers::actors::notif::system::{SystemActor, GetSystemUsersMap};
use crate::events::subscribers::handlers::actors::notif::action::{GetUsersNotifsMap, UserActionActor};
use crate::models::clp_events::{ClpEventData, ClpEvent};
use crate::models::users_checkouts::{UserCheckoutData, UserCheckout, NewUserCheckout};
use crate::models::users_clps::{UserClp, RegisterUserClpEventRequest, CancelUserClpEventRequest};
use crate::models::users_collections::{UserCollection, UserCollectionData, NewUserCollectionRequest, UpdateUserCollectionRequest};
use crate::models::users_deposits::{UserDepositData, UserDepositDataWithWalletInfo};
use crate::models::users_fans::{AcceptFriendRequest, AcceptInvitationRequest, EnterPrivateGalleryRequest, FriendData, InvitationRequestData, InvitationRequestDataResponse, RemoveFollower, RemoveFollowing, RemoveFriend, SendFriendRequest, UserFan, UserFanData, UserFanDataWithWalletInfo, UserRelations};
use crate::models::users_galleries::{UserPrivateGalleryInfoDataInvited, NewUserPrivateGalleryRequest, UpdateUserPrivateGalleryRequest, UserPrivateGallery, UserPrivateGalleryData, RemoveInvitedFriendFromPrivateGalleryRequest, SendInvitationRequest, UserPrivateGalleryInfoData, ExitFromPrivateGalleryRequest};
use crate::models::users_nfts::{AddReactionRequest, CreateNftMetadataUriRequest, NewUserNftRequest, NftReactionData, UpdateUserNftRequest, UserNft, UserNftData, UserNftDataWithWalletInfo, UserReactionData};
use crate::models::users_withdrawals::{UserWithdrawal, UserWithdrawalData};
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::helpers::passport::Passport; /* loading Passport macro to use get_user() method on HttpRequest object */
use crate::resp;
use crate::constants::*;
use crate::helpers::misc::*;
use actix::Addr;
use chrono::NaiveDateTime;
use s3req::Storage;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use futures_util::TryStreamExt;
use crate::*;
use crate::models::users::UserRole;
use crate::constants::*;
use crate::helpers::misc::*;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use models::users::{Id, NewIdRequest, UserIdResponse};
use models::users_deposits::{NewUserDepositRequest, UserDeposit};
use models::users_withdrawals::NewUserWithdrawRequest;
use crate::adapters::nftport::*;
use crate::models::token_stats::TokenStatInfoRequest;
use self::models::token_stats::TokenStatInfo;





/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/

#[post("/request-mail-code/{mail}")]
#[passport(user)]
pub(self) async fn request_mail_code(
    req: HttpRequest,
    user_mail: web::Path<String>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-request-mail-code", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
pub(self) async fn verify_mail_code(
    req: HttpRequest,
    check_user_verification_request: web::Json<CheckUserMailVerificationRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
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
pub(self) async fn request_phone_code(
    req: HttpRequest,
    user_phone: web::Path<String>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* we need rate limit in this api since otp providers have rate limits */
                    let identifier_key = format!("{}-request-phone-code", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
pub(self) async fn verify_phone_code(
    req: HttpRequest,
    check_user_verification_request: web::Json<CheckUserPhoneVerificationRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
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
pub(self) async fn login_with_identifier_and_password(
        req: HttpRequest, 
        user_login_info: web::Json<UserLoginInfoRequest>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
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
                return user.get_user_data_response_with_cookie(&user_login_info.clone().device_id, redis_client.clone(), redis_actix_actor, connection).await.unwrap();

            }

            let login_info = user_login_info.to_owned();
            match req.get_passport(login_info, redis_client.clone(), redis_actix_actor, connection).await{
                Ok(ok_resp) => ok_resp,
                Err(err_resp) => err_resp
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

#[post("/signup")]
pub(self) async fn signup_with_identifier_and_password(
        req: HttpRequest, 
        user_login_info: web::Json<UserLoginInfoRequest>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            let login_info = user_login_info.to_owned();

            match req.create_passport(login_info, connection).await{
                Ok(ok_resp) => ok_resp,
                Err(err_resp) => err_resp
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

#[get("/sessions/oauth/google")]
pub(self) async fn session_oauth_google(
        req: HttpRequest,
        google_query: web::Query<GoogleQueryCode>,
        app_state: web::Data<AppState>
    ) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();
    
    match storage.clone().unwrap().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();

            let code = &google_query.code;
            let state = &google_query.state;
            let device_id_ = &google_query.device_id;

            if code.is_empty(){
                resp!{
                    &[u8], // the data type
                    &[], // response data
                    GOOGLE_AUTH_CODE_IS_EMPTY, // response message
                    StatusCode::NOT_ACCEPTABLE, // status code
                    None::<Cookie<'_>>, // cookie
                }
            }

            // request google oauth access token with the passed in code
            let get_google_token = User::request_google_token(&code).await;
            let Ok(google_token) = get_google_token else{
                let err_resp = get_google_token.unwrap_err();
                return err_resp;
            };

            // fetching a google user info with the passed in fetched token info
            let get_google_user = User::get_google_user(google_token).await;
            let Ok(google_user_info) = get_google_user else{
                let err_resp = get_google_user.unwrap_err();
                return err_resp;
            };

            // find a user in db with the google mail in both identifier and mail fields, otherwise insert a new one
            let get_user = User::find_by_identifier_or_mail(&google_user_info.email, connection).await;
            let user_data = match get_user{

                // match takes the ownership of its right side operand
                // cause it returns the actual data in its Ok or Some
                // arm if anything goes well
                Ok(user_info) => {
    
                    // update user info with google user info 
                    let update_user = User::update_user_with_google_info(google_user_info, user_info.id, redis_actix_actor.clone(), connection, redis_client).await;
                    let Ok(user_info) = update_user else{
                        let err_resp = update_user.unwrap_err();
                        return err_resp;
                    };
    
                    user_info
                }, 
                Err(resp) => {

                    // insert a new user with google user info
                    let get_new_user = User::insert_new_google_user(google_user_info, connection, redis_client).await;
                    let Ok(user_info) = get_new_user else{
                        let err_resp = get_new_user.unwrap_err();
                        return err_resp;
                    };
    
                    user_info
                }
            };

            // generate cookie containing both jwts and send response
            user_data.get_user_data_response_with_cookie(
                device_id_,
                redis_client.clone(), 
                redis_actix_actor, 
                connection)
            .await.unwrap()

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
pub(self) async fn verify_twitter_account(
        req: HttpRequest,
        account_name: web::Path<String>,  
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
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
pub(self) async fn tasks_report(
        req: HttpRequest,
        limit: web::Query<Limit>,
        user_id: web::Path<i32>,  
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
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
pub(self) async fn charge_wallet_request(
    req: HttpRequest,
    charge_wallet_request: web::Json<ChargeWalletRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-charge-wallet-request", _id);

                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
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

                        if charge_wallet_request_object.tokens < 5{

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
                            /** -------------------------------------------------------------- */
                            /** --------------------- use stripe gateway --------------------- */
                            /** -------------------------------------------------------------- */
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
                                    stripe will divide the amount by 100 in checkout page to get the cent value
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
pub(self) async fn make_cid(
    req: HttpRequest,
    id_: web::Json<NewIdRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let new_object_id_request = id_.0;
    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let save_user_data = new_id.save(redis_client.to_owned(), redis_actix_actor.clone(), connection).await;
                        let Ok(user_data) = save_user_data else{
                            let resp = save_user_data.unwrap_err();
                            return resp;
                        };

                        /* ----------------------------------------------- */
                        /* creating a default private gallery for the user */
                        /* ----------------------------------------------- */
                        let create_new_gal = UserPrivateGallery::insert(
                            NewUserPrivateGalleryRequest{
                                owner_cid: new_id.clone().new_cid.unwrap_or(String::from("")),
                                gal_name: format!("{} with {} first private gallery", new_id.username, new_id.clone().screen_cid.unwrap_or(String::from(""))),
                                gal_description: format!("{} with {} first private gallery", new_id.username, new_id.clone().screen_cid.unwrap_or(String::from(""))),
                                extra: None,
                                tx_signature: String::from(""),
                                hash_data: String::from(""),
                            }, redis_actix_actor.clone(), connection).await;
                        
                        let Ok(new_gal) = create_new_gal else{
                            let error_resp = create_new_gal.unwrap_err();
                            return error_resp;
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
pub(self) async fn deposit(
    req: HttpRequest,
    contract_address: web::Path<String>,
    deposit: web::Json<NewUserDepositRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    
    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-deposit", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
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



                        let new_balance = user.balance.unwrap() - (deposit_object.amount + gastracker::calculate_gas_in_token(redis_client.clone()).await.unwrap());
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


                        // sender and recipient must be friend of each other
                        let depositor_screen_cid = &user.screen_cid.as_ref().unwrap();
                        let check_we_are_friend = UserFan::are_we_friends(
                            &polygon_recipient_address.clone(), 
                            depositor_screen_cid, connection).await;
                        
                        if check_we_are_friend.is_ok() && *check_we_are_friend.as_ref().unwrap(){

                            let (tx_hash, tid, res_mint_status) = start_minting_card_process(
                                depositor_screen_cid.to_string(),
                                deposit_object.clone(),  
                                contract_address.clone(),
                                contract_owner.clone(),
                                polygon_recipient_address.clone(),
                                deposit_object.nft_img_url.clone(),
                                deposit_object.nft_name,
                                deposit_object.nft_desc,
                                redis_client.clone(),
                                redis_actix_actor.clone(), 
                                connection
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
    
                            let mut uubd = None;

                            if !mint_tx_hash.is_empty(){

                                let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor.clone(), connection).await;
                                let Ok(updated_user_balance_data) = update_user_balance else{
                                    
                                    let err_resp = update_user_balance.unwrap_err();
                                    return err_resp;
                                    
                                };

                                uubd = Some(updated_user_balance_data);
                                
                                match UserDeposit::insert(deposit.to_owned(), mint_tx_hash, token_id, polygon_recipient_address, deposit_object.nft_img_url, redis_actix_actor, connection).await{
                                    Ok(user_deposit_data) => {
    
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
                                
                                if uubd.is_some(){
                                    let updated_user_balance_data = uubd.unwrap();
                                    let new_balance = updated_user_balance_data.balance.unwrap() + deposit_object.amount;
                                    let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor, connection).await;
                                    let Ok(updated_user_data) = update_user_balance else{
        
                                        let err_resp = update_user_balance.unwrap_err();
                                        return err_resp;
                                        
                                    };
                                }
    
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    CANT_MINT_CARD, // response message
                                    StatusCode::FAILED_DEPENDENCY, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            }

                        } else{
                
                            let recipient_username = recipient_info.clone().username;
                            let depositor_username = user.clone().username;
                            let resp_msg = format!("{recipient_username:} Is Not A Friend Of {depositor_username:}");
                            let resp = Response::<'_, &[u8]>{
                                data: Some(&[]),
                                message: &resp_msg,
                                status: 406,
                                is_error: true
                            };
                            return 
                                Ok(HttpResponse::NotAcceptable().json(resp));
                            
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
pub(self) async fn get_all_user_deposits(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                                Vec<UserDepositDataWithWalletInfo>, // the data type
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
pub(self) async fn withdraw(
    req: HttpRequest,
    contract_address: web::Path<String>,
    withdraw: web::Json<NewUserWithdrawRequest>,
    app_state: web::Data<AppState>,
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-withdraw", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
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
                            redis_client.clone(),
                            redis_actix_actor.clone(), 
                            connection
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

                        let new_balance = if user.balance.is_none(){0 + deposit_info.amount} else{user.balance.unwrap() + deposit_info.amount};
                        let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor.clone(), connection).await;
                        let Ok(updated_user_balance_data) = update_user_balance else{

                            let err_resp = update_user_balance.unwrap_err();
                            return err_resp;
                            
                        };

                        if !transfer_tx_hash.is_empty(){

                            match UserWithdrawal::insert(withdraw.to_owned(), transfer_tx_hash, connection).await{
                                Ok(user_withdrawal_data) => {
                                    
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

                            let new_balance = updated_user_balance_data.balance.unwrap() - deposit_info.amount;
                            let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actix_actor, connection).await;
                            let Ok(updated_user_balance_data) = update_user_balance else{

                                let err_resp = update_user_balance.unwrap_err();
                                return err_resp;
                                
                            };

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
pub(self) async fn get_all_user_withdrawals(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
pub(self) async fn get_all_user_unpaid_checkouts(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
pub(self) async fn get_all_user_paid_checkouts(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
pub(self) async fn get_recipient_unclaimed_deposits(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                                Vec<UserDepositDataWithWalletInfo>, // the data type
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
pub(self) async fn edit_bio(
    req: HttpRequest,
    update_bio_request: web::Json<UpdateBioRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
pub(self) async fn edit_extra(
    req: HttpRequest,
    update_extra_request: web::Json<UpdateExtraRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
pub(self) async fn upload_wallet_back(
    req: HttpRequest,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart,
) -> PanelHttpResponse{

    /* extracting shared storage data */
    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
                match req.get_user(granted_role, connection).await{
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
pub(self) async fn upload_avatar(
    req: HttpRequest,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart, /* form-data implementation to receive stream of byte fields */
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
pub(self) async fn upload_banner(
    req: HttpRequest,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart, /* form-data implementation to receive stream of byte fields */
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
pub(self) async fn update_password(
    req: HttpRequest,
    new_password_request: web::Json<NewPasswordRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-update-password", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        match User::update_password(_id, new_password_request, connection).await{
                            Ok(updated_user) => {

                                resp!{
                                    UserData, // the data type
                                    updated_user, // response data
                                    NEW_PASSWORD_UPDATED, // response message
                                    StatusCode::OK, // status code
                                    None::<Cookie<'_>>, // cookie
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

#[get("/profile/notifs/get/")]
#[passport(user)]
pub(self) async fn get_notifications(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    users_action_subscriber_actor: web::Data<Addr<UserActionActor>>,
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();
    let users_action_subscriber_actor = users_action_subscriber_actor.get_ref().to_owned();
    
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let redis_key = format!("user_notif_{}", _id);
                    let get_users_notifs: String = redis_client.clone().get(redis_key).unwrap_or(
                        serde_json::to_string_pretty(&UserNotif::default()).unwrap()
                    );
                    let mut user_notifs = serde_json::from_str::<UserNotif>(&get_users_notifs).unwrap();

                    let from = limit.from.unwrap_or(0) as usize;
                    let to = limit.to.unwrap_or(10) as usize;
            
                    if to < from {
                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: INVALID_QUERY_LIMIT,
                            status: 406,
                            is_error: true
                        };
                        return 
                            Ok(HttpResponse::NotAcceptable().json(resp))
                        
                    }

                    // sending an async message to users_action_subscriber_actor to get its latest 
                    // state which contains the whole app notifs for all users
                    /* 
                        let get_users_notifs = users_action_subscriber_actor
                            .send(GetUsersNotifsMap)
                            .await
                            .unwrap();
                    */

                    let mut all_user_notifs = user_notifs.get_user_notifs().await;
                    all_user_notifs.sort_by(|n1, n2|{
    
                        let n1_fired_at = n1.fired_at;
                        let n2_fired_at = n2.fired_at;
            
                        n2_fired_at.cmp(&n1_fired_at)
            
                    });

                    let sliced = if from < all_user_notifs.len(){
                        if all_user_notifs.len() > to{
                            let data = &all_user_notifs[from..to+1];
                            data.to_vec()
                        } else{
                            let data = &all_user_notifs[from..all_user_notifs.len()];
                            data.to_vec()
                        }
                    } else{
                        vec![]
                    };

                    // updating the user notif data with the sorted one to respond the user with that
                    user_notifs.update_new_slice_notifs(sliced);

                    resp!{
                        UserNotif, // the data type
                        user_notifs, // response data
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

/* >_______________________________________________________________________________
    this api must gets called by player with his conse rendezvous hyper server JWT 
    passed in to the request header 
    _______________________________________________________________________________
*/
#[post("/rendezvous/player/{player_id}/upload/avatar")]
pub(self) async fn upload_rendezvous_player_avatar(
    req: HttpRequest, 
    player_id: web::Path<String>, // mongodb objectid
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
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
    
                    let storage = app_state.app_sotrage.as_ref().to_owned();
                    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                    let player_id_img_key = format!("{player_id:}-img");

                    let get_redis_conn = redis_client.get_async_connection().await;
                    let Ok(mut redis_conn) = get_redis_conn else{

                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
pub(self) async fn create_private_gallery(
    req: HttpRequest,
    new_private_gallery_request: web::Json<NewUserPrivateGalleryRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-create-private-gallery", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
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

                        match UserPrivateGallery::insert(create_private_gallery_request_object, redis_actix_actor, connection).await{
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
pub(self) async fn update_private_gallery(
    req: HttpRequest,
    gal_id: web::Path<i32>,
    update_private_gallery_request: web::Json<UpdateUserPrivateGalleryRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-update-private-gallery", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
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
                            redis_client.clone(),
                            redis_actix_actor,
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
pub(self) async fn remove_invited_friend_from_gallery(
    req: HttpRequest,
    remove_invited_friend_request: web::Json<RemoveInvitedFriendFromPrivateGalleryRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
                    let is_request_verified = helpers::kyced::verify_request(
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
pub(self) async fn exit_from_private_gallery(
    req: HttpRequest,
    exit_from_private_gallery: web::Json<ExitFromPrivateGalleryRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
                    let is_request_verified = helpers::kyced::verify_request(
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

                    match UserPrivateGallery::exit_from_private_gallery(exit_from_private_gallery, redis_client.clone(), redis_actix_actor, connection).await{
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
pub(self) async fn send_private_gallery_invitation_request_to(
    req: HttpRequest,
    send_invitation_request: web::Json<SendInvitationRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                    let is_request_verified = helpers::kyced::verify_request(
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

                    match UserPrivateGallery::send_invitation_request_to(send_invitation_request, redis_client.clone(), redis_actix_actor, connection).await{
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
pub(self) async fn get_all_private_galleries_for(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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

                    match UserPrivateGallery::get_all_for(&user.screen_cid.unwrap(), limit, redis_client.clone(), connection).await{
                        
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
pub(self) async fn get_all_galleries_invited_to(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                        redis_client.clone(),
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
pub(self) async fn get_invited_friends_wallet_data_of_gallery(
    req: HttpRequest,
    gal_id: web::Path<i32>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                        limit, redis_client.clone(),
                        connection).await{
                        
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
pub(self) async fn get_user_unaccepted_invitation_requests(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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

                    match UserFan::get_user_unaccepted_invitation_requests(
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
pub(self) async fn get_user_unaccepted_friend_requests(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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

                    match UserFan::get_user_unaccepted_friend_requests(
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
pub(self) async fn accept_invitation_request(
    req: HttpRequest,
    accept_invitation_request: web::Json<AcceptInvitationRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
                    let is_request_verified = helpers::kyced::verify_request(
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
pub(self) async fn enter_private_gallery(
    req: HttpRequest,
    enter_private_gallery_request: web::Json<EnterPrivateGalleryRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
                    let is_request_verified = helpers::kyced::verify_request(
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
pub(self) async fn get_all_user_relations(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
pub(self) async fn accept_friend_request(
    req: HttpRequest,
    accept_friend_request: web::Json<AcceptFriendRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
                    let is_request_verified = helpers::kyced::verify_request(
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

                    match UserFan::accept_friend_request(accept_friend_request, redis_actix_actor, connection).await{
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
pub(self) async fn send_friend_request_to(
    req: HttpRequest,
    send_friend_request_to: web::Json<SendFriendRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
                    let is_request_verified = helpers::kyced::verify_request(
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

                    match UserFan::send_friend_request_to(send_friend_request_to, redis_actix_actor, connection).await{
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
pub(self) async fn remove_user_from_follower(
    req: HttpRequest,
    remove_follower_request: web::Json<RemoveFollower>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
                    let is_request_verified = helpers::kyced::verify_request(
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
pub(self) async fn remove_user_from_friend(
    req: HttpRequest,
    remove_friend_request: web::Json<RemoveFriend>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
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
                    let is_request_verified = helpers::kyced::verify_request(
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

                    match UserFan::remove_friend(remove_friend_request, redis_client.clone(), redis_actix_actor.clone(), connection).await{
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

#[post("/fan/remove/following")]
#[passport(user)]
pub(self) async fn remove_user_from_following(
    req: HttpRequest,
    remove_following_request: web::Json<RemoveFollowing>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let remove_following_request = remove_following_request.to_owned();
                    
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
                    let is_request_verified = helpers::kyced::verify_request(
                        _id, 
                        &remove_following_request.owner_cid, 
                        &remove_following_request.tx_signature, 
                        &remove_following_request.hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    match UserFan::remove_following(remove_following_request, redis_client.clone(), redis_actix_actor.clone(), connection).await{
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
pub(self) async fn get_all_my_friends(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
pub(self) async fn get_all_my_followings(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                                Vec<UserFanDataWithWalletInfo>, //// the data type
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
pub(self) async fn get_all_my_followers(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
pub(self) async fn get_friend_suggestions_for_owner(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                                Vec<UserWalletInfoResponseForUserSuggestions>, //// the data type
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
pub(self) async fn get_all_public_collections_for(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
pub(self) async fn get_all_private_galleries_general_info_for(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                        limit, redis_client.clone(), connection).await{
                        
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

#[post("/collection/{col_id}/upload/background")]
#[passport(user)]
pub(self) async fn upload_collection_banner(
    req: HttpRequest,
    col_id: web::Path<i32>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart,
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                        redis_actix_actor.clone(),
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
pub(self) async fn create_collection(
    req: HttpRequest,
    new_user_collection_request: web::Json<NewUserCollectionRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-create-collection", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
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
pub(self) async fn update_collection(
    req: HttpRequest,
    update_user_collection_request: web::Json<UpdateUserCollectionRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-update-collection", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
                            _id, 
                            &update_user_collection_request.owner_cid, 
                            &update_user_collection_request.tx_signature, 
                            &update_user_collection_request.hash_data, 
                            None,
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

#[get("/collection/{col_id}/get/all/minted-nfts/")]
#[passport(user)]
pub(self) async fn get_all_public_collection_nfts(
    req: HttpRequest,
    col_id: web::Path<i32>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
pub(self) async fn get_all_private_collections_for(
    req: HttpRequest,
    gal_id: web::Path<i32>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                        limit, redis_client.clone(), connection).await{
                        
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

#[get("/collection/firends-view/get/all/private/in-gallery/{gal_id}/for/{who}/")]
#[passport(user)]
pub(self) async fn get_all_private_collections_for_invited_friends(
    req: HttpRequest,
    gal_id_who_screen_cid: web::Path<(i32, String)>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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

                    // caller must be invited into the gallery by the owner
                    let gal_id_who_screen_cid = gal_id_who_screen_cid.to_owned();
                    let gal_id = gal_id_who_screen_cid.0;
                    let who_screen_cid = gal_id_who_screen_cid.1;
                    match UserCollection::get_all_private_collections_for_invited_friends(
                        &user.screen_cid.unwrap(), // caller 
                        &who_screen_cid.to_owned(), // owner
                        gal_id,
                        limit, redis_client.clone(), connection).await{
                        
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

#[post("/nft/create")]
#[passport(user)]
pub(self) async fn create_nft(
    req: HttpRequest,
    new_user_nft_request: web::Json<NewUserNftRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-create-nft", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
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
pub(self) async fn create_nft_metadata_uri(
    req: HttpRequest,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut payload: Multipart,
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-create-nft-metadata-uri", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
                            _id, 
                            &create_nft_metadata_uri_request.caller_cid, 
                            &create_nft_metadata_uri_request.tx_signature, 
                            &create_nft_metadata_uri_request.hash_data, 
                            None,
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
                            redis_actix_actor.clone(),
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

#[post("/gallery/{gal_id}/upload/background")]
#[passport(user)]
pub(self) async fn upload_private_gallery_back(
    req: HttpRequest,
    gal_id: web::Path<i32>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart,
) -> PanelHttpResponse{

    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                        redis_client.clone(),
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
pub(self) async fn update_nft(
    req: HttpRequest,
    update_user_nft_request: web::Json<UpdateUserNftRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
   
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
                    let is_request_verified = helpers::kyced::verify_request(
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
pub(self) async fn add_reaction_to_nft(
    req: HttpRequest,
    user_add_nft_reaction: web::Json<AddReactionRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                        
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
                    let is_request_verified = helpers::kyced::verify_request(
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
pub(self) async fn buy_nft(
    req: HttpRequest,
    user_buy_nft_request: web::Json<UpdateUserNftRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-buy-nft", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
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
pub(self) async fn mint_nft(
    req: HttpRequest,
    user_mint_nft_request: web::Json<UpdateUserNftRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-mint-nft", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        let is_request_verified = helpers::kyced::verify_request(
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

#[get("/nft/get/all/onchain/for/{who_screen_cid}/")]
#[passport(user)]
pub(self) async fn get_all_nfts_owned_by(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{
    
    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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

                    /* caller must have an screen_cid or has created a wallet */
                    let get_user = User::find_by_screen_cid(&who_screen_cid.to_owned(), connection).await;
                    let Ok(user) = get_user else{
                        let err_resp = get_user.unwrap_err();
                        return err_resp;
                    };

                    match UserNft::get_all_nfts_owned_by(
                        &who_screen_cid.to_owned(),
                        limit, connection).await{
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

#[get("/collection/get/all/onchain/for/{who_screen_cid}/")]
#[passport(user)]
pub(self) async fn get_all_collections_owned_by(
    req: HttpRequest,
    who_screen_cid: web::Path<String>,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{
    
    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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

                    /* caller must have an screen_cid or has created a wallet */
                    let get_user = User::find_by_screen_cid(&who_screen_cid.to_owned(), connection).await;
                    let Ok(user) = get_user else{
                        let err_resp = get_user.unwrap_err();
                        return err_resp;
                    };

                    match UserNft::get_all_collections_owned_by(
                        &who_screen_cid.to_owned(),
                        limit, connection).await{
                        Ok(user_cols) => {

                            resp!{
                                OnchainContracts, //// the data type
                                user_cols, //// response data
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

#[get("/nft/{nft_id}/reaction/get/all")]
#[passport(user)]
pub(self) async fn get_all_nft_reactions(
    req: HttpRequest,
    nft_id: web::Path<i32>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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

                    let nft_id = nft_id.to_owned();
                    match UserNft::get_all_nft_reactions(
                        nft_id,
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
pub(self) async fn register_clp_event(
    req: HttpRequest,
    register_clp_event_request: web::Json<RegisterUserClpEventRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-register-clp-event", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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
                        
                        let get_clp_event = ClpEvent::find_by_id(register_clp_event_request.clp_event_id, connection).await;
                        let Ok(clp_event) = get_clp_event else{
                            let err_resp = get_clp_event.unwrap_err();
                            return err_resp;
                        };

                        let get_current_users_in_this_event = UserClp::get_all_users_in_clp_event(register_clp_event_request.clp_event_id, connection).await;
                        let Ok(current_users_in_this_event) = get_current_users_in_this_event else{
                            let err_resp = get_current_users_in_this_event.unwrap_err();
                            return err_resp;
                        };

                        /** ------------------------------------------------------------------ */
                        /** the is_locked field will be updated by clp subscriber actor worker */ 
                        /** ------------------------------------------------------------------ */ 
                        // means it has been started already and can't register any more
                        if clp_event.is_locked{
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                CLP_EVENT_IS_LOCKED, //// response message
                                StatusCode::NOT_ACCEPTABLE, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }

                        /** ------------------------------------------------------------------ */
                        /** the expire_at field will be updated by clp subscriber actor worker */ 
                        /** ------------------------------------------------------------------ */
                        // means it has been expired already and can't register any more
                        if chrono::Local::now().timestamp() > clp_event.expire_at{
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                CLP_EVENT_IS_EXPIRED, //// response message
                                StatusCode::NOT_ACCEPTABLE, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }

                        if clp_event.max_supply as usize == current_users_in_this_event.len(){
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                CLP_EVENT_IS_FULL, //// response message
                                StatusCode::NOT_ACCEPTABLE, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }

                        let register_clp_event_request = register_clp_event_request.to_owned();
                        
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
                        let is_request_verified = helpers::kyced::verify_request(
                            _id, 
                            &register_clp_event_request.participant_cid, 
                            &register_clp_event_request.tx_signature, 
                            &register_clp_event_request.hash_data, 
                            Some(clp_event.mint_price),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };

                        let get_user_clp_event = UserClp::find_by_participant_and_event_id(_id, clp_event.id, connection).await;
                        if get_user_clp_event.is_ok(){
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                USER_CLP_EVENT_ALREADY_REGISTERED, //// response message
                                StatusCode::FOUND, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }
                        
                        match UserClp::insert(
                            clp_event.mint_price, 
                            _id, 
                            clp_event.id, 
                            redis_client.clone(),
                            redis_actix_actor,
                            connection
                        ).await{
                            Ok(user_clp_data) => {

                                resp!{
                                    UserClp, //// the data type
                                    user_clp_data, //// response data
                                    CREATED, //// response message
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

#[post("/clp/cancel")]
#[passport(user)]
pub(self) async fn cancel_clp_event(
    req: HttpRequest,
    cancel_clp_event_request: web::Json<CancelUserClpEventRequest>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let identifier_key = format!("{}-cancel-clp-event", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "cancel_clp_event");
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
                        
                        let cancel_clp_event_request = cancel_clp_event_request.to_owned();
                        
                        let get_clp_event = ClpEvent::find_by_id(cancel_clp_event_request.clp_event_id, connection).await;
                        let Ok(clp_event) = get_clp_event else{
                            let err_resp = get_clp_event.unwrap_err();
                            return err_resp;
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
                        let is_request_verified = helpers::kyced::verify_request(
                            _id, 
                            &cancel_clp_event_request.participant_cid, 
                            &cancel_clp_event_request.tx_signature, 
                            &cancel_clp_event_request.hash_data, 
                            Some(clp_event.mint_price),
                            connection
                        ).await;

                        let Ok(user) = is_request_verified else{
                            let error_resp = is_request_verified.unwrap_err();
                            return error_resp; /* terminate the caller with an actix http response object */
                        };
                        

                        let get_user_clp_event = UserClp::find_by_participant_and_event_id(_id, clp_event.id, connection).await;
                        if get_user_clp_event.is_ok(){
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                USER_CLP_EVENT_ALREADY_REGISTERED, //// response message
                                StatusCode::FOUND, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        }
                        
                        match UserClp::cancel_reservation(
                            _id, 
                            clp_event.id, 
                            redis_client.clone(),
                            redis_actix_actor,
                            connection
                        ).await{
                            Ok(user_data) => {

                                resp!{
                                    UserData, //// the data type
                                    user_data, //// response data
                                    CREATED, //// response message
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

#[get("/clp/get/new")]
#[passport(user)]
pub(self) async fn get_new_clp_event_info(
    req: HttpRequest,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{
    
    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                
                    match ClpEvent::get_latest(connection).await{
                        Ok(clp_event_data) => {

                            resp!{
                                ClpEventData, //// the data type
                                clp_event_data, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }

                        },
                        Err(resp) => resp
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

#[get("/clp/get/all/")]
#[passport(user)]
pub(self) async fn get_all_user_clp_events_info(
    req: HttpRequest,
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{
    
    let storage = app_state.app_sotrage.as_ref().to_owned(); /* as_ref() returns shared reference */
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
pub(self) async fn get_token_value(
        req: HttpRequest,  
        tokens: web::Path<i64>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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
                    
                    if let Err(resp) = TokenStatInfo::save(
                        TokenStatInfoRequest{
                            user_id: _id,
                            requested_tokens: tokens.to_owned(),
                            usd_token_price: value.0,
                        },
                        connection).await{
                            error!("can't store token stat info into db, check the log file (TokenStatInfo::save)");
                        }
                    
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
pub(self) async fn get_gas_fee(
        req: HttpRequest,  
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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

#[get("/get/all/top/")]
#[passport(user)]
pub(self) async fn get_top_users(
    req: HttpRequest,  
    limit: web::Query<Limit>,    
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse {


    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match User::get_top_users(connection, redis_client.clone(), limit).await{
                        
                        Ok(top_users_wallet_info) => {
                            resp!{
                                TopUsers, // the data type
                                top_users_wallet_info, // response data
                                FETCHED, // response message
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

#[get("/nft/get/{asset_id}")]
#[passport(user)]
pub(self) async fn get_single_nft(
    req: HttpRequest,  
    asset_id: web::Path<i32>,    
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse {


    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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

                    match UserNft::find_by_id(asset_id.to_owned(), connection).await{
                        
                        Ok(user_nft_data) => {
                            resp!{
                                UserNftDataWithWalletInfo, // the data type
                                UserNftDataWithWalletInfo{ 
                                    id: user_nft_data.id, 
                                    contract_address: user_nft_data.contract_address, 
                                    current_owner_wallet_info: {
                                        let user = User::find_by_screen_cid(&user_nft_data.current_owner_screen_cid, connection).await.unwrap();
                                        UserWalletInfoResponse{
                                            username: user.username,
                                            avatar: user.avatar,
                                            bio: user.bio,
                                            banner: user.banner,
                                            mail: user.mail,
                                            screen_cid: user.screen_cid,
                                            extra: user.extra,
                                            stars: user.stars,
                                            created_at: user.created_at.to_string(),
                                        }
                                    }, 
                                    metadata_uri: user_nft_data.metadata_uri, 
                                    extra: user_nft_data.extra, 
                                    onchain_id: user_nft_data.onchain_id, 
                                    nft_name: user_nft_data.nft_name, 
                                    is_minted: user_nft_data.is_minted, 
                                    nft_description: user_nft_data.nft_description, 
                                    current_price: user_nft_data.current_price, 
                                    is_listed: user_nft_data.is_listed, 
                                    freeze_metadata: user_nft_data.freeze_metadata, 
                                    comments: user_nft_data.comments, 
                                    likes: user_nft_data.likes, 
                                    tx_hash: user_nft_data.tx_hash, 
                                    created_at: user_nft_data.created_at.to_string(), 
                                    updated_at: user_nft_data.updated_at.to_string(),
                                    attributes: user_nft_data.attributes, 
                                }, // response data
                                FETCHED, // response message
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

#[get("/nft/get/")]
#[passport(user)]
pub(self) async fn get_nfts_owned_by(
    req: HttpRequest,  
    limit: web::Query<Limit>,
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse {


    let storage = app_state.app_sotrage.as_ref().to_owned();
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
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid or has created a wallet */
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


                    let from = limit.from.unwrap_or(0) as usize;
                    let to = limit.to.unwrap_or(10) as usize;

                    if to < from {
                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: INVALID_QUERY_LIMIT,
                            status: 406,
                            is_error: true
                        };
                        return 
                            Ok(HttpResponse::NotAcceptable().json(resp));
                    }

                    match UserNft::get_all_by_current_owner(&user.screen_cid.unwrap(), connection).await{
                        
                        Ok(mut user_nfts_data) => {

                            user_nfts_data.sort_by(|nft1, nft2|{

                                let nft1_created_at = NaiveDateTime
                                    ::parse_from_str(nft1.clone().created_at.as_str(), "%Y-%m-%d %H:%M:%S%.f")
                                    .unwrap();
                    
                                let nft2_created_at = NaiveDateTime
                                    ::parse_from_str(nft2.clone().created_at.as_str(), "%Y-%m-%d %H:%M:%S%.f")
                                    .unwrap();
                    
                                nft2_created_at.cmp(&nft1_created_at)
                    
                            });      
                            
                            let sliced = if from < user_nfts_data.len(){
                                if user_nfts_data.len() > to{
                                    let data = &user_nfts_data[from..to+1];
                                    data.to_vec()
                                } else{
                                    let data = &user_nfts_data[from..user_nfts_data.len()];
                                    data.to_vec()
                                }
                            } else{
                                vec![]
                            };


                            resp!{
                                Vec<UserNftDataWithWalletInfo>, // the data type
                                {
                                    sliced
                                        .into_iter()
                                        .map(|user_nft_data|{
                                            UserNftDataWithWalletInfo{ 
                                                id: user_nft_data.id, 
                                                contract_address: user_nft_data.contract_address, 
                                                current_owner_wallet_info: {
                                                    let user = User::find_by_screen_cid_none_async(&user_nft_data.current_owner_screen_cid, connection).unwrap();
                                                    UserWalletInfoResponse{
                                                        username: user.username,
                                                        avatar: user.avatar,
                                                        bio: user.bio,
                                                        banner: user.banner,
                                                        mail: user.mail,
                                                        screen_cid: user.screen_cid,
                                                        extra: user.extra,
                                                        stars: user.stars,
                                                        created_at: user.created_at.to_string(),
                                                    }
                                                }, 
                                                metadata_uri: user_nft_data.metadata_uri, 
                                                extra: user_nft_data.extra, 
                                                onchain_id: user_nft_data.onchain_id, 
                                                nft_name: user_nft_data.nft_name, 
                                                is_minted: user_nft_data.is_minted, 
                                                nft_description: user_nft_data.nft_description, 
                                                current_price: user_nft_data.current_price, 
                                                is_listed: user_nft_data.is_listed, 
                                                freeze_metadata: user_nft_data.freeze_metadata, 
                                                comments: user_nft_data.comments, 
                                                likes: user_nft_data.likes, 
                                                tx_hash: user_nft_data.tx_hash, 
                                                created_at: user_nft_data.created_at.to_string(), 
                                                updated_at: user_nft_data.updated_at.to_string(),
                                                attributes: user_nft_data.attributes, 
                                            }
                                        })
                                        .collect::<Vec<UserNftDataWithWalletInfo>>()
                                }, // response data
                                FETCHED, // response message
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
    pub use super::get_top_users;
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
    pub use super::get_user_unaccepted_invitation_requests;
    pub use super::get_user_unaccepted_friend_requests;
    pub use super::get_all_my_friends;
    pub use super::get_all_my_followers;
    pub use super::get_all_my_followings;
    pub use super::get_all_user_relations;
    pub use super::get_all_nft_reactions; /**** all nft comments, likes and dislikes ****/
    pub use super::get_all_nfts_owned_by;
    pub use super::get_all_collections_owned_by;
    pub use super::get_new_clp_event_info;
    pub use super::get_all_user_clp_events_info;
    pub use super::get_friend_suggestions_for_owner;
    pub use super::get_all_private_collections_for_invited_friends;
    pub use super::get_single_nft;
    pub use super::get_nfts_owned_by;
    pub use super::get_notifications;
    pub use super::session_oauth_google;
    pub use super::create_collection;
    pub use super::update_collection;
    pub use super::upload_collection_banner;
    pub use super::login_with_identifier_and_password;
    pub use super::signup_with_identifier_and_password;
    pub use super::verify_twitter_account;
    pub use super::edit_bio;
    pub use super::edit_extra;
    pub use super::upload_avatar;
    pub use super::upload_banner;
    pub use super::update_password;
    pub use super::upload_wallet_back;
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
    pub use super::remove_user_from_friend;
    pub use super::remove_user_from_following;
    pub use super::remove_invited_friend_from_gallery;
    pub use super::exit_from_private_gallery;
    pub use super::create_private_gallery;
    pub use super::update_private_gallery;
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
    pub use super::cancel_clp_event;
}