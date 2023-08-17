


use crate::*;
use crate::models::users_deposits::{UserDepositData, DecodedSignedDepositData};
use crate::models::users_withdrawals::{UserWithdrawal, UserWithdrawalData, DecodedSignedWithdrawalData};
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use futures_util::TryStreamExt; /* TryStreamExt can be used to call try_next() on future object */
use crate::*;
use crate::models::users::UserRole;
use crate::constants::*;
use crate::misc::*;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use models::users::{Id, NewIdRequest, UserIdResponse};
use models::users_deposits::{NewUserDepositRequest, UserDeposit};
use models::users_withdrawals::NewUserWithdrawRequest;





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
    ),
    components(
        schemas(
            UserData,
            FetchUserTaskReport,
            UserLoginInfoRequest,
            TaskData
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
                        last_login: { 
                            if updated_user.last_login.is_some(){
                                Some(updated_user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: user.created_at.to_string(),
                        updated_at: updated_user.updated_at.to_string(),
                        gmail: user.gmail,
                        phone_number: user.phone_number,
                        paypal_id: user.paypal_id,
                        account_number: user.account_number,
                        device_id: user.device_id,
                        social_id: user.social_id,
                        cid: user.cid,
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
                        last_login: { 
                            if updated_user.last_login.is_some(){
                                Some(updated_user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: user.created_at.to_string(),
                        updated_at: updated_user.updated_at.to_string(),
                        gmail: user.gmail,
                        phone_number: user.phone_number,
                        paypal_id: user.paypal_id,
                        account_number: user.account_number,
                        device_id: user.device_id,
                        social_id: user.social_id,
                        cid: user.cid,
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
async fn make_id(
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

                    let identifier_key = format!("{}", _id);

                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)));
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
                            new_object_id_request.username.clone(),
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


                        /* ------------------ test retrieving and signing ------------------ */
                        /* building the keypair from the public and private keys */
                        let retrieve_keypair = Id::retrieve_keypair(
                            &new_id.new_cid.as_ref().unwrap(),
                            &new_id.signer.as_ref().unwrap()
                        );


                        /* signing the data using the private key */
                        let signed_id_hex_string = new_id.test_sign();
                        /* ----------------------------------------------------------------- */


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

                    let identifier_key = format!("{}", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)));
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

                        let deposit_object = deposit.to_owned();

                        /* 
                            here is the process of verifying the signature of signed data 
                            using the private key inside js using themis wasm.
                            we've cloned the tx_signature and hex_pubkey since the PartialEq
                            trait takes the ownership of the WithdrawRequest instacen
                            when we want to compare the decoded data with the actual instance
                            inside the request body
                        */
                        let hex_tx_signature = deposit_object.tx_signature.clone();
                        let hex_pubkey = deposit_object.from_cid.clone();

                        /* dropping the first 2 bytes or 0x from the hex string */
                        let tx_signature = &hex_tx_signature[2..];
                        let pubkey = &hex_pubkey[2..];
                        
                        let decode_pubkey = hex::decode(pubkey.to_string());
                        let decode_signature = hex::decode(tx_signature.to_string());

                        let Ok(pubkey_bytes) = decode_pubkey else{
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_CID, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };


                        let Ok(signature) = decode_signature else{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                SIGNATURE_DECODE_ISSUE, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        /* verifying the data against the generated signature */
                        let get_encoded_data = Id::verify(
                            &signature, 
                            &pubkey_bytes
                        );

                        let Ok(encoded_data) = get_encoded_data else{
                            
                            /* verifying signature or building pubkey error */
                            let error = get_encoded_data.unwrap_err();
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_SIGNATUE_OR_CID, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

                        /* decoding the signed message bytes */
                        let pure_data = serde_json::from_slice::<DecodedSignedDepositData>(&encoded_data).unwrap();

                        /* the decoded data must be equals to the request body */
                        let request_body_data = DecodedSignedDepositData{
                            from_cid: deposit_object.from_cid,
                            recipient_cid: deposit_object.recipient_cid,
                            amount: deposit_object.amount,
                        };

                        if pure_data == request_body_data.clone(){

                            /* 

                                if we have mint_tx_signature means that the whole payment 
                                process is done successfully, minter paid IR exchange and 
                                the exchange charged the server paypal with PYUSD, because 
                                once we received the payapl successful PYUSD payment 
                                transaction we should mint the NFT on chain.

                                ----------------- PAYMENT/FIAT LOGIC -----------------
                                    IR payment process to charge exchange
                                            ↓↑
                                    exchange charge server paypal with usdt or PYUSD
                                            ↓↑
                                    thridweb minting process on polygon chain (mint an nft to the receiver)
                                            ↓↑
                                    withdraw api
                                -------------------------------------------------------

                            */

                            tokio::task::spawn(async move{

                                // run python code to mint the nft using thirdweb api
                                // ...

                            });
                            
                            let deposit_ir_res = 200;
                            let mint_tx_signature = "tx-mint-hash".to_string();
                            if deposit_ir_res == 200{

                                match UserDeposit::insert(deposit.to_owned(), mint_tx_signature, connection).await{
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
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    CANT_DEPOSIT, // response message
                                    StatusCode::EXPECTATION_FAILED, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            }

                        } else{
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_DATA, // response message
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
        (status=429, description="Rate Limited, Chill 30 Seconds", body=&[u8]),
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

                    let identifier_key = format!("{}", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)));
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

                       
                       let hex_user_cid = &user_cid.to_owned()[2..];
                       match UserDeposit::get_all_for(hex_user_cid.to_string(), connection).await{
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

                    let identifier_key = format!("{}", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)));
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

                        let withdraw_object = withdraw.to_owned();

                        /* 
                            here is the process of verifying the signature of signed data 
                            using the private key inside js using themis wasm.
                            we've cloned the tx_signature and hex_pubkey since the PartialEq
                            trait takes the ownership of the WithdrawRequest instacen
                            when we want to compare the decoded data with the actual instance
                            inside the request body
                        */
                        let hex_tx_signature = withdraw_object.tx_signature.clone();
                        let hex_pubkey = withdraw_object.recipient_cid.clone();

                        /* dropping the first 2 bytes or 0x from the hex string */
                        let tx_signature = &hex_tx_signature[2..];
                        let pubkey = &hex_pubkey[2..];
                        
                        let decode_pubkey = hex::decode(pubkey.to_string());
                        let decode_signature = hex::decode(tx_signature.to_string());

                        let Ok(pubkey_bytes) = decode_pubkey else{
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_CID, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };


                        let Ok(signature) = decode_signature else{

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                SIGNATURE_DECODE_ISSUE, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        /* verifying the data against the generated signature */
                        let get_encoded_data = Id::verify(
                            &signature, 
                            &pubkey_bytes
                        );

                        let Ok(encoded_data) = get_encoded_data else{
                            
                            /* verifying signature or building pubkey error */
                            let error = get_encoded_data.unwrap_err();
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_SIGNATUE_OR_CID, // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

                        /* decoding the signed message bytes */
                        let pure_data = serde_json::from_slice::<DecodedSignedWithdrawalData>(&encoded_data).unwrap();

                        /* the decoded data must be equals to the request body */
                        let request_body_data = DecodedSignedWithdrawalData{
                            recipient_cid: withdraw_object.recipient_cid,
                            deposit_id: withdraw_object.deposit_id
                        };

                        /* the decoded data must be equals to the request body */
                        if pure_data == request_body_data.clone(){

                            /*
     
                                ----------------- WITHDRAWAL LOGIC --------------------
                                    withdraw process
                                            ↓↑
                                    burn the minted nft id on chain to get the burn tx signature
                                            ↓↑
                                    paying out the receiver with the PYUSD from the server paypal
                                            ↓↑
                                    show claimed paypal transaction to minter
                                ------------------------------------------------------- 
                            
                            */

                            tokio::task::spawn(async move{

                                // run python code to burn the minted nft using thirdweb api
                                // ...

                            });
                            
                            let payout_pyusd_res = 200;
                            let burn_tx_signature = "tx-burn-hash".to_string();
                            if payout_pyusd_res == 200{

                                match UserWithdrawal::insert(withdraw.to_owned(), burn_tx_signature, connection).await{
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
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    CANT_DEPOSIT, // response message
                                    StatusCode::EXPECTATION_FAILED, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            }

                        } else{
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                INVALID_DATA, // response message
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
        (status=201, description="Fetched Successfully", body=Vec<UserWithdrawalData>),
        (status=429, description="Rate Limited, Chill 30 Seconds", body=&[u8]),
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

                    let identifier_key = format!("{}", _id);
                    let Ok(mut redis_conn) = get_redis_conn else{

                        /* handling the redis connection error using PanelError */
                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)));
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

                       
                       let hex_user_cid = &user_cid.to_owned()[2..];
                       match UserWithdrawal::get_all_for(hex_user_cid.to_string(), connection).await{
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
    pub use super::make_id;
    pub use super::deposit;
    pub use super::withdraw;
    pub use super::get_all_user_withdrawals;
    pub use super::get_all_user_deposits;
}