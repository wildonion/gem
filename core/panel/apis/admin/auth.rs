


pub use super::*;



/* 
    >-------------------------------------------------------------------------
    There are access and refresh tokens in cookie response in form of 
        /accesstoken={access_token:}&accesstoken_time={time_hash_hex_string:}&refrestoken={refresh_token:} 
    once the access token gets expired we can pass refresh token into 
    the request header in place of access token to get a new set of 
    keys on behalf of user, instead of redirecting client to the 
    login page again.
*/
#[post("/login")]
pub(self) async fn login(
        req: HttpRequest, 
        login_info: web::Json<LoginInfoRequest>, 
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {
   
    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();

    // logic to prevent bruteforce attacks regardless of who is trying to send a request
    let mut redis_conn = redis_client.clone().get_async_connection().await.unwrap();
    let chill_key = format!("chill_admin_login");
    let login_attempts = format!("admin_login_attempts");
    let get_chill_key: RedisResult<u8> = redis_conn.get(chill_key.clone()).await;
    let mut attempts = match get_chill_key{
        Ok(val) => {
            resp!{
                &[u8], // the data type
                &[], // response data
                &format!("Chill For 5 Mins"), // response message
                StatusCode::NOT_ACCEPTABLE, // status code
                None::<Cookie<'_>>, // cookie
            }
        },
        Err(e) => {
            let get_redis_login_attempts: RedisResult<u8> = redis_conn.get(login_attempts.clone()).await;
            let redis_login_attempts = match get_redis_login_attempts{
                Ok(attempts) => attempts,
                Err(e) => 0 // no attempts yet
            };
            redis_login_attempts
        }
    };

    if attempts == 3{
        // chill 5 mins
        let _: () = redis_conn.set_ex(chill_key.clone(), 0, 300).await.unwrap();
        // reject request
        resp!{
            &[u8], // the data type
            &[], // response data
            &format!("You've Reached Your Max Attempts, Chill 5 Mins"), // response message
            StatusCode::NOT_ACCEPTABLE, // status code
            None::<Cookie<'_>>, // cookie
        }
    }
    attempts += 1;


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            let check_refresh_token = req.check_refresh_token(connection);
            let Ok(user) = check_refresh_token else{
                let err_resp = check_refresh_token.unwrap_err();
                return err_resp;
            };

            if user.id != 0{

                info!("generating new set of token with refresh token for admin with id: {}", user.id);
                return user.get_user_data_response_with_cookie(&login_info.device_id, redis_client.clone(), redis_actix_actor, connection).await.unwrap();

            }

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
                                
                                // 1 failed attempt 
                                let _: () = redis_conn.set(login_attempts, attempts).await.unwrap();

                                resp!{
                                    String, // the data type
                                    user_name.to_owned(), // response data
                                    WRONG_PASSWORD, // response message
                                    StatusCode::FORBIDDEN, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
                            }
        
                            user.get_user_data_response_with_cookie(&login_info.device_id, redis_client.clone(), redis_actix_actor, connection).await.unwrap()
        
                        },
                        _ => {
                            
                            // 1 failed attempt 
                            let _: () = redis_conn.set(login_attempts, attempts).await.unwrap();

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


pub mod exports{
    pub use super::login;
}