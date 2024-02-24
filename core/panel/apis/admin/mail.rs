


pub use super::*;


#[post("/mail/send/to")]
#[passport(admin)]
pub(self) async fn send_mail(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    users_mail_info: web::Json<UserMailInfos>
) -> PanelHttpResponse{
    
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


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;
                    let users_mail_info = users_mail_info.to_owned();

                    // https://users.rust-lang.org/t/crate-lettre-doesnt-support-multiple-recipients/80813/11
                    let mut mails = vec![];
                    for uid in users_mail_info.ids{
                        let get_user = User::find_by_id(uid, connection).await;
                        if get_user.is_err(){
                            continue;
                        }
                        let get_user_mail = get_user.unwrap().mail;
                        if get_user_mail.is_none(){
                            continue;
                        }
                        mails.push(get_user_mail.unwrap());
                    }

                    if mails.is_empty(){
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            NOT_VERIFIED_USERS, // response message
                            StatusCode::NOT_ACCEPTABLE, // status code
                            None::<Cookie<'_>>, // cookie
                        }
                    }

                    let batch_send = mailreq::send_batch(
                        APP_NAME, 
                        mails, 
                        &users_mail_info.body, 
                        &users_mail_info.subject
                    ).await;

                    let Ok(batch_send_res) = batch_send else{
                        let err_resp = batch_send.unwrap_err();
                        return err_resp;
                    };

                    resp!{
                        &[u8], // the data type
                        &[], // response data
                        BATCH_MAIL_SENT, // response message
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
    pub use super::send_mail;
}