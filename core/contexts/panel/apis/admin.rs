



use crate::*;
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use crate::schema::users_tasks::dsl::*;
use crate::schema::users_tasks;



/*
     ------------------------
    |          DOCS
    | ------------------------
    |
    |

*/
#[derive(OpenApi)]
#[openapi(paths(reveal_role))]
// #[openapi(paths(login))]
// #[openapi(paths(egister_new_admin))]
// #[openapi(paths(register_new_task))]
// #[openapi(paths(delete_task))]
// #[openapi(paths(edit_task))]
// #[openapi(paths(delete_user))]
// #[openapi(paths(get_users))]
// #[openapi(paths(get_admin_tasks))]
// #[openapi(paths(get_users_tasks))]
pub struct AdminApiDoc;


/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/
#[utoipa::path(
    context_path="/admin",
    responses(
        (status=201, description="Created Successfully", body=Result<HttpResponse, actix_web::Error>)
    )
)]
#[post("/notif/register/reveal-role/{id}")]
async fn reveal_role(
        req: HttpRequest, 
        _id: web::Path<i32>, 
        redis_conn: web::Data<RedisConnection>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    
    if let Some(header_value) = req.headers().get("Authorization"){
    
        let token = header_value.to_str().unwrap();
        
        /*
            @params: 
                - @request       → actix request object
                - @storage       → instance inside the request object
                - @access levels → vector of access levels
        */
        match passport!{ token }{
            true => {

                //// -------------------------------------------------------------------------------------
                //// ------------------------------- ACCESS GRANTED REGION -------------------------------
                //// -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_conn = redis_conn.to_owned();
                let mongo_db = storage.clone().unwrap().get_mongodb().await.unwrap();

                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
            
                        
                        // 🥑 todo - publish or fire the reveal role topic or event using redis pubsub
                        // 🥑 todo - also call the /reveal/roles api of the hyper server                 
                        // ...

                        let mq = events::redis::mmq::MatchQueue{..Default::default()};
                        let role = events::redis::role::Reveal;
            
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            CREATED, //// response message
                            StatusCode::CREATED, //// status code
                            None, //// cookie
                        } 
            
            
                    },
                    None => {
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            STORAGE_ISSUE, //// response message
                            StatusCode::INTERNAL_SERVER_ERROR, //// status code
                            None, //// cookie
                        }
                    }
                }

                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------

            },
            false => {
                
                resp!{
                    &[u8], //// the date type
                    &[], //// the data itself
                    INVALID_TOKEN, //// response message
                    StatusCode::FORBIDDEN, //// status code
                    None, //// cookie
                }
            }
        }

    } else{
        
        resp!{
            &[u8], //// the date type
            &[], //// the data itself
            NOT_AUTH_HEADER, //// response message
            StatusCode::FORBIDDEN, //// status code
            None, //// cookie
        }
    }

}

#[utoipa::path(
    context_path="/admin",
    responses(
        (status=200, description="Loggedin Successfully", body=Result<HttpResponse, actix_web::Error>)
    )
)]
#[post("/login")]
pub(super) async fn login(
        req: HttpRequest, 
        login_info: web::Json<LoginInfoRequest>,
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {
   
    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            let user_name = login_info.to_owned().username;
            let password = login_info.to_owned().password;

            /* we can pass usernmae by reference or its slice form instead of cloning it */
            match User::find_by_username(&user_name, connection).await{
                Ok(user) => {

                    match user.user_role{
                        UserRole::Admin => {
        
                            let hash_pswd = User::hash_pswd(password.as_str()).unwrap();
                            let Ok(_) = user.verify_pswd(hash_pswd.as_str()) else{
                                resp!{
                                    String, //// the data type
                                    user_name.to_owned(), //// response data
                                    WRONG_PASSWORD, //// response message
                                    StatusCode::FORBIDDEN, //// status code
                                    None, //// cookie
                                }
                            };
        
                            /* generate cookie 🍪 from token time and jwt */
                            /* since generate_cookie() takes the ownership of the user instance we must clone it then call this */
                            /* generate_cookie() returns a Cookie instance with a 'static lifetime which allows us to return it from here*/
                            let cookie_info = user.clone().generate_cookie().unwrap();
                            let cookie_token_time = cookie_info.1;
                            
                            /* update the login token time */
                            let now = chrono::Local::now().naive_local();
                            let updated_user = diesel::update(users.find(user.id))
                                .set((last_login.eq(now), token_time.eq(cookie_token_time)))
                                .execute(connection)
                                .unwrap();
                            
                            let user_login_data = UserLoginData{
                                id: user.id,
                                username: user.username.clone(),
                                twitter_username: user.twitter_username.clone(),
                                facebook_username: user.facebook_username.clone(),
                                discord_username: user.discord_username.clone(),
                                wallet_address: user.wallet_address.clone(),
                                user_role: user.user_role.clone(),
                                token_time: user.token_time,
                                last_login: user.last_login,
                                created_at: user.created_at,
                                updated_at: user.updated_at,
                            };

                            resp!{
                                UserLoginData, //// the data type
                                user_login_data, //// response data
                                LOGGEDIN, //// response message
                                StatusCode::OK, //// status code,
                                Some(cookie_info.0), //// cookie 
                            } 
        
                        },
                        _ => {
        
                            resp!{
                                String, //// the data type
                                user_name.to_owned(), //// response data
                                ACCESS_DENIED, //// response message
                                StatusCode::FORBIDDEN, //// status code
                                None, //// cookie
                            } 
                        }
                    }
                },
                Err(resp) => {

                    /* USER NOT FOUND response */
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
                None, //// cookie
            }
        }
    }

}

#[post("/register-new-admin")]
async fn register_new_admin(
        req: HttpRequest,  
        new_admin: web::Json<NewAdminInfoRequest>,
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* to_owned() will take the NewAdminInfoRequest instance out of the web::Json*/
                    match User::insert_new_admin(new_admin.to_owned(), connection).await{
                        Ok(_) => {
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                CREATED, //// response message
                                StatusCode::CREATED, //// status code
                                None, //// cookie
                            }
                        }, 
                        Err(resp) => {

                            /* DIESEL INSERT ERROR RESPONSE */
                            resp
                        }
                    }



                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None, //// cookie
            }
        }
    }        

}

#[post("/edit-user")]
async fn edit_user(
        req: HttpRequest, 
        new_user: web::Json<EditUserByAdminRequest>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match User::edit_by_admin(new_user.to_owned(), connection).await{
                        Ok(updated_user) => {

                            resp!{
                                FetchUser, //// the data type
                                updated_user, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None, //// cookie
                            }

                        },
                        Err(resp) => {

                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL EDIT ERROR RESPONSE
                                - USER_NOT_FOUND 

                            */
                            resp

                        }
                    }
                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None, //// cookie
            }
        }
    }
}

#[post("/delete-user/{user_id}")]
async fn delete_user(
        req: HttpRequest, 
        doer_id: web::Path<i32>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match User::delete_by_admin(doer_id.to_owned(), connection).await{
                        Ok(num_deleted) => {

                            if num_deleted == 0{
                                resp!{
                                    &[u8], //// the data type
                                    &[], //// response data
                                    DELETED, //// response message
                                    StatusCode::OK, //// status code
                                    None, //// cookie
                                }
                            } else{
                                
                                resp!{
                                    &[u8], //// the data type
                                    &[], //// response data
                                    TASK_NOT_FOUND, //// response message
                                    StatusCode::NOT_FOUND, //// status code
                                    None, //// cookie
                                }
                            }

                        },  
                        Err(resp) => {

                            /* DIESEL DELETE RESPONSE */
                            resp
                        }
                    }

                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None, //// cookie
            }
        }
    }         


}

#[post("/get-users")]
async fn get_users(
        req: HttpRequest, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match User::get_all(connection).await{
                        Ok(all_users) => {
                            resp!{
                                Vec<User>, //// the data type
                                all_users, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None, //// cookie
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None, //// cookie
            }
        }
    }         
}

#[post("/register-new-task")]
async fn register_new_task(
        req: HttpRequest, 
        new_task: web::Json<NewTaskRequest>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match Task::insert(new_task.to_owned(), redis_conn, connection).await{
                        Ok(_) => {

                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                CREATED, //// response message
                                StatusCode::CREATED, //// status code
                                None, //// cookie
                            }

                        },
                        Err(resp) => {
                            
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
                                - FOUND_TASK
                            */
                            resp

                        }
                    } 
                    
                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None, //// cookie
            }
        }
    }         


}

#[post("/delete-task/{job_id}")]
async fn delete_task(
        req: HttpRequest, 
        job_id: web::Path<i32>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match Task::delete(job_id.to_owned(), connection).await{
                        Ok(num_deleted) => {

                            if num_deleted == 0{
                                resp!{
                                    &[u8], //// the data type
                                    &[], //// response data
                                    DELETED, //// response message
                                    StatusCode::OK, //// status code
                                    None, //// cookie
                                }
                            } else{
                                
                                resp!{
                                    &[u8], //// the data type
                                    &[], //// response data
                                    TASK_NOT_FOUND, //// response message
                                    StatusCode::NOT_FOUND, //// status code
                                    None, //// cookie
                                }
                            }

                        },
                        Err(resp) => {
                            
                            /* DIESEL DELETE ERROR RESPONSE */
                            resp
                        }
                    }

                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None, //// cookie
            }
        }
    }         


}

#[post("/edit-task")]
async fn edit_task(
        req: HttpRequest, 
        new_task: web::Json<EditTaskRequest>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    
                    match Task::edit(new_task.to_owned(), connection).await{
                        Ok(updated_task) => {

                            resp!{
                                Task, //// the data type
                                updated_task, //// response data
                                UPDATED, //// response message
                                StatusCode::OK, //// status code
                                None, //// cookie
                            }

                        },
                        Err(resp) => {
                            
                            /* DIESEL EDIT ERROR RESPONSE */
                            resp
                        }
                    }

                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None, //// cookie
            }
        }
    }
}

#[post("/get-admin-tasks/{owner_id}")]
async fn get_admin_tasks(
        req: HttpRequest, 
        owner_id: web::Path<i32>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match Task::get_all_admin(owner_id.to_owned(), connection).await{
                        Ok(admin_tasks) => {

                            resp!{
                                Vec<Task>, //// the data type
                                admin_tasks, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None, //// cookie
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None, //// cookie
            }
        }
    }         


}

#[get("/get-users-tasks")]
async fn get_users_tasks(
        req: HttpRequest,  
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match UserTask::tasks_per_user(connection).await{
                        Ok(all_users_tasks) => {

                            resp!{
                                Vec<(User, Vec<Task>)>, //// the data type
                                all_users_tasks, //// response data
                                FETCHED, //// response message
                                StatusCode::OK, //// status code
                                None, //// cookie
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
                &[u8], //// the data type
                &[], //// response data
                STORAGE_ISSUE, //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                None, //// cookie
            }
        }
    }         


}


pub mod exports{
    pub use super::reveal_role;   
    pub use super::login;
    pub use super::register_new_admin;
    pub use super::register_new_task; 
    pub use super::delete_task;
    pub use super::edit_task;
    pub use super::edit_user;
    pub use super::delete_user;
    pub use super::get_users;
    pub use super::get_admin_tasks;
    pub use super::get_users_tasks;
}