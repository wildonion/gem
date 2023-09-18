


use crate::*;
use crate::models::users_deposits::{UserDepositData, DecodedSignedDepositData};
use crate::models::users_withdrawals::{UserWithdrawal, UserWithdrawalData, DecodedSignedWithdrawalData};
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::misc::s3::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use futures_util::TryStreamExt;
use secp256k1::ecdsa::Signature; /* TryStreamExt can be used to call try_next() on future object */
use crate::*;
use crate::models::users::UserRole;
use crate::constants::*;
use crate::misc::*;
use crate::misc::s3::*;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use models::users::{Id, NewIdRequest, UserIdResponse};
use models::users_deposits::{NewUserDepositRequest, UserDeposit};
use models::users_withdrawals::NewUserWithdrawRequest;
use wallexerr::Wallet;




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
        deposit,
        withdraw,
        get_all_user_withdrawals,
        get_all_user_deposits,
        get_recipient_unclaimed_deposits,
        request_mail_code,
        verify_mail_code,
        request_phone_code,
        verify_phone_code,
        charge_wallet
    ),
    components(
        schemas(
            UserData,
            FetchUserTaskReport,
            UserLoginInfoRequest,
            TaskData,
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

                    /* checking that the incoming request is already rate limited or not */
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
        ("mail" = String, Path, description = "user mail")
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

                    /* checking that the incoming request is already rate limited or not */
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
                        
                      
                        match User::send_phone_verification_code_to(_id, user_phone.to_owned(), connection).await{
                            
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
                    
                    let redis_result_rate_limiter: RedisResult<String> = redis_conn.get("rate_limiter").await;
                    let mut redis_rate_limiter = match redis_result_rate_limiter{
                        Ok(data) => {
                            let rl_data = serde_json::from_str::<HashMap<u64, u64>>(data.as_str()).unwrap();
                            rl_data
                        },
                        Err(e) => {
                            let empty_rate_limiter = HashMap::<u64, u64>::new();
                            let rl_data = serde_json::to_string(&empty_rate_limiter).unwrap();
                            let _: () = redis_conn.set("rate_limiter", rl_data).await.unwrap();
                            HashMap::new()
                        }
                    };

                    if let Some(last_used) = redis_rate_limiter.get(&(_id as u64)){
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
                        redis_rate_limiter.insert(_id as u64, now); //// updating the redis rate limiter map
                        let rl_data = serde_json::to_string(&redis_rate_limiter).unwrap();
                        let _: () = redis_conn.set("rate_limiter", rl_data).await.unwrap(); //// writing to redis ram


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
#[get("/report-tasks/{user_id}")]
#[passport(user)]
pub async fn tasks_report(
        req: HttpRequest,
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


                    match UserTask::reports(user_id.to_owned(), connection).await{
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

#[utoipa::path(
    context_path = "/user",
    request_body = ChargeWalletRequest,
    responses(
        (status=201, description="Paid Successfully", body=UserData),
        (status=500, description="Internal Server Erros Caused By Diesel or Redis", body=&[u8]),
    ),
    tag = "crate::apis::user",
    security(
        ("jwt" = [])
    )
)]

#[get("/cid/wallet/charge")]
#[passport(user)]
async fn charge_wallet(
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

                    let charge_wallet_request_object = charge_wallet_request.to_owned();

                    let get_user = User::find_by_id(_id, connection).await;
                        let Ok(user) = get_user else{
                            let error_resp = get_user.unwrap_err();
                            return error_resp;
                    };

                    let get_secp256k1_pubkey = PublicKey::from_str(&charge_wallet_request.buyer_cid);
                    let get_secp256k1_signature = Signature::from_str(&charge_wallet_request.tx_signature);
                    
                    let Ok(secp256k1_pubkey) = get_secp256k1_pubkey else{

                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            INVALID_CID, // response message
                            StatusCode::NOT_ACCEPTABLE, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    let Ok(secp256k1_signature) = get_secp256k1_signature else{

                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            SIGNATURE_ENCODE_ISSUE, // response message
                            StatusCode::NOT_ACCEPTABLE, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    let strigified_deposit_data = serde_json::json!({
                        "user_id": charge_wallet_request.user_id,
                        "buyer_cid": charge_wallet_request.buyer_cid,
                        "tokens": charge_wallet_request.tokens
                    });

                    /* verifying the data against the generated signature */
                    let get_verification = Wallet::verify_secp256k1_signature_from_pubkey_str(
                        strigified_deposit_data.to_string().as_str(),
                        &charge_wallet_request.tx_signature, 
                        &charge_wallet_request.buyer_cid
                    );


                    let Ok(_) = get_verification else{

                        let verification_error = get_verification.unwrap_err();
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            &verification_error.to_string(), // response message
                            StatusCode::NOT_ACCEPTABLE, // status code
                            None::<Cookie<'_>>, // cookie
                        }

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

                    let u_region = user.region.unwrap();

                    let token_price = calculate_token_value(charge_wallet_request_object.tokens).await;
                    let usd_token_price = token_price.0;
                    let irr_token_price = token_price.1;

                    let gateway_resp = match u_region.as_str(){
                        "ir" => {

                            if user.account_number.is_some() && 
                                !user.account_number.unwrap().is_empty(){

                                    /* this is the equivalent amount of token price that must be paid in ir */
                                    let final_amount_to_be_paid = irr_token_price as f64 / 1000000.0; /* converting the ir price back to float */
                                    
                                    // 🚪 ir gateway
                                    // ...

                                    200 

                            } else{

                                resp!{
                                    i32, // the data type
                                    _id, // response data
                                    INVALID_ACCOUNT_NUMBER, // response message
                                    StatusCode::NOT_ACCEPTABLE, // status code
                                    None::<Cookie<'_>>, // cookie
                                }

                            }

                        },
                        _ => {

                            if user.paypal_id.is_some() && 
                                !user.paypal_id.unwrap().is_empty(){

                                    /* this is the equivalent amount of token price that must be paid in none ir */
                                    let final_amount_to_be_paid = usd_token_price as f64 / 10000.0;  /* converting the usd price back to float */

                                    // 🚪 paypal_id gateway to charge user with current_token_price amount
                                    // ...

                                    200 

                            } else{

                                resp!{
                                    i32, // the data type
                                    _id, // response data
                                    INVALID_PAYPAL_ID, // response message
                                    StatusCode::NOT_ACCEPTABLE, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            }

                        }
                    };
                    

                    if gateway_resp == 200 || gateway_resp == 201{

                        let new_balance = user.balance.unwrap() + charge_wallet_request.tokens;
                        match User::update_balance(_id, new_balance, connection).await{

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


                    } else{

                        resp!{
                            i32, // the data type
                            _id, // response data
                            CANT_CHARGE_WALLET, // response message
                            StatusCode::EXPECTATION_FAILED, // status code
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

                    /* checking that the incoming request is already rate limited or not */
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
#[post("/deposit")]
#[passport(user)]
async fn deposit(
    req: HttpRequest,
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

                    /* checking that the incoming request is already rate limited or not */
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

                        /* making sure that the user has a full filled paypal id */
                        let get_user = User::find_by_id(_id, connection).await;
                        let Ok(user) = get_user else{
                            let error_resp = get_user.unwrap_err();
                            return error_resp;
                        };

                        /* if the phone wasn't verified user can't deposit */
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

                        let deposit_object = deposit.to_owned();

                        let find_user_screen_cid = User::find_by_username(&deposit_object.recipient, connection).await;
                        let Ok(recipient_info) = find_user_screen_cid else{
                            
                            resp!{
                                String, // the data type
                                deposit_object.recipient, // response data
                                &RECIPIENT_NOT_FOUND, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

                        let get_secp256k1_pubkey = PublicKey::from_str(&deposit_object.from_cid);
                        let get_secp256k1_signature = Signature::from_str(&deposit_object.tx_signature);
                        
                        let Ok(secp256k1_pubkey) = get_secp256k1_pubkey else{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_CID, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        let Ok(secp256k1_signature) = get_secp256k1_signature else{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                SIGNATURE_ENCODE_ISSUE, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        let strigified_deposit_data = serde_json::json!({
                            "recipient": deposit_object.recipient,
                            "from_cid": deposit_object.from_cid,
                            "amount": deposit_object.amount
                        });

                        /* verifying the data against the generated signature */
                        let get_verification = Wallet::verify_secp256k1_signature_from_pubkey_str(
                            strigified_deposit_data.to_string().as_str(),
                            &deposit_object.tx_signature, 
                            &deposit_object.from_cid
                        );


                        let Ok(_) = get_verification else{

                            let verification_error = get_verification.unwrap_err();
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &verification_error.to_string(), // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        /* 

                            note that when a user wants to deposit, frontend must call the get token price api 
                            to get the latest and exact equivalent token of the gift card price to charge the 
                            user for paying that price which is the deposit_object.amount field in deposit 
                            request body also if a user want to claim the card he gets paid by sending the exact
                            token that depositor has paid for to his wallet
                        
                        */
                        if user.balance.is_some() && 
                            user.balance.unwrap() > 0 && 
                            user.balance.unwrap() > deposit_object.amount{

                            let new_balance = user.balance.unwrap() - deposit_object.amount;

                            let (mint_tx_hash_sender, mut mint_tx_hash_receiver) = tokio::sync::mpsc::channel::<(String, String)>(1024);
                            let mut mint_tx_hash = String::from("");
                            let mut token_id = String::from("");
                            
                            /* 
                                simd ops on u256 bits can be represented as an slice with 4 elements 
                                each of type 64 bits or 8 bytes, also 256 bits is 64 chars in hex 
                                and 32 bytes of utf8
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
                            let polygon_recipient_address = recipient_info.screen_cid.unwrap();
                            let cloned_polygon_recipient_address = polygon_recipient_address.clone();
                            
                            /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
                            /* calling the thirdweb minting api to mint inside tokio green threadpool */
                            /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
                            tokio::task::spawn(async move{

                                
                                let host = std::env::var("THIRDWEB_HOST").expect("⚠️ no thirdweb host variable set");
                                let port = std::env::var("THIRDWEB_PORT").expect("⚠️ no thirdweb port variable set").parse::<u16>().unwrap();
                                let api_path = format!("http://{}:{}/mint/to/{}/{}", host, port, cloned_polygon_recipient_address.clone(), deposit_object.amount.to_string());
                                let client = reqwest::Client::new();
                                let res = client
                                    .post(api_path.as_str())
                                    .send()
                                    .await
                                    .unwrap();
                                
                                let mint_response = res.json::<ThirdwebMintResponse>().await.unwrap();
                                
                                /* calling thirdweb python server to mint nft */
                                let token_id_string = String::from(mint_response.token_id);
                                let mint_tx_hash = mint_response.mint_tx_hash;
                                
                                if mint_tx_hash.starts_with("0x"){
                                    mint_tx_hash_sender.send((mint_tx_hash, token_id_string)).await;
                                }


                            });

                            /* receiving asyncly from the channel */
                            while let Some((tx_hash, tid)) = mint_tx_hash_receiver.recv().await{
                                mint_tx_hash = tx_hash;
                                token_id = tid;
                            }
                            
                            if !mint_tx_hash.is_empty(){
                                
                                match UserDeposit::insert(deposit.to_owned(), mint_tx_hash, token_id, polygon_recipient_address, connection).await{
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
                            
                        } else{
                            resp!{
                                &[u8], // the date type
                                &[], // the data itself
                                INSUFFICIENT_FUNDS, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
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
#[get("/deposit/get/user/{cid}")]
#[passport(user)]
async fn get_all_user_deposits(
    req: HttpRequest,
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

                    match UserDeposit::get_all_for(user_cid.to_string(), connection).await{
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
#[post("/withdraw")]
#[passport(user)]
async fn withdraw(
    req: HttpRequest,
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

                    /* checking that the incoming request is already rate limited or not */
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

                        let get_user = User::find_by_id(_id, connection).await;
                        let Ok(user) = get_user else{
                            let error_resp = get_user.unwrap_err();
                            return error_resp;
                        };

                        /* if the phone wasn't verified user can't deposit */
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
                        
                        let withdraw_object = withdraw.to_owned();

                        /* recipient_cid will be used to verify the signature only cause only the receiver can withdraw the deposited amount */
                        let get_secp256k1_pubkey = PublicKey::from_str(&withdraw_object.recipient_cid);
                        let get_secp256k1_signature = Signature::from_str(&withdraw_object.tx_signature);
                        
                        let Ok(secp256k1_pubkey) = get_secp256k1_pubkey else{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_CID, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        let Ok(secp256k1_signature) = get_secp256k1_signature else{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                SIGNATURE_ENCODE_ISSUE, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        let strigified_withdraw_data = serde_json::json!({
                            "recipient_cid": withdraw_object.recipient_cid,
                            "deposit_id": withdraw_object.deposit_id,
                        });

                        /* verifying the data against the generated signature */
                        let get_verification = Wallet::verify_secp256k1_signature_from_pubkey_str(
                            strigified_withdraw_data.to_string().as_str(),
                            &withdraw_object.tx_signature, 
                            &withdraw_object.recipient_cid
                        );

                        let Ok(_) = get_verification else{

                            let verification_error = get_verification.unwrap_err();
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &verification_error.to_string(), // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        let get_deposit_info = UserDeposit::find_by_id(withdraw_object.deposit_id, connection).await;
                        let Ok(deposit_info) = get_deposit_info else{

                            let error = get_deposit_info.unwrap_err();
                            return error;
                        };

                        /* generate keccak256 from recipient_cid to check aginst the one in db */
                        let polygon_recipient_address = Wallet::generate_keccak256_from(withdraw_object.recipient_cid.to_owned().clone());
                        if deposit_info.recipient_screen_cid != polygon_recipient_address{
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                NO_DEPOSIT_FOR_THIS_RECIPIENT, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        let (burn_tx_hash_sender, mut burn_tx_hash_receiver) = tokio::sync::mpsc::channel::<String>(1024);
                        let mut burn_tx_hash = String::from("");
                        let token_id = deposit_info.nft_id;
                        
                        /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
                        /* calling the thirdweb burning api to burn inside tokio green threadpool */
                        /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
                        tokio::task::spawn(async move{

                            let host = std::env::var("THIRDWEB_HOST").expect("⚠️ no thirdweb host variable set");
                            let port = std::env::var("THIRDWEB_PORT").expect("⚠️ no thirdweb port variable set").parse::<u16>().unwrap();
                            let api_path = format!("http://{}:{}/burn/{}", host, port, token_id);
                            let client = reqwest::Client::new();
                            let res = client
                                .post(api_path.as_str())
                                .send()
                                .await
                                .unwrap();
                            
                            let burn_response = res.json::<ThirdwebBurnResponse>().await.unwrap();

                            let burn_tx_hash = burn_response.burn_tx_hash;
                            if burn_tx_hash.starts_with("0x"){
                                burn_tx_hash_sender.send(burn_tx_hash).await;
                            }

                        });

                        /* receiving asyncly from the channel */
                        while let Some(tx_hash) = burn_tx_hash_receiver.recv().await{
                            burn_tx_hash = tx_hash;
                        }

                        if !burn_tx_hash.is_empty(){

                            match UserWithdrawal::insert(withdraw.to_owned(), burn_tx_hash, connection).await{
                                Ok(user_withdrawal_data) => {
                                    
                                    let update_user_balance = User::update_balance(user.id, deposit_info.amount, connection).await;
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
                                CANT_BURN_CARD, // response message
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
#[get("/withdraw/get/user/{cid}")]
#[passport(user)]
async fn get_all_user_withdrawals(
    req: HttpRequest,
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
    
                    match UserWithdrawal::get_all_for(user_cid.to_string(), connection).await{
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
#[get("/deposit/get/unclaimed/recipient/{recipient_cid}")]
#[passport(user)]
async fn get_recipient_unclaimed_deposits(
    req: HttpRequest,
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

                    match UserDeposit::get_unclaimeds_for(polygon_recipient_address, connection).await{
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


pub mod exports{
    pub use super::login;
    pub use super::login_with_identifier_and_password;
    pub use super::verify_twitter_account;
    pub use super::tasks_report;
    pub use super::make_cid;
    pub use super::get_all_user_withdrawals;
    pub use super::get_all_user_deposits;
    pub use super::get_recipient_unclaimed_deposits;
    pub use super::request_mail_code;
    pub use super::verify_mail_code;
    pub use super::request_phone_code;
    pub use super::verify_phone_code;
    // https://docs.opensea.io/reference/api-overview
    /*
    pub use super::verify_social_id; // update the social_id field
    pub use super::verify_account_number; // update the account_number field
    pub use super::verify_paypal_id; // update the paypal_id field
    pub use super::add_post_comment;
    pub use super::like_post;
    pub use super::add_nft_comment;
    pub use super::like_nft;
    pub use super::get_user_nfts;
    */
    /* ---------------------------------------------------- 
        user must pay token in following calls and 
        backend pay the gas fee with matic through 
        thirdweb calls also followings need CID signature 
        and user must sign the calls
    ------------------------------------------------------- */
    pub use super::deposit;
    pub use super::withdraw;
    pub use super::charge_wallet;
    /*
    pub use super::give_stars_to;
    pub use super::send_invitation_link;
    pub use super::get_private_room_nfts_info_of; // fetch private room info and nfts of a user, only user can see it
    pub use super::mint_nft; // goes to private room
    pub use super::add_offer_to; 
    pub use super::accept_offer; // pay with token
    pub use super::reject_offer; // pay back the token
    pub use super::start_auction_on; 
    pub use super::add_bid_on;
    pub use super::sell_nft; 
    pub use super::add_user_to_friend;
    pub use super::remove_user_from_friend;
    pub use super::advertise_collection;
    pub use super::get_public_room_nfts_info_of; // fetch public room info and nfts of a user, only friends can see it
    */
}