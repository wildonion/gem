


pub use super::*;


#[post("/add-twitter-account")]
#[passport(admin)]
pub(self) async fn add_twitter_account(
        req: HttpRequest,   
        new_account: web::Json<Keys>,
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
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    let file_open = std::fs::File::open("twitter-accounts.json");
                    let Ok(file) = file_open else{

                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: &file_open.unwrap_err().to_string(),
                            status: 500,
                            is_error: true
                        };
                        return 
                            Ok(
                                HttpResponse::InternalServerError().json(resp)
                            );

                    };

                   
                    let accounts_value: serde_json::Value = serde_json::from_reader(file).unwrap(); /* converting the file buffer into serde Value to build the struct from its String */
                    let accounts_json_string = serde_json::to_string(&accounts_value).unwrap(); // reader in serde_json::from_reader can be a tokio tcp stream, a file or a buffer that contains the u8 bytes
                    let mut twitter = serde_json::from_str::<helpers::misc::TwitterAccounts>(&accounts_json_string).unwrap(); 
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

    pub use super::add_twitter_account;
}