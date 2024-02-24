


pub use super::*;

/* >_______________________________________________________________________________
    this api must gets called by player with his conse rendezvous hyper server JWT 
    passed in to the request header 
    _______________________________________________________________________________
*/
#[post("/rendezvous/player/{player_id}/upload/avatar")]
pub(self) async fn upload_rendezvous_player_avatar(
    req: HttpRequest, 
    player_id: web::Path<String>, // mongodb objectid
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    mut img: Multipart, /* form-data implementation to receive stream of byte fields */
    ) -> PanelHttpResponse{


        if let Some(header_value) = req.headers().get("Authorization"){

            let token = header_value.to_str().unwrap();
            
            /*
                @params: 
                    - @token          → JWT
    
                note that this token must be taken from the conse rendezvous hyper server
            */
            match rendezvous_passport!{ token }{
                true => {
    
                    // -------------------------------------------------------------------------------------
                    // ------------------------------- ACCESS GRANTED REGION -------------------------------
                    // -------------------------------------------------------------------------------------
                    /*  
                        this route requires the player access token from the conse 
                        rendezvous hyper server to update avatar image, we'll send a request
                        to the conse rendezvous hyper server to verify the passed in JWT of the
                        player and if it was verified we'll allow the user to update the image
                    */
    
                    let storage = app_state.app_sotrage.as_ref().to_owned();
                    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                    let player_id_img_key = format!("{player_id:}-img");

                    let get_redis_conn = redis_client.get_async_connection().await;
                    let Ok(mut redis_conn) = get_redis_conn else{

                        let redis_get_conn_error = get_redis_conn.err().unwrap();
                        let redis_get_conn_error_string = redis_get_conn_error.to_string();
                        use helpers::error::{ErrorKind, StorageError::Redis, PanelError};
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

                    let img = std::sync::Arc::new(tokio::sync::Mutex::new(img));
                    let get_player_img_path = multipartreq::store_file(
                        AVATAR_UPLOAD_PATH, &format!("{}", player_id), 
                        "player", 
                        img).await;
                    let Ok(player_img_filepath) = get_player_img_path else{
            
                        let err_res = get_player_img_path.unwrap_err();
                        return err_res;
                    };

                    
                    /* 
                        writing the avatar image filename to redis ram, by doing this we can 
                        retrieve the value from redis in conse hyper rendezvous server when we call 
                        the check token api
                    */
                    let _: () = redis_conn.set(player_id_img_key.as_str(), player_img_filepath.as_str()).await.unwrap();
                
                    resp!{
                        &[u8], // the date type
                        &[], // the data itself
                        RENDEZVOUS_PLAYER_AVATAR_IMG_UPDATED, // response message
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

pub mod exports{
    pub use super::upload_rendezvous_player_avatar;
}