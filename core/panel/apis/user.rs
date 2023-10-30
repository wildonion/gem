


use crate::*;
use crate::adapters::stripe::{create_product, create_price, create_session, StripeCreateCheckoutSessionData};
use crate::models::users_checkouts::{UserCheckoutData, UserCheckout, NewUserCheckout};
use crate::models::users_collections::{UserCollection, UserCollectionData};
use crate::models::users_deposits::UserDepositData;
use crate::models::users_fans::InvitationRequestDataResponse;
use crate::models::users_galleries::{NewUserPrivateGalleryRequest, UpdateUserPrivateGalleryRequest, UserPrivateGallery, UserPrivateGalleryData, RemoveInvitedFriendFromPrivateGalleryRequest, SendInvitationRequest};
use crate::models::users_withdrawals::{UserWithdrawal, UserWithdrawalData};
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use s3::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use futures_util::TryStreamExt;
use crate::*;
use crate::models::users::UserRole;
use crate::constants::*;
use crate::misc::*;
use s3::*;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use models::users::{Id, NewIdRequest, UserIdResponse};
use models::users_deposits::{NewUserDepositRequest, UserDeposit};
use models::users_withdrawals::NewUserWithdrawRequest;
use wallexerr::Wallet;
use crate::adapters::nftport::*;




/*
     -------------------------------
    |          SWAGGER DOCS
    | ------------------------------
    |
    |

*/
#[derive(OpenApi)]
#[openapi(
    paths(
        login,
        login_with_identifier_and_password,
        verify_twitter_account,
        tasks_report,
        make_cid,
        withdraw,
        deposit,
        get_all_user_withdrawals,
        get_all_user_deposits,
        get_recipient_unclaimed_deposits,
        request_mail_code,
        verify_mail_code,
        request_phone_code,
        verify_phone_code,
    ),
    components(
        schemas(
            UserData,
            FetchUserTaskReport,
            UserLoginInfoRequest,
            TaskData,
            ReportTaskData,
            UserDepositData,
            UserWithdrawalData,
            CheckUserMailVerificationRequest,
            CheckUserPhoneVerificationRequest,
            NewUserDepositRequest,
            NewIdRequest,
            UserIdResponse,
            ChargeWalletRequest
        )
    ),
    tags(
        (name = "crate::apis::user", description = "User Endpoints")
    ),
    info(
        title = "User Access APIs"
    ),
    modifiers(&SecurityAddon),
)]
pub struct UserApiDoc;
struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}

/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/
#[utoipa::path(
    context_path = "/user",
    responses(
        (status=200, description="Loggedin Successfully", body=UserData),
        (status=201, description="Registered Successfully", body=UserData),
        (status=500, description="Can't Generate Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("identifier" = String, Path, description = "login identifier")
    ),
    tag = "crate::apis::user",
)]
#[post("/login/{identifier}")]
async fn login(
        req: HttpRequest, 
        login_identifier: web::Path<String>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            /* we can pass usernmae by reference or its slice form instead of cloning it */
            match User::find_by_identifier(&login_identifier.to_owned(), connection).await{
                Ok(user) => {
        
                    /* generate cookie 🍪 from token time and jwt */
                    /* since generate_cookie_and_jwt() takes the ownership of the user instance we must clone it then call this */
                    let keys_info = user.clone().generate_cookie_and_jwt().unwrap();
                    let cookie_token_time = keys_info.1;
                    let jwt = keys_info.2;

                    let now = chrono::Local::now().naive_local();
                    let updated_user = diesel::update(users.find(user.id))
                        .set((last_login.eq(now), token_time.eq(cookie_token_time)))
                        .returning(FetchUser::as_returning())
                        .get_result(connection)
                        .unwrap();
                    
                    let user_login_data = UserData{
                        id: user.id,
                        region: user.region.clone(),
                        username: user.username.clone(),
                        bio: user.bio.clone(),
                        avatar: user.avatar.clone(),
                        banner: user.banner.clone(),
                        wallet_background: user.wallet_background.clone(),
                        activity_code: user.activity_code.clone(),
                        twitter_username: user.twitter_username.clone(),
                        facebook_username: user.facebook_username.clone(),
                        discord_username: user.discord_username.clone(),
                        identifier: user.identifier.clone(),
                        user_role: {
                            match user.user_role.clone(){
                                UserRole::Admin => "Admin".to_string(),
                                UserRole::User => "User".to_string(),
                                _ => "Dev".to_string(),
                            }
                        },
                        token_time: updated_user.token_time,
                        balance: updated_user.balance,
                        last_login: { 
                            if updated_user.last_login.is_some(){
                                Some(updated_user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: user.created_at.to_string(),
                        updated_at: updated_user.updated_at.to_string(),
                        mail: user.mail,
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
                        stars: user.stars
                    };

                    resp!{
                        UserData, // the data type
                        user_login_data, // response data
                        LOGGEDIN, // response message
                        StatusCode::OK, // status code,
                        Some(keys_info.0), // cookie 
                    } 

                },
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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=200, description="Verification Code Sent Successfully", body=UserData),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("mail" = String, Path, description = "user mail")
    ),
    tag = "crate::apis::user",
)]
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
            match User::passport(req, granted_role, connection).await{
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
                    
                        match User::send_mail_verification_code_to(_id, user_mail.to_owned(), connection).await{
                            
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

#[utoipa::path(
    context_path = "/user",
    request_body = CheckUserMailVerificationRequest,
    responses(
        (status=200, description="Mail Verified Successfully", body=UserData),
        (status=500, description="Storage Issue", body=[u8])
    ),
    tag = "crate::apis::user",
)]
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match User::check_mail_verification_code(check_user_verification_request.to_owned(), _id, connection).await{
                        
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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=200, description="Verification Code Sent Successfully", body=UserData),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("phone" = String, Path, description = "user phone")
    ),
    tag = "crate::apis::user",
)]
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
            match User::passport(req, granted_role, connection).await{
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
                        
                      
                        match User::send_phone_verification_code_to(_id, user_phone.to_owned(), user_ip.clone(), connection).await{
                            
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

#[utoipa::path(
    context_path = "/user",
    request_body = CheckUserPhoneVerificationRequest,
    responses(
        (status=200, description="Phone Verified Successfully", body=UserData),
        (status=500, description="Storage Issue", body=[u8])
    ),
    tag = "crate::apis::user",
)]
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match User::check_phone_verification_code(check_user_verification_request.to_owned(), _id, connection).await{
                        
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

#[utoipa::path(
    context_path = "/user",
    request_body = UserLoginInfoRequest,
    responses(
        (status=200, description="Loggedin Successfully", body=UserData),
        (status=201, description="Registered Successfully", body=UserData),
        (status=500, description="Can't Generate Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    tag = "crate::apis::user",
)]
#[post("/login")]
async fn login_with_identifier_and_password(
        req: HttpRequest, 
        user_login_info: web::Json<UserLoginInfoRequest>,
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            let login_info = user_login_info.to_owned();

            /* we can pass usernmae by reference or its slice form instead of cloning it */
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
        
                    /* generate cookie 🍪 from token time and jwt */
                    /* since generate_cookie_and_jwt() takes the ownership of the user instance we must clone it then call this */
                    let keys_info = user.clone().generate_cookie_and_jwt().unwrap();
                    let cookie_token_time = keys_info.1;
                    let jwt = keys_info.2;

                    let now = chrono::Local::now().naive_local();
                    let updated_user = diesel::update(users.find(user.id))
                        .set((last_login.eq(now), token_time.eq(cookie_token_time)))
                        .returning(FetchUser::as_returning())
                        .get_result(connection)
                        .unwrap();
                    
                    let user_login_data = UserData{
                        id: user.id,
                        region: user.region.clone(),
                        username: user.username.clone(),
                        bio: user.bio.clone(),
                        avatar: user.avatar.clone(),
                        banner: user.banner.clone(),
                        wallet_background: user.wallet_background.clone(),
                        activity_code: user.activity_code.clone(),
                        twitter_username: user.twitter_username.clone(),
                        facebook_username: user.facebook_username.clone(),
                        discord_username: user.discord_username.clone(),
                        identifier: user.identifier.clone(),
                        user_role: {
                            match user.user_role.clone(){
                                UserRole::Admin => "Admin".to_string(),
                                UserRole::User => "User".to_string(),
                                _ => "Dev".to_string(),
                            }
                        },
                        token_time: updated_user.token_time,
                        balance: updated_user.balance,
                        last_login: { 
                            if updated_user.last_login.is_some(){
                                Some(updated_user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: user.created_at.to_string(),
                        updated_at: updated_user.updated_at.to_string(),
                        mail: user.mail,
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
                        stars: user.stars
                    };

                    resp!{
                        UserData, // the data type
                        user_login_data, // response data
                        LOGGEDIN, // response message
                        StatusCode::OK, // status code,
                        Some(keys_info.0), // cookie 
                    } 

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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=200, description="Updated Successfully", body=UserData),
        (status=404, description="User Not Found", body=i32), // not found by id
        (status=404, description="User Not Found", body=String), // not found by identifier
        (status=404, description="No Value Found In Cookie Or JWT In Header", body=[u8]),
        (status=403, description="JWT Not Found In Cookie", body=[u8]),
        (status=406, description="No Time Hash Found In Cookie", body=[u8]),
        (status=406, description="Invalid Cookie Format", body=[u8]),
        (status=403, description="Cookie Has Been Expired", body=[u8]),
        (status=406, description="Invalid Cookie Time Hash", body=[u8]),
        (status=403, description="Access Denied", body=i32),
        (status=406, description="No Expiration Time Found In Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("account_name" = String, Path, description = "twitter account")
    ),
    tag = "crate::apis::user",
    security(
        ("jwt" = [])
    )
)]
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* rate limiter based on doer id */
                    let chill_zone_duration = 30_000u64; //// 30 seconds chillzone
                    let now = chrono::Local::now().timestamp_millis() as u64;
                    let mut is_rate_limited = false;
                    
                    let redis_result_verify_username_rate_limiter: RedisResult<String> = redis_conn.get("verify_username_rate_limiter").await;
                    let mut redis_verify_username_rate_limiter = match redis_result_verify_username_rate_limiter{
                        Ok(data) => {
                            let rl_data = serde_json::from_str::<HashMap<u64, u64>>(data.as_str()).unwrap();
                            rl_data
                        },
                        Err(e) => {
                            let empty_verify_username_rate_limiter = HashMap::<u64, u64>::new();
                            let rl_data = serde_json::to_string(&empty_verify_username_rate_limiter).unwrap();
                            let _: () = redis_conn.set("verify_username_rate_limiter", rl_data).await.unwrap();
                            HashMap::new()
                        }
                    };

                    if let Some(last_used) = redis_verify_username_rate_limiter.get(&(_id as u64)){
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
                        redis_verify_username_rate_limiter.insert(_id as u64, now); //// updating the redis rate limiter map
                        let rl_data = serde_json::to_string(&redis_verify_username_rate_limiter).unwrap();
                        let _: () = redis_conn.set("verify_username_rate_limiter", rl_data).await.unwrap(); //// writing to redis ram


                        /* we can pass usernmae by reference or its slice form instead of cloning it */
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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=200, description="Fetched Successfully", body=[u8]),
        (status=404, description="User Not Found", body=i32), // not found by id
        (status=404, description="Task Not Found", body=i32), // not found by id
        (status=404, description="No Value Found In Cookie Or JWT In Header", body=[u8]),
        (status=403, description="JWT Not Found In Cookie", body=[u8]),
        (status=406, description="No Time Hash Found In Cookie", body=[u8]),
        (status=406, description="Invalid Cookie Format", body=[u8]),
        (status=403, description="Cookie Has Been Expired", body=[u8]),
        (status=406, description="Invalid Cookie Time Hash", body=[u8]),
        (status=403, description="Access Denied", body=i32),
        (status=406, description="No Expiration Time Found In Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("user_id" = i32, Path, description = "user id"),
    ),
    tag = "crate::apis::user",
    security(
        ("jwt" = [])
    )
)]
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
            match User::passport(req, granted_role, connection).await{
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
            match User::passport(req, granted_role, connection).await{
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
                        
                        /* ----------------------------
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
                        let token_price = calculate_token_value(charge_wallet_request_object.tokens, redis_client.clone()).await;

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
                                    using stripe api.
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

#[utoipa::path(
    context_path = "/user",
    request_body = NewIdRequest,
    responses(
        (status=201, description="Built Successfully", body=UserIdResponse),
        (status=500, description="Internal Server Erros  Caused By Diesel or Redis", body=&[u8]),
        (status=429, description="Rate Limited, Chill 30 Seconds", body=&[u8]),
    ),
    tag = "crate::apis::user",
    security(
        ("jwt" = [])
    )
)]
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
            match User::passport(req, granted_role, connection).await{
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

                    /* if the phone wasn't verified user can't create id */
                    if user.phone_number.is_none() || 
                        !user.is_phone_verified{
                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            NOT_VERIFIED_PHONE, // response message
                            StatusCode::NOT_ACCEPTABLE, // status code
                            None::<Cookie<'_>>, // cookie
                        }
                    }

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
                        let save_user_data = new_id.save(connection).await;
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

#[utoipa::path(
    context_path = "/user",
    request_body = NewUserDepositRequest,
    responses(
        (status=201, description="Deposited Successfully", body=UserDepositData),
        (status=429, description="Rate Limited, Chill 30 Seconds", body=&[u8]),
        (status=406, description="Not Acceptable Errors (Invalid Signatures, CID, Data and ...)", body=&[u8]),
        (status=500, description="Internal Server Erros  Caused By Diesel or Redis", body=&[u8]),
    ),
    tag = "crate::apis::user",
    security(
        ("jwt" = [])
    )
)]
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
            match User::passport(req, granted_role, connection).await{
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

                        /* ----------------------------
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
                        
                        /* 
                            simd ops on u256 bits can be represented as an slice with 4 elements 
                            each of type 64 bits or 8 bytes, also 256 bits is 64 chars in hex 
                            and 32 bytes of utf8 and  rust doesn't have u256
                        */
                        let u256 = web3::types::U256::from_str("0").unwrap().0;

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

                                    let update_user_balance = User::update_balance(user.id, new_balance, connection).await;
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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=201, description="Fetched Successfully", body=Vec<UserDepositData>),
        (status=500, description="Internal Server Erros  Caused By Diesel or Redis", body=&[u8]),
    ),
    params(
        ("cid" = String, Path, description = "user cid"),
    ),
    tag = "crate::apis::user",
    security(
        ("jwt" = [])
    )
)]
#[get("/deposit/get/user/{cid}/")]
#[passport(user)]
async fn get_all_user_deposits(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    user_cid: web::Path<String>
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match UserDeposit::get_all_for(user_cid.to_string(), limit, connection).await{
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

#[utoipa::path(
    context_path = "/user",
    request_body = NewUserWithdrawRequest,
    responses(
        (status=201, description="Withdrawn Successfully", body=UserWithdrawalData),
        (status=429, description="Rate Limited, Chill 30 Seconds", body=&[u8]),
        (status=406, description="Not Acceptable Errors (Invalid Signatures, CID, Data and ...)", body=&[u8]),
        (status=404, description="Deposit Object Not Found", body=i32),
        (status=500, description="Internal Server Erros  Caused By Diesel or Redis", body=&[u8]),
        (status=302, description="Already Withdrawn", body=&[u8]),
    ),
    tag = "crate::apis::user",
    security(
        ("jwt" = [])
    )
)]
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
            match User::passport(req, granted_role, connection).await{
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
                    
                        /* ----------------------------
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
                        let polygon_recipient_address = Wallet::generate_keccak256_from(withdraw_object.recipient_cid.to_owned().clone());
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
                                    let update_user_balance = User::update_balance(user.id, new_balance, connection).await;
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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=201, description="Fetched Successfully", body=Vec<UserWithdrawalData>),
        (status=500, description="Internal Server Erros  Caused By Diesel or Redis", body=&[u8]),
    ),
    params(
        ("cid" = String, Path, description = "user cid"),
    ),
    tag = "crate::apis::user",
    security(
        ("jwt" = [])
    )
)]
#[get("/withdraw/get/user/{cid}/")]
#[passport(user)]
async fn get_all_user_withdrawals(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    user_cid: web::Path<String>
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
    
                    match UserWithdrawal::get_all_for(user_cid.to_string(), limit, connection).await{
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

#[get("/checkout/get/unpaid/user/{cid}/")]
#[passport(user)]
async fn get_all_user_unpaid_checkouts(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    user_cid: web::Path<String>
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
    
                    match UserCheckout::get_all_unpaid_for(&user_cid, limit, connection).await{
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

#[get("/checkout/get/paid/user/{cid}/")]
#[passport(user)]
async fn get_all_user_paid_checkouts(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    user_cid: web::Path<String>
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
    
                    match UserCheckout::get_all_paid_for(&user_cid, limit, connection).await{
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

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=201, description="Fetched Successfully", body=Vec<UserWithdrawalData>),
        (status=500, description="Internal Server Erros  Caused By Diesel or Redis", body=&[u8]),
    ),
    params(
        ("cid" = String, Path, description = "user cid"),
    ),
    tag = "crate::apis::user",
    security(
        ("jwt" = [])
    )
)]
#[get("/deposit/get/unclaimed/recipient/{recipient_cid}/")]
#[passport(user)]
async fn get_recipient_unclaimed_deposits(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    recipient_cid: web::Path<String>
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* generate keccak256 from recipient_cid to mint nft to */
                    let polygon_recipient_address = Wallet::generate_keccak256_from(recipient_cid.to_owned().clone());

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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let new_bio = update_bio_request.to_owned().bio;

                    match User::update_bio(_id, &new_bio, connection).await{
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
                match User::passport(req, granted_role, connection).await{
                    Ok(token_data) => {
                        
                        let _id = token_data._id;
                        let role = token_data.user_role;
    
                        
                        match User::update_wallet_back(_id, img, connection).await{
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    
                    match User::update_avatar(_id, img, connection).await{
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match User::update_banner(_id, img, connection).await{
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

/* this api must gets called by player with his JWT passed in to the request header */
#[post("/mafia/player/{player_id}/upload/avatar")]
async fn update_mafia_player_avatar(
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
    
                note that this token must be taken from the conse mafia hyper server
            */
            match mafia_passport!{ token }{
                true => {
    
                    // -------------------------------------------------------------------------------------
                    // ------------------------------- ACCESS GRANTED REGION -------------------------------
                    // -------------------------------------------------------------------------------------
                    /*  
                        this route requires the player access token from the conse 
                        mafia hyper server to update avatar image, we'll send a request
                        to the conse mafia hyper server to verify the passed in JWT of the
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

                    /* creating the asset folder if it doesn't exist */
                    tokio::fs::create_dir_all(AVATAR_UPLOAD_PATH).await.unwrap();
                    let mut player_img_filepath = "".to_string();
                    
                    /*  
                        streaming over incoming img multipart form data to extract the
                        field object for writing the bytes into the file
                    */
                    while let Ok(Some(mut field)) = img.try_next().await{

                        /* getting the content_disposition header which contains the filename */
                        let content_type = field.content_disposition();

                        /* creating the filename and the filepath */
                        let filename = content_type.get_filename().unwrap().to_lowercase();
                        let ext_position_png = filename.find("png");
                        let ext_position_jpg = filename.find("jpg");
                        let ext_position_jpeg = filename.find("jpeg");

                        let ext_position = if filename.find("png").is_some(){
                            ext_position_png.unwrap()
                        } else if filename.find("jpg").is_some(){
                            ext_position_jpg.unwrap()
                        } else if filename.find("jpeg").is_some(){
                            ext_position_jpeg.unwrap()
                        } else{

                            resp!{
                                &[u8], // the date type
                                &[], // the data itself
                                UNSUPPORTED_IMAGE_TYPE, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            } 
                        };

                        let player_img_filename = format!("player:{}-img:{}.{}", player_id, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(), &filename[ext_position..]);
                        let filepath = format!("{}/{}", EVENT_UPLOAD_PATH, sanitize_filename::sanitize(&player_img_filename));
                        player_img_filepath = filepath.clone();

                        /* 
                            receiving asyncly by streaming over the field future io object,
                            getting the some part of the next field future object to extract 
                            the image bytes from it
                        */
                        let mut file_buffer = vec![];
                        while let Some(chunk) = field.next().await{
                            
                            /* chunk is a Bytes object that can be used to be written into a buffer */
                            let data = chunk.unwrap();

                            /* getting the size of the file */
                            file_buffer.extend_from_slice(&data);
                            
                        }

                        /* if the file size was greater than 1 MB reject the request */
                        if file_buffer.len() > env::var("IMG_FILE_SIZE").unwrap().parse::<usize>().unwrap(){

                            /* terminate the method and respond the caller */
                            let resp = Response::<&[u8]>{
                                data: Some(&[]),
                                message: TOO_LARGE_FILE_SIZE,
                                status: 406
                            };
                            return Ok(HttpResponse::NotAcceptable().json(resp));
                        }

                        /* 
                            web::block() executes a blocking function on a actix threadpool
                            using spawn_blocking method of actix runtime so in here we're 
                            creating a file inside a actix runtime threadpool to fill it with 
                            the incoming bytes inside the field object by streaming over field
                            object to extract the bytes
                        */
                        let mut f = web::block(|| std::fs::File::create(filepath).unwrap()).await.unwrap();

                        /* writing bytes into the created file with the extracted filepath */
                        f = web::block(move || f.write_all(&file_buffer).map(|_| f))
                            .await
                            .unwrap()
                            .unwrap();

                    }

                    /* 
                        writing the avatar image filename to redis ram, by doing this we can 
                        retrieve the value from redis in conse hyper mafia server when we call 
                        the check token api
                    */
                    let _: () = redis_conn.set(player_id_img_key.as_str(), player_img_filepath.as_str()).await.unwrap();
                
                    resp!{
                        &[u8], // the date type
                        &[], // the data itself
                        EVENT_IMG_UPDATED, // response message
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
            match User::passport(req, granted_role, connection).await{
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
                    
                        
                        let create_private_gallery_request_object = new_private_gallery_request.to_owned();
                        
                        /* ----------------------------
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

#[post("/gallery/update/{gal_id}")]
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
            match User::passport(req, granted_role, connection).await{
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
                    
                        let update_private_gallery_request_object = update_private_gallery_request.to_owned();
                        
                        /* ----------------------------
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

                        match UserPrivateGallery::update(&Wallet::generate_keccak256_from(
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let remove_invited_friend_request = remove_invited_friend_request.to_owned();
                    
                    /* ----------------------------
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

                    match UserPrivateGallery::remove_invited_friend_from(remove_invited_friend_request, connection).await{
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let send_invitation_request = send_invitation_request.to_owned();
                    
                    /* ----------------------------
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

#[post("/gallery/get/all/for/{who}/")]
#[passport(user)]
async fn get_all_private_galleries_for(
    req: HttpRequest,
    who: web::Path<String>,
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match UserPrivateGallery::get_all_for(&Wallet::generate_keccak256_from(who.to_owned()), limit, connection).await{
                        
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

#[post("/gallery/get/all/for/{who}/caller/{caller}/")]
#[passport(user)]
async fn get_all_galleries_invited_to(
    req: HttpRequest,
    who_and_caller: web::Path<(String, String)>,
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let (who, caller) = who_and_caller.to_owned();
                    match UserPrivateGallery::get_all_galleries_invited_to(
                        &Wallet::generate_keccak256_from(caller),
                        &Wallet::generate_keccak256_from(who), 
                        limit, connection).await{
                        
                        Ok(galleries) => {

                            resp!{
                                Vec<Option<UserPrivateGalleryData>>, //// the data type
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

#[post("/gallery/get/invited-friends/of/{gal_id}/caller/{caller}/")]
#[passport(user)]
async fn get_invited_friends_wallet_data_of_gallery(
    req: HttpRequest,
    gal_id_and_caller: web::Path<(i32, String)>,
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let (gal_id, caller) = gal_id_and_caller.to_owned();
                    match UserPrivateGallery::get_invited_friends_wallet_data_of_gallery(
                        &Wallet::generate_keccak256_from(caller),
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

#[post("/collection/get/all/for/{who}/")]
#[passport(user)]
async fn get_all_public_collections_for(
    req: HttpRequest,
    who: web::Path<String>,
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match UserCollection::get_all_public_collections_for(
                        &Wallet::generate_keccak256_from(who.to_owned()), 
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

pub mod exports{
    // -----------------------------------------------
    /*         verifications and report apis         */
    // -----------------------------------------------
    pub use super::login;
    pub use super::login_with_identifier_and_password;
    pub use super::verify_twitter_account;
    pub use super::edit_bio;
    pub use super::upload_avatar;
    pub use super::upload_banner;
    pub use super::upload_wallet_back;
    pub use super::update_mafia_player_avatar;
    pub use super::make_cid;
    pub use super::request_mail_code;
    pub use super::verify_mail_code;
    pub use super::request_phone_code;
    pub use super::verify_phone_code;
    pub use super::tasks_report;
    pub use super::get_all_user_withdrawals;
    pub use super::get_all_user_deposits;
    pub use super::get_recipient_unclaimed_deposits;
    pub use super::get_all_user_unpaid_checkouts;
    pub use super::get_all_user_paid_checkouts;
    // -----------------------------------------------
    /*                  gallery apis                 */
    // -----------------------------------------------
    pub use super::create_private_gallery;
    pub use super::update_private_gallery;
    pub use super::get_all_private_galleries_for;
    pub use super::get_all_galleries_invited_to;
    pub use super::get_all_public_collections_for;
    pub use super::get_invited_friends_wallet_data_of_gallery;
    pub use super::send_private_gallery_invitation_request_to;
    pub use super::remove_invited_friend_from_gallery;
    // pub use super::add_user_to_friend;
    // pub use super::remove_user_from_friend;
    // pub use super::send_friend_request_to;
    // pub use super::get_user_unaccpeted_invitation_requests;
    // pub use super::accept_invitation_request;
    // -----------------------------------------------
    /*             chatroom launchpad apis           */
    // -----------------------------------------------
    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
    // https://docs.nftport.xyz/reference/deploy-nft-collection-contract
    // https://docs.nftport.xyz/reference/deploy-nft-product-contract
    // https://docs.nftport.xyz/reference/transfer-minted-nft
    // https://docs.nftport.xyz/reference/customizable-minting
    // ...
    // -----------------------------------------------
    /*             collection advieh apis            */
    // -----------------------------------------------
    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
    // ...
    // -----------------------------------------------
    /*              polygon marketplace apis         */
    // -----------------------------------------------
    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
    // pub use super::create_collection;
    // pub use super::update_collection;
    // pub use super::create_nft; /* add img Multipart data for img url */
    // pub use super::mint_nft;
    // pub use super::sell_nft; 
    // pub use super::buy_nft;
    // pub use super::transfer_nft;
    // pub use super::update_nft;
    // pub use super::add_nft_comment;
    // pub use super::like_nft;
    // pub use super::dislike_nft;
    // -----------------------------------------------
    /*        deposit/withdraw in-app token apis     */
    // -----------------------------------------------
    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
    // pub use super::sell_token; /* update account_number and paypal_id fields */
    pub use super::deposit; /* gift card money transfer */
    pub use super::withdraw; /* gift card money claim */
    pub use super::charge_wallet_request;
}