


use crate::*;
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;



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
        verify_twitter_account,
        get_tasks,
        do_task,
        tasks_report,
    ),
    components(
        schemas(
            UserData,
            FetchUserTaskReport,
            TaskData
        )
    )
)]
pub struct UserApiDoc;

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
        ("wallet", description = "wallet address")
    ),
)]
#[post("/login/{wallet}")]
async fn login(
        req: HttpRequest, 
        wallet: web::Path<String>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            /* we can pass usernmae by reference or its slice form instead of cloning it */
            match User::find_by_wallet(&wallet.to_owned(), connection).await{
                Ok(user) => {
        
                    /* generate cookie 🍪 from token time and jwt */
                    /* since generate_cookie() takes the ownership of the user instance we must clone it then call this */
                    let cookie_info = user.clone().generate_cookie().unwrap();
                    let cookie_token_time = cookie_info.1;

                    let now = chrono::Local::now().naive_local();
                    let updated_user = diesel::update(users.find(user.id))
                        .set((last_login.eq(now), token_time.eq(cookie_token_time)))
                        .execute(connection)
                        .unwrap();
                    
                    let user_login_data = UserData{
                        id: user.id,
                        username: user.username.clone(),
                        activity_code: user.activity_code.clone(),
                        twitter_username: user.twitter_username.clone(),
                        facebook_username: user.facebook_username.clone(),
                        discord_username: user.discord_username.clone(),
                        wallet_address: user.wallet_address.clone(),
                        user_role: {
                            match user.user_role.clone(){
                                UserRole::Admin => "Admin".to_string(),
                                UserRole::User => "User".to_string(),
                                _ => "Dev".to_string(),
                            }
                        },
                        token_time: user.token_time,
                        last_login: { 
                            if user.last_login.is_some(){
                                Some(user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: user.created_at.to_string(),
                        updated_at: user.updated_at.to_string(),
                    };

                    resp!{
                        UserData, //// the data type
                        user_login_data, //// response data
                        LOGGEDIN, //// response message
                        StatusCode::OK, //// status code,
                        Some(cookie_info.0), //// cookie 
                    } 

                },
                Err(resp) => {

                    /* USER NOT FOUND response */
                    // resp
                    
                    /* gently, we'll insert this user into table */
                    match User::insert(wallet.to_owned(), connection).await{
                        Ok((user_login_data, cookie)) => {

                            resp!{
                                UserData, //// the data type
                                user_login_data, //// response data
                                REGISTERED, //// response message
                                StatusCode::CREATED, //// status code,
                                Some(cookie), //// cookie 
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None::<Cookie<'_>>, //// cookie
            }
        }
    }


}

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=200, description="Updated Successfully", body=UserData),
        (status=404, description="User Not Found", body=i32), // not found by id
        (status=404, description="User Not Found", body=String), // not found by wallet
        (status=404, description="No Value Found In Cookie", body=[u8]),
        (status=403, description="JWT Not Found In Cookie", body=[u8]),
        (status=406, description="No Time Hash Found In Cookie", body=[u8]),
        (status=406, description="Invalid Cookie Format", body=[u8]),
        (status=403, description="Cookie Has Expired", body=[u8]),
        (status=406, description="Invalid Cookie Time Hash", body=[u8]),
        (status=403, description="Access Denied", body=i32),
        (status=406, description="No Expiration Time Found In Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("account_name", description = "twitter account")
    ),
)]
#[post("/verify-twitter-account/{account_name}")]
async fn verify_twitter_account(
        req: HttpRequest,
        account_name: web::Path<String>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, Some(UserRole::User), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    let wallet = token_data.wallet.unwrap();

                    /* we can pass usernmae by reference or its slice form instead of cloning it */
                    match User::update_social_account(&wallet, &account_name.to_owned(), connection).await{
                        Ok(updated_user) => {
                
                            resp!{
                                UserData, //// the data type
                                updated_user, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code,
                                None::<Cookie<'_>>, //// cookie 
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None::<Cookie<'_>>, //// cookie
            }
        }
    }
}

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=200, description="Fetched Successfully", body=[TaskData]),
        (status=404, description="User Not Found", body=i32), // not found by id
        (status=404, description="No Value Found In Cookie", body=[u8]),
        (status=403, description="JWT Not Found In Cookie", body=[u8]),
        (status=406, description="No Time Hash Found In Cookie", body=[u8]),
        (status=406, description="Invalid Cookie Format", body=[u8]),
        (status=403, description="Cookie Has Expired", body=[u8]),
        (status=406, description="Invalid Cookie Time Hash", body=[u8]),
        (status=403, description="Access Denied", body=i32),
        (status=406, description="No Expiration Time Found In Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
)]
#[get("/get-tasks")]
async fn get_tasks(
        req: HttpRequest, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, Some(UserRole::User), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match Task::get_all(connection).await{
                        Ok(all_tasks) => {

                            resp!{
                                Vec<TaskData>, //// the data type
                                all_tasks, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None::<Cookie<'_>>, //// cookie
            }
        }
    }         


}

#[utoipa::path(
    context_path = "/user",
    responses(
        (status=201, description="Created Successfully", body=[u8]),
        (status=404, description="User Not Found", body=i32), // not found by id
        (status=404, description="Task Not Found", body=i32), // not found by id
        (status=404, description="No Value Found In Cookie", body=[u8]),
        (status=403, description="JWT Not Found In Cookie", body=[u8]),
        (status=406, description="No Time Hash Found In Cookie", body=[u8]),
        (status=406, description="Invalid Cookie Format", body=[u8]),
        (status=403, description="Cookie Has Expired", body=[u8]),
        (status=406, description="Invalid Cookie Time Hash", body=[u8]),
        (status=403, description="Access Denied", body=i32),
        (status=406, description="No Expiration Time Found In Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("task_id", description = "task id"),
        ("user_id", description = "user id"),
    ),
)]
#[post("/do-task/{task_id}/{user_id}")]
pub async fn do_task(
        req: HttpRequest,
        task_id: web::Path<i32>,
        user_id: web::Path<i32>,
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, Some(UserRole::User), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    let wallet = token_data.wallet.unwrap();

                    match UserTask::insert(user_id.to_owned(), task_id.to_owned(), connection).await{
                        Ok(_) => {
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                CREATED, //// response message
                                StatusCode::CREATED, //// status code
                                None::<Cookie<'_>>, //// cookie
                            }
                        },
                        Err(resp) => {

                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                                - TASK_NOT_FOUND
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None::<Cookie<'_>>, //// cookie
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
        (status=404, description="No Value Found In Cookie", body=[u8]),
        (status=403, description="JWT Not Found In Cookie", body=[u8]),
        (status=406, description="No Time Hash Found In Cookie", body=[u8]),
        (status=406, description="Invalid Cookie Format", body=[u8]),
        (status=403, description="Cookie Has Expired", body=[u8]),
        (status=406, description="Invalid Cookie Time Hash", body=[u8]),
        (status=403, description="Access Denied", body=i32),
        (status=406, description="No Expiration Time Found In Cookie", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("user_id", description = "user id"),
    ),
)]
#[post("/report-tasks/{user_id}")]
pub async fn tasks_report(
        req: HttpRequest,
        user_id: web::Path<i32>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, Some(UserRole::User), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    let wallet = token_data.wallet.unwrap();


                    match UserTask::reports(user_id.to_owned(), connection).await{
                        Ok(user_stask_reports) => {

                            resp!{
                                FetchUserTaskReport, //// the data type
                                user_stask_reports, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None::<Cookie<'_>>, //// cookie
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None::<Cookie<'_>>, //// cookie
            }
        }
    }
}


pub mod exports{
    pub use super::login;
    pub use super::verify_twitter_account;
    pub use super::get_tasks;
    pub use super::do_task;
    pub use super::tasks_report;
}