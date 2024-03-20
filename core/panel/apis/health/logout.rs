

pub use super::*;


#[post("/logout")]
#[passport(admin, user, dev)]
pub(self) async fn logout(
        req: HttpRequest,  
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {


    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

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

            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    let _token_time = token_data.token_time;

                    /* 
                        🔐 logout supports also for jwt only, it sets the token time field 
                        inside the users table related to the logged in user to 0, this will
                        also remove the jwt field inside the users_logins table related to 
                        the user device id, means that logging out from a device won't logout
                        user completely from the app it only logs him out from only the device
                        he is sent the request
                    */
                    match User::logout(_id, _token_time, redis_client.to_owned(), redis_actix_actor, connection).await{
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


pub mod exports{
    pub use super::logout;
}