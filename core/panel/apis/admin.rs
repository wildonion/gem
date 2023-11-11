






use actix_web::HttpMessage;
use futures_util::TryStreamExt; /* TryStreamExt can be used to call try_next() on future object */
use mongodb::bson::oid::ObjectId;
use crate::*;
use crate::events::publishers::role::{PlayerRoleInfo, Reveal};
use crate::models::users_checkouts::{UserCheckout, UserCheckoutData};
use crate::models::users_deposits::{UserDeposit, UserDepositData};
use crate::models::users_withdrawals::{UserWithdrawal, UserWithdrawalData};
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use s3::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use crate::schema::users_tasks::dsl::*;
use crate::schema::users_tasks;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};


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
        add_twitter_account,
        get_all_users_withdrawals,
        get_all_users_deposits
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
            EditTaskRequest,
            UserDepositData,
            UserWithdrawalData
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
    ) -> PanelHttpResponse {

    /* 
        reveal role event and webhook handler to call the reveal role api of conse mafia
        hyper server then publish the roles into the redis pubsub channel, cause we'll
        subscribe to the roles in ws server and notify each session about his role.  
        webhook means once an event gets triggered an api call will be invoked to 
        notify (it's like a notification to the server) server about the event happend 
        as a result of handling another process in some where like a payment result in 
        which server subscribes to incoming event type and can publish it to redispubsub 
        so other app, threads and scopes can also subscribe to it
    */

    if let Some(header_value) = req.headers().get("Authorization"){

        let token = header_value.to_str().unwrap();
        
        /*
            @params: 
                - @token          → JWT

            note that this token must be taken from the conse mafia hyper server
        */
        match mafia_passport!{ token }{
            true => {

                // -------------------------------------------------------------------------------------
                // ------------------------------- ACCESS GRANTED REGION -------------------------------
                // -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();
                
                let host = env::var("HOST").expect("⚠️ no host variable set");
                let port = env::var("MAFIA_PORT").expect("⚠️ no port variable set");
                let reveal_api = format!("http://{}:{}/event/reveal/roles", host, port);
                
                let mut revealed = events::publishers::role::Reveal::default();
                let mut map = HashMap::new();
                map.insert("_id", event_id.to_owned());

                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
                        
                        info!("📥 sending reveal role request to the conse mafia hyper server at {} for event [{}]", chrono::Local::now().timestamp_nanos_opt().unwrap(), event_id);

                        /* calling rveal role API of the mafia hyper server to get the players' roles */
                        let get_response_value = reqwest::Client::new()
                            .post(reveal_api.as_str())
                            .json(&map)
                            .header("Authorization", token)
                            .send()
                            .await;

                        let Ok(response_value) = get_response_value else{

                            let err = get_response_value.unwrap_err();
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &err.to_string(), // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }

                        };

                        /* if we're here means that the conse mafia hyper server is up and we got a response from it */
                        let response_value = response_value.json::<serde_json::Value>().await.unwrap();

                        let data = response_value.get("data");
                        if data.is_some(){

                            let players_field = data.unwrap().get("players");
                            let event_id_field = data.unwrap().get("_id");
                            
                            if players_field.is_some() && event_id_field.is_some(){

                                let players_rvealed_roles = players_field.unwrap().to_owned();
                                let event_id = event_id_field.unwrap().to_owned();
                                
                                let decoded_event_id = serde_json::from_value::<ObjectId>(event_id).unwrap();
                                let decoded_players = serde_json::from_value::<Vec<PlayerRoleInfo>>(players_rvealed_roles).unwrap();

                                revealed.players = decoded_players;
                                revealed.event_id = decoded_event_id.to_string();
                            
                            }
                        }

                        if revealed.players.is_empty(){
                            let resp_message_value = response_value.get("message").unwrap().to_owned();
                            let resp_message = serde_json::from_value::<String>(resp_message_value).unwrap();

                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                &resp_message, // response message
                                StatusCode::EXPECTATION_FAILED, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        }

                        let notif_room = revealed.event_id.clone();
                        let player_roles = revealed.players.clone();
                        let stringified_player_roles = serde_json::to_string(&player_roles).unwrap(); /* topic that is going to be published */
                        let channel = format!("reveal-role-{notif_room:}"); /* reveal roles notif channels start with reveal-role */
                        

                        /* publishing the revealed roles in the background asyncly until 1 subscriber gets subscribed to the channel */
                        Reveal::publish(
                            redis_actix_actor, 
                            &channel, 
                            &stringified_player_roles,
                            &notif_room
                        ).await;

                    
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            PUSH_NOTIF_SENT, // response message
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

#[post("/mafia/event/{event_id}/upload/img")]
async fn update_mafia_event_img(
    req: HttpRequest, 
        event_id: web::Path<String>, // mongodb objectid
        storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
        mut img: Multipart, /* form-data implementation to receive stream of byte fields */
    ) -> PanelHttpResponse{


        if let Some(header_value) = req.headers().get("Authorization"){

            let token = header_value.to_str().unwrap();
            
            /*
                @params: 
                    - @token          → JWT
    
                note that this token must be taken from the conse mafia hyper server
            */
            match mafia_passport!{ token }{
                true => {
    
                    // -------------------------------------------------------------------------------------
                    // ------------------------------- ACCESS GRANTED REGION -------------------------------
                    // -------------------------------------------------------------------------------------
                    /*  
                        this route requires the admin or god access token from the conse 
                        mafia hyper server to update an event image, we'll send a request
                        to the conse mafia hyper server to verify the passed in JWT of the
                        admin and it was verified we'll allow the user to update the image
                    */
    
                    let storage = storage.as_ref().to_owned();
                    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                    let event_id_img_key = format!("{event_id:}-img");

                    let get_redis_conn = redis_client.get_async_connection().await;
                    let Ok(mut redis_conn) = get_redis_conn else{

                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use error::{ErrorKind, StorageError::Redis, PanelError};
                        let error_content = redis_get_conn_error_string.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)), "update_event_img");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        resp!{
                            &[u8], // the date type
                            &[], // the data itself
                            &redis_get_conn_error_string, // response message
                            StatusCode::INTERNAL_SERVER_ERROR, // status code
                            None::<Cookie<'_>>, // cookie
                        }

                    };

                    let get_event_img_path = misc::store_file(
                        EVENT_UPLOAD_PATH, &format!("{}", event_id), 
                        "event", 
                        img).await;
                    let Ok(event_img_filepath) = get_event_img_path else{
            
                        let err_res = get_event_img_path.unwrap_err();
                        return err_res;
                    };


                    /* 
                        writing the event image filename to redis ram, by doing this we can 
                        retrieve the value from redis in conse hyper mafia server when we call 
                        the get event info api
                    */
                    let _: () = redis_conn.set(event_id_img_key.as_str(), event_img_filepath.as_str()).await.unwrap();
                
                    resp!{
                        &[u8], // the date type
                        &[], // the data itself
                        MAFIA_EVENT_IMG_UPDATED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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
    ) -> PanelHttpResponse {
   
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
                                region: user.region.clone(),
                                username: user.username.clone(),
                                bio: user.bio.clone(),
                                avatar: user.avatar.clone(),
                                banner: user.banner.clone(),
                                wallet_background: user.wallet_background.clone(),
                                activity_code: user.activity_code.clone(),
                                twitter_username: user.twitter_username.clone(),
                                facebook_username: user.facebook_username.clone(),
                                discord_username: user.discord_username.clone(),
                                identifier: user.identifier.clone(),
                                user_role: {
                                    match user.user_role.clone(){
                                        UserRole::Admin => "Admin".to_string(),
                                        UserRole::Dev => "User".to_string(),
                                        _ => "Dev".to_string(),
                                    }
                                },
                                token_time: updated_user.token_time,
                                balance: updated_user.balance,
                                last_login: { 
                                    if updated_user.last_login.is_some(){
                                        Some(updated_user.last_login.unwrap().to_string())
                                    } else{
                                        Some("".to_string())
                                    }
                                },
                                created_at: user.created_at.to_string(),
                                updated_at: updated_user.updated_at.to_string(),
                                mail: user.mail,
                                is_mail_verified: user.is_mail_verified,
                                is_phone_verified: user.is_phone_verified,
                                phone_number: user.phone_number,
                                paypal_id: user.paypal_id,
                                account_number: user.account_number,
                                device_id: user.device_id,
                                social_id: user.social_id,
                                cid: user.cid,
                                screen_cid: user.screen_cid,
                                snowflake_id: user.snowflake_id,
                                stars: user.stars
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
#[passport(admin)]
async fn register_new_user(
        req: HttpRequest,  
        new_user: web::Json<NewUserInfoRequest>, 
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

        
    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();


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
            /*
                we have to pass the cloned request object since it's borrowed to get its extensions 
                and if a type is behind a pointer we can't move it into a new scope we must either
                clone it or borrow it
            */
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* to_owned() will take the NewUserInfoRequest instance out of the web::Json*/
                    match User::insert_new_user(new_user.to_owned(), connection, redis_client).await{
                        Ok(_) => {

                            /* fetch all users again to get the newly one */
                            let get_all_users = User::get_all_without_limit(connection).await;
                            let Ok(all_users) = get_all_users else{
                                let resp = get_all_users.unwrap_err();
                                return resp;
                            };

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
#[passport(admin)]
async fn edit_user(
        req: HttpRequest, 
        new_user: web::Json<EditUserByAdminRequest>,  
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
            match User::passport(req, granted_role, connection).await{
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
#[passport(admin)]
async fn delete_user(
        req: HttpRequest, 
        doer_id: web::Path<i32>,  // doer is the user who do task
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
            match User::passport(req, granted_role, connection).await{
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
#[get("/get-users/")]
#[passport(admin)]
async fn get_users(
        req: HttpRequest,  
        limit: web::Query<Limit>,
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
            
            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match User::get_all(connection, limit).await{
                        Ok(all_users) => {

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
#[passport(admin)]
async fn register_new_task(
        req: HttpRequest, 
        new_task: web::Json<NewTaskRequest>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

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


            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match Task::insert(new_task.to_owned(), redis_client, connection).await{
                        Ok(_) => {

                            /* fetch all admin tasks again to get the newly one */
                            let get_all_admin_tasks = Task::get_all_admin_without_limit(new_task.admin_id, connection).await;
                            let Ok(all_admin_tasks) = get_all_admin_tasks else{
                                let resp = get_all_admin_tasks.unwrap_err();
                                return resp;
                            };

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
#[passport(admin)]
async fn delete_task(
        req: HttpRequest, 
        job_id: web::Path<i32>,  
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


            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, granted_role, connection).await{
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
#[passport(admin)]
async fn edit_task(
        req: HttpRequest, 
        new_task: web::Json<EditTaskRequest>,  
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


            /* --------- ONLY ADMIN CAN DO THIS LOGIC --------- */
            match User::passport(req, granted_role, connection).await{
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
#[get("/get-admin-tasks/{owner_id}/")]
#[passport(admin)]
async fn get_admin_tasks(
        req: HttpRequest, 
        owner_id: web::Path<i32>,  
        limit: web::Query<Limit>,
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match Task::get_all_admin(owner_id.to_owned(), limit, connection).await{
                        Ok(admin_tasks) => {
                            
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
#[get("/get-users-tasks/")]
#[passport(admin)]
async fn get_users_tasks(
        req: HttpRequest,
        limit: web::Query<Limit>,   
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
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    
                    match UserTask::tasks_per_user(limit, connection).await{
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
#[passport(admin)]
async fn add_twitter_account(
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
            match User::passport(req, granted_role, connection).await{
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

                   
                    let accounts_value: serde_json::Value = serde_json::from_reader(file).unwrap(); /* converting the file buffer into serde Value to build the struct from its String */
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


#[utoipa::path(
    context_path = "/admin",
    responses(
        (status=201, description="Fetched Successfully", body=Vec<UserDepositData>),
        (status=500, description="Internal Server Erros  Caused By Diesel or Redis", body=&[u8]),
    ),
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[get("/deposit/get/")]
#[passport(admin)]
async fn get_all_users_deposits(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match UserDeposit::get_all(limit, connection).await{
                        Ok(user_deposits) => {

                            resp!{
                                Vec<UserDepositData>, // the data type
                                user_deposits, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
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
        (status=201, description="Fetched Successfully", body=Vec<UserDepositData>),
        (status=500, description="Internal Server Erros  Caused By Diesel or Redis", body=&[u8]),
    ),
    tag = "crate::apis::admin",
    security(
        ("jwt" = [])
    )
)]
#[get("/withdraw/get/")]
#[passport(admin)]
async fn get_all_users_withdrawals(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match UserWithdrawal::get_all(limit, connection).await{
                        Ok(all_users_withdrawals) => {

                            resp!{
                                Vec<UserWithdrawalData>, // the data type
                                all_users_withdrawals, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
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

#[get("/checkouts/get/")]
#[passport(admin)]
async fn get_all_users_checkouts(
    req: HttpRequest,
    limit: web::Query<Limit>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    match UserCheckout::get_all(connection, limit).await{
                        Ok(all_users_checkouts) => {

                            resp!{
                                Vec<UserCheckoutData>, // the data type
                                all_users_checkouts, // response data
                                FETCHED, // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }


                        },
                        Err(resp) => {
                            /* 
                                🥝 response can be one of the following:
                                
                                - DIESEL INSERT ERROR RESPONSE
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

#[post("/listener/start/tcp/{port}")]
#[passport(admin)]
async fn start_tcp_server(
        req: HttpRequest,  
        port: web::Path<u16>,
        tcp_server_data: web::Json<TcpServerData>,
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let async_redis_client = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();


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

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            
            let tcp_server_data = tcp_server_data.to_owned();

            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req, granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let find_user_screen_cid = User::find_by_screen_cid(&tcp_server_data.from_cid, connection).await;
                        let Ok(user_info) = find_user_screen_cid else{
                            
                            resp!{
                                String, // the data type
                                tcp_server_data.from_cid, // response data
                                &USER_SCREEN_CID_NOT_FOUND, // response message
                                StatusCode::NOT_FOUND, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

                    let verification_res = wallet::evm::verify_signature(
                        user_info.screen_cid.unwrap(), 
                        &tcp_server_data.tx_signature,
                        &tcp_server_data.hash_data
                    ).await;
                    if verification_res.is_err(){
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            &INVALID_SIGNATURE, // response message
                            StatusCode::NOT_ACCEPTABLE, // status code
                            None::<Cookie<'_>>, // cookie
                        }
                    }
                    
                    /* ----------------------------------------- */
                    /* starting a tcp listener in the background */
                    /* ----------------------------------------- */

                    let bind_address = format!("0.0.0.0:{}", port.to_owned());
                    let mut api_listener = tokio::net::TcpListener::bind(bind_address.as_str()).await;
                    
                    if api_listener.is_err(){
                        resp!{
                            &[u8], // response data
                            &[], // response message
                            TCP_SERVER_ERROR,
                            StatusCode::EXPECTATION_FAILED, // status code
                            None::<Cookie<'_>>, // cookie
                        }
                    }

                    let api_listener = api_listener.unwrap();
                    info!("➔ 🚀 tcp listener is started at [{}]", bind_address);

                    tokio::spawn(async move{

                        while let Ok((mut api_streamer, addr)) = api_listener.accept().await{
                            info!("🍐 new peer connection: [{}]", addr);

                            let tcp_server_data = tcp_server_data.clone();

                            tokio::spawn(async move {
        
                                let mut buffer = vec![0; 1024];

                                while match api_streamer.read(&mut buffer).await {
                                    Ok(rcvd_bytes) if rcvd_bytes == 0 => return,
                                    Ok(rcvd_bytes) => {
                    
                                        let string_data = std::str::from_utf8(&buffer[..rcvd_bytes]).unwrap();
                                        info!("📺 received data from peer: {}", string_data);
                    
                                        let send_tcp_server_data = tcp_server_data.data.clone();
                                        if let Err(why) = api_streamer.write_all(&send_tcp_server_data.as_bytes()).await{
                                            error!("❌ failed to write to api_streamer; {}", why);
                                            return;
                                        } else{
                                            info!("🗃️ sent {}, wrote {} bytes to api_streamer", tcp_server_data.data.clone(), send_tcp_server_data.len());
                                            return;
                                        }
                                    
                                    },
                                    Err(e) => {
                                        error!("❌ failed to read from api_streamer; {:?}", e);
                                        return;
                                    }
                                    
                                }{}
                    
                            });
                        }{}
                        
                    });

                    resp!{
                        &[u8], // response data
                        &[], // response message
                        TCP_SERVER_STARTED,
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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
    // pub use super::request_ecq;  // `<---mafia jwt--->` mafia hyper server
    // pub use super::advertise_event;  // `<---mafia jwt--->` mafia hyper server
    pub use super::reveal_role; // `<---mafia jwt--->` mafia hyper server
    pub use super::login;
    pub use super::register_new_user;
    pub use super::register_new_task; 
    pub use super::delete_task;
    pub use super::edit_task;
    pub use super::edit_user;
    pub use super::delete_user;
    pub use super::add_twitter_account;
    pub use super::update_mafia_event_img; // `<---mafia jwt--->` mafia hyper server
    pub use super::start_tcp_server;
    pub use super::get_all_users_checkouts;
    pub use super::get_users;
    pub use super::get_admin_tasks;
    pub use super::get_users_tasks;
    pub use super::get_all_users_withdrawals;
    pub use super::get_all_users_deposits;
}