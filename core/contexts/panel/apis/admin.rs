



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
                        }
                    };

                    let cookie_info = user.generate_cookie().unwrap();
                    let cookie_value = cookie_info.0.value().to_string();
                    let cookie_token_time = cookie_info.1;

                    let now = chrono::Local::now().naive_local();
                    let updated_user = diesel::update(users.find(user.id))
                        .set((last_login.eq(now), token_time.eq(cookie_token_time)))
                        .execute(connection)
                        .unwrap();
                    
                    resp!{
                        String, //// the data type
                        cookie_value, //// response data
                        FETCHED, //// response message
                        StatusCode::OK, //// status code
                    } 

                },
                _ => {

                    resp!{
                        String, //// the data type
                        user_name.to_owned(), //// response data
                        ACCESS_DENIED, //// response message
                        StatusCode::FORBIDDEN, //// status code
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

                    let inserted_admin_id = diesel::insert_into(users::table)
                        .values(&new_admin)
                        .execute(connection)
                        .unwrap();


                    resp!{
                        &[u8], //// the data type
                        &[], //// response data
                        CREATED, //// response message
                        StatusCode::OK, //// status code
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
            
            match User::passport(req, UserRole::Admin, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    let task = NewTaskRequest{
                        task_name: new_task.task_name.clone(),
                        task_description: new_task.task_description.clone(),
                        task_score: new_task.task_score,
                        admin_id: new_task.admin_id,
                    };


                    // add new task to db using join
                    // publish/fire new task/event or topic to all 
                    //  users who have user role, using redis 
                    // ...

                    let inserted_admin_id = diesel::insert_into(tasks::table)
                        .values(&task)
                        .execute(connection)
                        .unwrap();


                    resp!{
                        &[u8], //// the data type
                        &[], //// response data
                        CREATED, //// response message
                        StatusCode::OK, //// status code
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
            }
        }
    }         


}





pub mod exports{
    pub use super::login;
    pub use super::register_new_admin;
    pub use super::register_new_task;    
}