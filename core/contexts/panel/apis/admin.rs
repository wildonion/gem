



use crate::*;
use crate::models::{users::*, tasks::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;



/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/
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
                            FETCHED, //// response message
                            StatusCode::CREATED, //// status code
                            None,
                        } 
            
            
                    },
                    None => {
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            STORAGE_ISSUE, //// response message
                            StatusCode::INTERNAL_SERVER_ERROR, //// status code
                            None,
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
                    None,
                }
            }
        }

    } else{
        
        resp!{
            &[u8], //// the date type
            &[], //// the data itself
            NOT_AUTH_HEADER, //// response message
            StatusCode::FORBIDDEN, //// status code
            None,
        }
    }

}

#[post("/login")]
pub(super) async fn login(
        req: HttpRequest, 
        user_name: web::Path<String>,
        password: web::Path<String>,
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {
   
    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            let single_user = users
                                .filter(username.eq(user_name.to_owned()))
                                .first::<User>(connection);

            let Ok(user) = single_user else{
                resp!{
                    String, //// the data type
                    user_name.to_owned(), //// response data
                    USER_NOT_FOUND, //// response message
                    StatusCode::NOT_FOUND, //// status code
                    None,
                } 
            };

            match user.user_role{
                UserRole::Admin => {

                    let hash_pswd = User::hash_pswd(password.as_str()).unwrap();
                    let Ok(_) = user.verify_pswd(hash_pswd.as_str()) else{
                        resp!{
                            String, //// the data type
                            user_name.to_owned(), //// response data
                            WRONG_PASSWORD, //// response message
                            StatusCode::FORBIDDEN, //// status code
                            None,
                        }
                    };

                    /* generate cookie from token time and jwt */
                    let cookie_info = user.generate_cookie().unwrap();
                    let cookie_token_time = cookie_info.1;

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
                        FETCHED, //// response message
                        StatusCode::OK, //// status code,
                        Some(cookie_info.0), //// response cookie 
                    } 

                },
                _ => {

                    resp!{
                        String, //// the data type
                        user_name.to_owned(), //// response data
                        ACCESS_DENIED, //// response message
                        StatusCode::FORBIDDEN, //// status code
                        None,
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
                None,
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
            
            /* ONLY ADMIN CAN DO THIS LOGIC */
            match User::passport(req, UserRole::Admin, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let hash_pswd = User::hash_pswd(new_admin.password.as_str()).unwrap();
                    let new_admin = NewUser{
                        username: new_admin.username.as_str(),
                        user_role: UserRole::Admin,
                        pswd: hash_pswd.as_str()
                    };

                    let affected_row = diesel::insert_into(users::table)
                        .values(&new_admin)
                        .execute(connection)
                        .unwrap();


                    resp!{
                        &[u8], //// the data type
                        &[], //// response data
                        CREATED, //// response message
                        StatusCode::CREATED, //// status code
                        None,
                    }

                },
                Err(resp) => {
                    
                    /* 
                        based on the flow response can be one of the following:
                        
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
                None,
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
            
            /* ONLY ADMIN CAN DO THIS LOGIC */
            match User::passport(req, UserRole::Admin, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* fetch user info based on the data inside jwt */ 
                    let single_user = users
                        .filter(users::id.eq(new_user.user_id.to_owned()))
                        .first::<User>(connection);

                    let Ok(user) = single_user else{
                        resp!{
                            i32, //// the data type
                            _id.to_owned(), //// response data
                            USER_NOT_FOUND, //// response message
                            StatusCode::NOT_FOUND, //// status code
                            None,
                        }
                    };
                    
                    /* if the passed in password was some then we must updated the password */
                    let password = if let Some(password) = &new_user.password{ //// borrowing the new_user to prevent from moving

                        /* we can pass &str to the method by borrowing the String since String will be coerced into &str at compile time */
                        User::hash_pswd(password).unwrap()

                    } else{
                        
                        /* if the passed in password was none then we must use the old one */
                        user.pswd

                    };
                    
                    let updated_user = diesel::update(users.find(new_user.user_id.to_owned()))
                        .set(EditUserByAdmin{
                            user_role: {
                                let role = new_user.role.as_str(); 
                                match role{
                                    "user" => UserRole::User,
                                    "admin" => UserRole::Admin,
                                    _ => UserRole::Dev
                                }
                            },
                            /* pswd is of type &str thus by borrowing password we can convert it into &str */
                            pswd: &password
                        })
                        .returning(FetchUser::as_returning())
                        .get_result(connection)
                        .unwrap();


                    resp!{
                        FetchUser, //// the data type
                        updated_user, //// response data
                        UPDATED, //// response message
                        StatusCode::OK, //// status code
                        None,
                    }

                },
                Err(resp) => {
                    
                    /* 
                        response can be one of the following:
                        
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
                None,
            }
        }
    }
}

#[post("/delete-user")]
async fn delete_user(
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
            
            /* ONLY ADMIN CAN DO THIS LOGIC */
            match User::passport(req, UserRole::Admin, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    let num_deleted = diesel::delete(users.filter(users::id.eq(user_id.to_owned())))
                        .execute(connection)
                        .unwrap();

                    resp!{
                        &[u8], //// the data type
                        &[], //// response data
                        DELETED, //// response message
                        StatusCode::OK, //// status code
                        None,
                    }

                },
                Err(resp) => {
                    
                    /* 
                        response can be one of the following:
                        
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
                None,
            }
        }
    }         


}

#[post("/get-users")]
async fn get_users(
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
            
            /* ONLY ADMIN CAN DO THIS LOGIC */
            match User::passport(req, UserRole::Admin, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    let results = users.load::<User>(connection).unwrap();

                    resp!{
                        Vec<User>, //// the data type
                        results, //// response data
                        FETCHED, //// response message
                        StatusCode::OK, //// status code
                        None,
                    }

                },
                Err(resp) => {
                    
                    /* 
                        response can be one of the following:
                        
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
                None,
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
            
            /* ONLY ADMIN CAN DO THIS LOGIC */
            match User::passport(req, UserRole::Admin, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    let single_task = tasks
                                .filter(task_name.eq(new_task.task_name.clone()))
                                .first::<Task>(connection);

                    let Ok(task) = single_task else{
                        resp!{
                            String, //// the data type
                            new_task.task_name.clone(), //// response data
                            FOUND_TASK, //// response message
                            StatusCode::FOUND, //// status code
                            None,
                        } 
                    };

                    let task = NewTask{
                        task_name: new_task.task_name.as_str(),
                        task_description: Some(new_task.task_description.as_str()),
                        task_score: new_task.task_score,
                        admin_id: new_task.admin_id,
                    };

                    // publish/fire new task event/topic using redis 
                    // ... 

                    let affected_row = diesel::insert_into(tasks::table)
                        .values(&task)
                        .execute(connection)
                        .unwrap();

                    resp!{
                        &[u8], //// the data type
                        &[], //// response data
                        CREATED, //// response message
                        StatusCode::CREATED, //// status code
                        None,
                    }

                },
                Err(resp) => {
                    
                    /* 
                        response can be one of the following:
                        
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
                None,
            }
        }
    }         


}

#[post("/delete-task")]
async fn delete_task(
        req: HttpRequest, 
        task_id: web::Path<i32>, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* ONLY ADMIN CAN DO THIS LOGIC */
            match User::passport(req, UserRole::Admin, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    let num_deleted = diesel::delete(tasks.filter(tasks::id.eq(task_id.to_owned())))
                        .execute(connection)
                        .unwrap();

                    resp!{
                        &[u8], //// the data type
                        &[], //// response data
                        DELETED, //// response message
                        StatusCode::OK, //// status code
                        None,
                    }

                },
                Err(resp) => {
                    
                    /* 
                        response can be one of the following:
                        
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
                None,
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
            
            /* ONLY ADMIN CAN DO THIS LOGIC */
            match User::passport(req, UserRole::Admin, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    
                    let updated_task = diesel::update(tasks.find(new_task.task_id.to_owned()))
                        .set(EditTask{
                            /* 
                                task name and description are of type &str 
                                thus by borrowing new_task struct fields we
                                can convert them into &str 
                            */
                            task_name: &new_task.task_name, 
                            task_description: &new_task.task_description,
                            task_score: new_task.task_score
                        })
                        .returning(Task::as_returning())
                        .get_result(connection)
                        .unwrap();


                    resp!{
                        Task, //// the data type
                        updated_task, //// response data
                        UPDATED, //// response message
                        StatusCode::OK, //// status code
                        None,
                    }

                },
                Err(resp) => {
                    
                    /* 
                        response can be one of the following:
                        
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
                None,
            }
        }
    }
}

#[post("/get-admin-tasks")]
async fn get_admin_tasks(
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
            
            /* ONLY ADMIN CAN DO THIS LOGIC */
            match User::passport(req, UserRole::Admin, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    /* get the passed in admin info by its id */
                    let user = users::table
                        .filter(users::id.eq(user_id.to_owned()))
                        .select(User::as_select())
                        .get_result::<User>(connection)
                        .unwrap();

                    /* get all tasks belonging to the passed in admin id */
                    let admin_tasks = Task::belonging_to(&user)
                        .select(Task::as_select())
                        .load(connection)
                        .unwrap();


                    resp!{
                        Vec<Task>, //// the data type
                        admin_tasks, //// response data
                        FETCHED, //// response message
                        StatusCode::OK, //// status code
                        None,
                    }

                },
                Err(resp) => {
                    
                    /* 
                        response can be one of the following:
                        
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
                None,
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
}