

pub use super::*;


#[get("/get-user-wallet-info/{identifier}")]
pub(self) async fn get_user_wallet_info(
        req: HttpRequest,   
        user_identifier: web::Path<String>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            match User::fetch_wallet_by_username_or_mail_or_scid(&user_identifier.to_owned(), connection).await{

                Ok(user_info) => {

                    resp!{
                        UserWalletInfoResponse, // the data type
                        user_info, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
                    }

                },
                Err(resp) => {
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


#[get("/get-users-wallet-info/")]
pub(self) async fn get_users_wallet_info(
        req: HttpRequest,   
        limit: web::Query<Limit>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            match User::fetch_all_users_wallet_info(limit, connection).await{

                Ok(users_info) => {

                    resp!{
                        Vec<Option<UserWalletInfoResponseWithBalance>>, // the data type
                        {
                            let mut users_info = users_info
                                .into_iter()
                                .map(|user|{
                                    if user.username == "adminy" || user.username == "devdevy"{
                                        None
                                    } else{
                                        Some(user)
                                    }
                                })
                                .collect::<Vec<Option<UserWalletInfoResponseWithBalance>>>();
                            users_info.retain(|user| user.is_some());
                            users_info
                            
                        }, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
                    }

                },
                Err(resp) => {
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
    pub use super::get_user_wallet_info;
    pub use super::get_users_wallet_info;
}