






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
use std::io::Write;


/*
     -------------------------------
    |          SWAGGER DOC
    | ------------------------------
    |
    |

*/
#[derive(OpenApi)]
#[openapi(
    paths(
        reveal_role,   
        login,
        register_new_user,
        register_new_task, 
        delete_task,
        edit_task,
        edit_user,
        delete_user,
        get_users,
        get_admin_tasks,
        get_users_tasks,
        add_twitter_account
    ),
    components(
        schemas(
            Keys,
            TwitterAccounts,
            UserTaskData,
            UserData,
            TaskData,
            LoginInfoRequest,
            NewUserInfoRequest,
            EditUserByAdminRequest,
            NewTaskRequest,
            EditTaskRequest
        )
    ),
    tags(
        (name = "crate::apis::admin", description = "Admin Endpoints")
    ),
    info(
        title = "Admin Access APIs"
    ),
    modifiers(&SecurityAddon),
)]
pub struct AdminApiDoc;
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
    context_path = "/admin",
    responses(
        (status=201, description="Created Successfully", body=[u8]),
        (status=403, description="Invalid Token", body=[u8]),
        (status=403, description="No Authorization Header Is Provided", body=[u8]),
        (status=500, description="Storage Issue", body=[u8])
    ),
    params(
        ("event_id" = String, Path, description = "event id")
    ),
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[post("/notif/register/reveal-role/{event_id}")]
async fn reveal_role(
        req: HttpRequest, 
        event_id: web::Path<String>, // mongodb objectid
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
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

                // -------------------------------------------------------------------------------------
                // ------------------------------- ACCESS GRANTED REGION -------------------------------
                // -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                let mongo_db = storage.clone().unwrap().get_mongodb().await.unwrap();
                let redis_password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());
                let redis_actor = storage.as_ref().clone().unwrap().get_redis_actor().await.unwrap();

                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
                


                        // todo - fetch from mafia hyper server



                        let revealed = events::redis::role::Reveal::default();


                        /* 
                                    
                            --------------------------------
                              AUTHORIZING WITH REDIS ACTOR
                            --------------------------------

                        */

                        /* sending command to redis actor to authorize the this ws client */
                        let redis_auth_resp = redis_actor
                            .send(Command(resp_array!["AUTH", redis_password.as_str()])).await;

                        let Ok(_) = redis_auth_resp else{
                            
                            let mailbox_err = redis_auth_resp.unwrap_err();
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &mailbox_err.to_string(), // response message
                                StatusCode::NOT_ACCEPTABLE, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

                        /* 

                            ----------------------------------------------------------------------
                              PUBLISHING REVEALED ROLE TO THE PASSED IN CHANNEL WITH REDIS ACTOR
                            ----------------------------------------------------------------------
                            since the websocket session has a 1 second interval for push notif 
                            subscription, we must publish roles contantly to the related channel 
                            to make sure that the subscribers will receive the message at their 
                            own time.

                        */

                        let notif_room = revealed.event_id.clone();
                        let revealed_roels = revealed.roles.clone();
                        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

                        tokio::spawn(async move{

                            /* publish until at least one sub subscribe to the topic */
                            loop{

                                interval.tick().await;

                                let redis_pub_resp = redis_actor
                                    .send(Command(resp_array!["PUBLISH", &notif_room.clone(), revealed_roels.clone()]))
                                    .await
                                    .unwrap();
    
                                    match redis_pub_resp{
                                        Ok(resp_val) => {
            
                                            match resp_val{
                                                RespValue::Integer(subs) => {
                                                    
                                                    if subs >= 1{
                                                        
                                                        /* if we're here means that ws session received the notif */
                                                        info!("💡 --- [{subs:}] online users subscribed to event: [{notif_room:}] to receive roles notif");
                                                        break;
        
                                                    }
            
                                                },
                                                _ => {}
                                            }
            
                                        },
                                        Err(e) => {}
                                    }
                            }

                        });
                    
                    resp!{
                        &[u8], // the data type
                        &[], // response data
                        PUSH_NOTIF_ACTIVATED, // response message
                        StatusCode::CREATED, // status code
                        None::<Cookie<'_>>, // cookie
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

#[utoipa::path(
    context_path = "/admin",
    request_body = LoginInfoRequest,
    responses(
        (status=200, description="Loggedin Successfully", body=UserData),
        (status=403, description="Wrong Password", body=String),
        (status=404, description="User Not Found", body=i32), // not found by id
        (status=500, description="Storage Issue", body=[u8]),
        (status=403, description="Access Denied", body=String), // access denied by wallet
    ),
    tag = "crate::apis::admin",
)]
#[post("/login")]
async fn login(
        req: HttpRequest, 
        login_info: web::Json<LoginInfoRequest>, 
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {
   
    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


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
                            
                            let pswd_verification = user.verify_pswd(password.as_str()); 
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
                                    user_name.to_owned(), // response data
                                    WRONG_PASSWORD, // response message
                                    StatusCode::FORBIDDEN, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            }
        
                            /* generate cookie 🍪 from token time and jwt */
                            /* since generate_cookie_and_jwt() takes the ownership of the user instance we must clone it then call this */
                            /* generate_cookie_and_jwt() returns a Cookie instance with a 'static lifetime which allows us to return it from here*/
                            let keys_info = user.clone().generate_cookie_and_jwt().unwrap();
                            let cookie_token_time = keys_info.1;
                            let jwt = keys_info.2;
                            
                            /* update the login token time */
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
                                wallet_address: user.wallet_address.clone(),
                                user_role: {
                                    match user.user_role.clone(){
                                        UserRole::Admin => "Admin".to_string(),
                                        UserRole::Dev => "User".to_string(),
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
                            };

                            resp!{
                                UserData, // the data type
                                user_login_data, // response data
                                LOGGEDIN, // response message
                                StatusCode::OK, // status code,
                                Some(keys_info.0), // cookie 
                            } 
        
                        },
                        _ => {
        
                            resp!{
                                String, // the data type
                                user_name.to_owned(), // response data
                                ACCESS_DENIED, // response message
                                StatusCode::FORBIDDEN, // status code
                                None::<Cookie<'_>>, // cookie
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
    context_path = "/admin",
    request_body = NewUserInfoRequest,
    responses(
        (status=201, description="Created Successfully", body=[u8]),
        (status=404, description="User Not Found", body=i32), // not found by id
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
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[post("/register-new-user")]
async fn register_new_user(
        req: HttpRequest,  
        new_user: web::Json<NewUserInfoRequest>, 
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* to_owned() will take the NewUserInfoRequest instance out of the web::Json*/
                    match User::insert_new_user(new_user.to_owned(), connection, redis_client).await{
                        Ok(_) => {

                            /* fetch all users again to get the newly one */
                            let get_all_users = User::get_all(connection).await;
                            let Ok(all_users) = get_all_users else{
                                let resp = get_all_users.unwrap_err();
                                return resp;
                            };

                            /* update redis cacher with the new user */
                            let rc_data = serde_json::to_string(&all_users).unwrap();
                            let _: () = redis_conn.set("get_all_users", rc_data).await.unwrap();


                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                CREATED, // response message
                                StatusCode::CREATED, // status code
                                None::<Cookie<'_>>, // cookie
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
    context_path = "/admin",
    request_body = EditUserByAdminRequest,
    responses(
        (status=200, description="Updated Successfully", body=[UserData]),
        (status=404, description="User Not Found", body=i32), // not found by id
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
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[post("/edit-user")]
async fn edit_user(
        req: HttpRequest, 
        new_user: web::Json<EditUserByAdminRequest>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match User::edit_by_admin(new_user.to_owned(), connection).await{
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
    context_path = "/admin",
    responses(
        (status=200, description="Deleted Successfully", body=[u8]),
        (status=404, description="User Not Found", body=i32), // not found by id
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
        ("user_id" = i32, Path, description = "user id")
    ),
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[post("/delete-user/{user_id}")]
async fn delete_user(
        req: HttpRequest, 
        doer_id: web::Path<i32>,  // doer is the user who do task
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match User::delete_by_admin(doer_id.to_owned(), connection).await{
                        Ok(num_deleted) => {

                            /* 
                                in this case we are sure that we removed at least one record 
                                of users table and >1 records from the users_tasks table
                            */
                            if num_deleted > 0{
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    DELETED, // response message
                                    StatusCode::OK, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            } else{
                                
                                resp!{
                                    i32, // the data type
                                    doer_id.to_owned(), // response data
                                    USER_NOT_FOUND, // response message
                                    StatusCode::NOT_FOUND, // status code
                                    None::<Cookie<'_>>, // cookie
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
    context_path = "/admin",
    responses(
        (status=200, description="Fetched Successfully", body=[UserData]),
        (status=404, description="User Not Found", body=i32), // not found by id
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
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[get("/get-users")]
async fn get_users(
        req: HttpRequest,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* create a response cacher using redis */
                    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
                    let get_all_users_key = format!("get_all_users");
                    let redis_result_check_token: RedisResult<String> = redis_conn.get(get_all_users_key.as_str()).await;
                    let mut redis_get_users = match redis_result_check_token{
                        Ok(data) => {
                            let rc_data = serde_json::from_str::<Vec<UserData>>(data.as_str()).unwrap();
                            Some(rc_data)
                        },
                        Err(e) => {
                            let empty_get_users: Option<Vec<UserData>> = None;
                            let rc_data = serde_json::to_string(&empty_get_users).unwrap();
                            let _: () = redis_conn.set("get_all_users", rc_data).await.unwrap();
                            None
                        }
                    };
                    
                    /* no caching is in redis we must fetch from pg */
                    if redis_get_users.is_none(){

                        match User::get_all(connection).await{
                            Ok(all_users) => {

                                /* chache the response for the next request */
                                let rc_data = serde_json::to_string(&all_users).unwrap();
                                let _: () = redis_conn.set("get_all_users", rc_data).await.unwrap();

                                resp!{
                                    Vec<UserData>, // the data type
                                    all_users, // response data
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

                    /* return redis cache */
                    } else{

                        resp!{
                            Vec<UserData>, // the data type
                            redis_get_users.unwrap(), // response data
                            FETCHED, // response message
                            StatusCode::OK, // status code
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
    context_path = "/admin",
    request_body = NewTaskRequest,
    responses(
        (status=201, description="Created Successfully", body=[TaskData]),
        (status=302, description="Task Has Already Been Registered", body=[TaskData]),
        (status=404, description="User Not Found", body=i32), // not found by id
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
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[post("/register-new-task")]
async fn register_new_task(
        req: HttpRequest, 
        new_task: web::Json<NewTaskRequest>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match Task::insert(new_task.to_owned(), redis_client, connection).await{
                        Ok(_) => {

                            /* fetch all admin tasks again to get the newly one */
                            let get_all_admin_tasks = Task::get_all_admin(new_task.admin_id, connection).await;
                            let Ok(all_admin_tasks) = get_all_admin_tasks else{
                                let resp = get_all_admin_tasks.unwrap_err();
                                return resp;
                            };

                            /* update redis cacher with the new task */
                            let rc_data = serde_json::to_string(&all_admin_tasks).unwrap();
                            let get_admin_tasks_key = format!("get_admin_tasks_{:?}", new_task.admin_id);
                            let _: () = redis_conn.set(get_admin_tasks_key.as_str(), rc_data).await.unwrap();


                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                CREATED, // response message
                                StatusCode::CREATED, // status code
                                None::<Cookie<'_>>, // cookie
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
    context_path = "/admin",
    responses(
        (status=200, description="Deleted Successfully", body=[u8]),
        (status=404, description="Task Not Found", body=i32), // not found by id
        (status=404, description="User Not Found", body=i32), // not found by id
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
        ("job_id" = i32, Path, description = "task id")
    ),
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[post("/delete-task/{job_id}")]
async fn delete_task(
        req: HttpRequest, 
        job_id: web::Path<i32>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match Task::delete(job_id.to_owned(), connection).await{
                        Ok(num_deleted) => {

                            if num_deleted == 1{
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    TASK_DELETED_WITH_NO_DOER, // response message
                                    StatusCode::OK, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            } else if num_deleted > 1{
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    TASK_DELETED_WITH_DOER, // response message
                                    StatusCode::OK, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            } else{
                                
                                /* task didn't found thus no users_tasks record related to the task is exists */
                                resp!{
                                    i32, // the data type
                                    job_id.to_owned(), // response data
                                    TASK_NOT_FOUND, // response message
                                    StatusCode::NOT_FOUND, // status code
                                    None::<Cookie<'_>>, // cookie
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
    context_path = "/admin",
    request_body = EditTaskRequest,
    responses(
        (status=200, description="Updated Successfully", body=[TaskData]),
        (status=404, description="Task Not Found", body=[u8]),
        (status=404, description="User Not Found", body=i32), // not found by id
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
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[post("/edit-task")]
async fn edit_task(
        req: HttpRequest, 
        new_task: web::Json<EditTaskRequest>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    
                    match Task::edit(new_task.to_owned(), connection).await{
                        Ok(updated_task) => {

                            resp!{
                                TaskData, // the data type
                                updated_task, // response data
                                UPDATED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
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
    context_path = "/admin",
    responses(
        (status=200, description="Fetched Successfully", body=[TaskData]),
        (status=404, description="User Not Found", body=i32), // not found by id
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
        ("owner_id" = i32, Path, description = "task owner id")
    ),
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[get("/get-admin-tasks/{owner_id}")]
async fn get_admin_tasks(
        req: HttpRequest, 
        owner_id: web::Path<i32>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;


                    /* create a response cacher using redis */
                    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
                    let get_admin_tasks_key = format!("get_admin_tasks_{:?}", owner_id.to_owned());
                    let redis_result_admin_tasks: RedisResult<String> = redis_conn.get(get_admin_tasks_key.as_str()).await;
                    let mut redis_get_admin_tasks = match redis_result_admin_tasks{
                        Ok(data) => {
                            let rc_data = serde_json::from_str::<Vec<TaskData>>(data.as_str()).unwrap();
                            Some(rc_data)
                        },
                        Err(e) => {
                            let empty_admin_tasks: Option<Vec<TaskData>> = None;
                            let rc_data = serde_json::to_string(&empty_admin_tasks).unwrap();
                            let get_admin_tasks_key = format!("get_admin_tasks_{:?}", owner_id.to_owned());
                            let _: () = redis_conn.set(get_admin_tasks_key.as_str(), rc_data).await.unwrap();
                            None
                        }
                    };

                    if redis_get_admin_tasks.is_none(){

                        match Task::get_all_admin(owner_id.to_owned(), connection).await{
                            Ok(admin_tasks) => {
    
                                /* chache the response for the next request */
                                let rc_data = serde_json::to_string(&admin_tasks).unwrap();
                                let get_admin_tasks_key = format!("get_admin_tasks_{:?}", owner_id.to_owned());
                                let _: () = redis_conn.set(get_admin_tasks_key.as_str(), rc_data).await.unwrap();
    
                                resp!{
                                    Vec<TaskData>, // the data type
                                    admin_tasks, // response data
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

                    } else{

                        resp!{
                            Vec<TaskData>, // the data type
                            redis_get_admin_tasks.unwrap(), // response data
                            FETCHED, // response message
                            StatusCode::OK, // status code
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
    context_path = "/admin",
    responses(
        (status=200, description="Fetched Successfully", body=[UserTaskData]),
        (status=404, description="User Not Found", body=i32), // not found by id
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
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[get("/get-users-tasks")]
async fn get_users_tasks(
        req: HttpRequest,   
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match UserTask::tasks_per_user(connection).await{
                        Ok(all_users_tasks) => {

                            resp!{
                                Vec<UserTaskData>, // the data type
                                all_users_tasks, // response data
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
    context_path = "/admin",
    request_body = Keys,
    responses(
        (status=200, description="Updated Successfully", body=[u8]),
        (status=404, description="User Not Found", body=i32), // not found by id
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
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[post("/add-twitter-account")]
async fn add_twitter_account(
        req: HttpRequest,   
        new_account: web::Json<Keys>,
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, Some(UserRole::Admin), connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    let file_open = std::fs::File::open("twitter-accounts.json");
                    let Ok(file) = file_open else{

                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: &file_open.unwrap_err().to_string(),
                            status: 500
                        };
                        return 
                            Ok(
                                HttpResponse::InternalServerError().json(resp)
                            );

                    };

                   
                    let accounts_value: serde_json::Value = serde_json::from_reader(file).unwrap();
                    let accounts_json_string = serde_json::to_string(&accounts_value).unwrap(); // reader in serde_json::from_reader can be a tokio tcp stream, a file or a buffer that contains the u8 bytes
                    let mut twitter = serde_json::from_str::<misc::TwitterAccounts>(&accounts_json_string).unwrap(); 
                    let twitter_accounts = &mut twitter.keys;

                    /* twitter var will be mutated too since twitter_accounts is a mutable reference to twitter */
                    twitter_accounts.push(new_account.to_owned());


                    /* saving the twitter back to the file */
                    let json_string_twitter = serde_json::to_string_pretty(&twitter).unwrap();
                    let updated_twitter_accounts_buffer = json_string_twitter.as_bytes();

                    /* overwriting the file */
                    match std::fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open("twitter-accounts.json"){
                        Ok(mut file) => {
                            match file.write(updated_twitter_accounts_buffer){
                                Ok(bytes) => { /* written bytes */
        
                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        TWITTER_KEYS_ADDED, // response message
                                        StatusCode::OK, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
        
                                },
                                Err(e) => {
                                    
                                    resp!{
                                        &[u8], // the data type
                                        &[], // response data
                                        &e.to_string(), // response message
                                        StatusCode::INTERNAL_SERVER_ERROR, // status code
                                        None::<Cookie<'_>>, // cookie
                                    }
        
                                }
                            }
                        },
                        Err(e) => {

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &e.to_string(), // response message
                                StatusCode::INTERNAL_SERVER_ERROR, // status code
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





pub mod exports{
    pub use super::reveal_role;   
    pub use super::login;
    pub use super::register_new_user;
    pub use super::register_new_task; 
    pub use super::delete_task;
    pub use super::edit_task;
    pub use super::edit_user;
    pub use super::delete_user;
    pub use super::get_users;
    pub use super::get_admin_tasks;
    pub use super::get_users_tasks;
    pub use super::add_twitter_account;
}