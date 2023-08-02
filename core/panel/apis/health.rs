


use crate::*;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::models::users::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::models::{tasks::*, users_tasks::*};



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
        index,
        check_token,
        get_tasks,
        logout,
    ),
    components(
        schemas(
            UserData,
            Health,
            TaskData
        )
    ),
    tags(
        (name = "crate::apis::health", description = "Tasks Verification Endpoints")
    ),
    info(
        title = "Health Access APIs"
    ),
    modifiers(&SecurityAddon),
)]
pub struct HealthApiDoc;
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
    |        SCHEMAS
    | ------------------------
    |
    |

*/
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct Health{
    pub status: String,
}


/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/
#[utoipa::path(
    context_path = "/health",
    responses(
        (status=200, description="I'm Alive", body=Health),
    ),
    tag = "crate::apis::health",
)]
#[get("/check-server")]
#[passport(admin, user, dev)]
async fn index(
        req: HttpRequest,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

        let iam_healthy = Health{
            status: "🥞 Alive".to_string()
        };
    
        resp!{
            Health, // the data type
            iam_healthy, // response data
            IAM_HEALTHY, // response message
            StatusCode::OK, // status code
            None::<Cookie<'_>>,
        }

}

#[utoipa::path(
    context_path = "/health",
    responses(
        (status=200, description="Fetched Successfully", body=UserData),
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
    tag = "crate::apis::health",
    security(
        ("jwt" = [])
    )
)]
#[get("/check-token")]
#[passport(admin, user, dev)]
async fn check_token(
        req: HttpRequest,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    

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


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;


                    /* create a response cacher using redis */
                    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
                    let check_token_key = format!("check_token_{_id:}");
                    let redis_result_check_token: RedisResult<String> = redis_conn.get(check_token_key.as_str()).await;
                    let mut redis_check_token = match redis_result_check_token{
                        Ok(data) => {
                            let rc_data = serde_json::from_str::<HashMap<i32, UserData>>(data.as_str()).unwrap();
                            rc_data
                        },
                        Err(e) => {
                            let empty_check_token = HashMap::<i32, UserData>::new();
                            let rc_data = serde_json::to_string(&empty_check_token).unwrap();
                            let check_token_key = format!("check_token_{_id:}");
                            let _: () = redis_conn.set(check_token_key.as_str(), rc_data).await.unwrap();
                            HashMap::new()
                        }
                    };
    
                    if let Some(user_data) = redis_check_token.get(&_id){

                        /* sending response from redis */
                        resp!{
                            UserData, // the data type
                            user_data.clone(), // response data
                            FETCHED, // response message
                            StatusCode::OK, // status code
                            None::<Cookie<'_>>,
                        }

                    /* if there was no cache simply we'll build a new one */
                    } else{

                        let single_user = users
                            .filter(id.eq(_id))
                            .select((id, username, activity_code, twitter_username, 
                                    facebook_username, discord_username,
                                    wallet_address, user_role, token_time,
                                    last_login, created_at, updated_at))
                            .first::<FetchUser>(connection);


                        let Ok(user) = single_user else{
                            resp!{
                                i32, // the data type
                                _id, // response data
                                USER_NOT_FOUND, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>,
                            } 
                        };

                        let user_data = UserData { 
                            id: user.id, 
                            username: user.username, 
                            activity_code: user.activity_code,
                            twitter_username: user.twitter_username, 
                            facebook_username: user.facebook_username, 
                            discord_username: user.discord_username, 
                            wallet_address: user.wallet_address, 
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


                        /* chache the response for the next request for this user */
                        let check_token_key = format!("check_token_{_id:}");
                        redis_check_token.insert(_id, user_data.clone());
                        let rc_data = serde_json::to_string(&redis_check_token).unwrap();
                        let _: () = redis_conn.set(check_token_key.as_str(), rc_data).await.unwrap();

                        /* sending pg response */
                        resp!{
                            UserData, // the data type
                            user_data, // response data
                            FETCHED, // response message
                            StatusCode::OK, // status code
                            None::<Cookie<'_>>,
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
                None::<Cookie<'_>>,
            }
        }
    }

}

#[utoipa::path(
    context_path = "/health",
    responses(
        (status=200, description="Loggedout Successfully", body=UserData),
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
    tag = "crate::apis::health",
    security(
        ("jwt" = [])
    )
)]
#[post("/logout")]
#[passport(admin, user, dev)]
async fn logout(
        req: HttpRequest,  
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

            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* 
                        🔐 logout supports also for jwt only, it sets the token time field 
                        inside the users table related to the logged in user to 0, this wiill 
                        be checked inside the **passport** function to see that the token time 
                        inside the passed in jwt to the request header must be the one 
                        inside the users table
                    */
                    match User::logout(_id, connection).await{
                        Ok(_) => {
                            
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                LOGOUT, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>,
                            }
        
                        },
                        Err(resp) => {
            
                            /* DIESEL UPDATE ERROR RESPONSE */
                            resp
        
                        }
                    }

                },
                Err(resp) => {
                    
                    /* 
                        🥝 based on the flow response can be one of the following:
                        
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
    context_path = "/health",
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
    tag = "crate::apis::health",
    security(
        ("jwt" = [])
    )
)]
#[get("/get-tasks")]
#[passport(admin, user, dev)]
async fn get_tasks(
        req: HttpRequest,  
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
                    
                    match Task::get_all(connection).await{
                        Ok(all_tasks) => {

                            resp!{
                                Vec<TaskData>, // the data type
                                all_tasks, // response data
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

pub mod exports{
    pub use super::index;
    pub use super::check_token;
    pub use super::logout;
    pub use super::get_tasks;
}