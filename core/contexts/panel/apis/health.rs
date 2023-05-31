


use crate::*;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::models::users::*;
use crate::schema::users::dsl::*;
use crate::schema::users;



/*
     -------------------------------
    |          SWAGGER DOCS
    | ------------------------------
    |
    |

*/



/*
     ------------------------
    |        SCHEMAS
    | ------------------------
    |
    |

*/
#[derive(Serialize, Deserialize, Clone)]
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
#[get("/check-server")]
async fn index(
        req: HttpRequest, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

        let iam_healthy = Health{
            status: "🥞 Alive".to_string()
        };
    
        resp!{
            Health, //// the data type
            iam_healthy, //// response data
            IAM_HEALTHY, //// response message
            StatusCode::OK, //// status code
            None::<Cookie<'_>>,
        }

}

#[get("/check-token")]
async fn check_token(
        req: HttpRequest, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            
            match User::passport(req, Some(UserRole::Admin), connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    
                    let single_user = users
                            .filter(id.eq(_id))
                            .select((id, username, twitter_username, 
                                    facebook_username, discord_username,
                                    wallet_address, user_role, token_time,
                                    last_login, created_at, updated_at))
                            .first::<FetchUser>(connection);


                    let Ok(user) = single_user else{
                        resp!{
                            i32, //// the data type
                            _id, //// response data
                            USER_NOT_FOUND, //// response message
                            StatusCode::NOT_FOUND, //// status code
                            None::<Cookie<'_>>,
                        } 
                    };

                    resp!{
                        FetchUser, //// the data type
                        user, //// response data
                        FETCHED, //// response message
                        StatusCode::OK, //// status code
                        None::<Cookie<'_>>,
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
                None::<Cookie<'_>>,
            }
        }
    }

}

#[post("/logout")]
async fn logout(
        req: HttpRequest, 
        redis_client: web::Data<RedisClient>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {


    let storage = storage.as_ref().to_owned();
    let redis_conn = redis_client.get_async_connection().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            match User::passport(req, None, connection){
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match User::logout(_id, connection).await{
                        Ok(_) => {
                            
                            resp!{
                                &[u8], //// the data type
                                &[], //// response data
                                LOGOUT, //// response message
                                StatusCode::OK, //// status code
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
    pub use super::index;
    pub use super::check_token;
    pub use super::logout;
}