


pub use super::*;


#[post("/notif/register/reveal-role/{event_id}")]
pub(self) async fn reveal_role(
        req: HttpRequest, 
        event_id: web::Path<String>, // mongodb objectid
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    /* 
        reveal role event and webhook handler to call the reveal role api of conse rendezvous
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

            note that this token must be taken from the conse rendezvous hyper server
        */
        match rendezvous_passport!{ token }{
            true => {

                // -------------------------------------------------------------------------------------
                // ------------------------------- ACCESS GRANTED REGION -------------------------------
                // -------------------------------------------------------------------------------------

                let storage = storage.as_ref().to_owned();
                let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();
                
                let host = env::var("HOST").expect("⚠️ no host variable set");
                let port = env::var("RENDEZVOUS_PORT").expect("⚠️ no port variable set");
                let reveal_api = format!("http://{}:{}/event/reveal/roles", host, port);
                
                let mut revealed = events::publishers::role::Reveal::default();
                let mut map = HashMap::new();
                map.insert("_id", event_id.to_owned());

                match storage.clone().unwrap().get_pgdb().await{
                    Some(pg_pool) => {
                        
                        info!("📥 sending reveal role request to the conse rendezvous hyper server at {} for event [{}]", chrono::Local::now().timestamp_nanos_opt().unwrap(), event_id);

                        /* calling rveal role API of the rendezvous hyper server to get the players' roles */
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

                        /* if we're here means that the conse rendezvous hyper server is up and we got a response from it */
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


#[post("/rendezvous/event/{event_id}/upload/img")]
pub(self) async fn update_rendezvous_event_img(
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
    
                note that this token must be taken from the conse rendezvous hyper server
            */
            match rendezvous_passport!{ token }{
                true => {
    
                    // -------------------------------------------------------------------------------------
                    // ------------------------------- ACCESS GRANTED REGION -------------------------------
                    // -------------------------------------------------------------------------------------
                    /*  
                        this route requires the admin or god access token from the conse 
                        rendezvous hyper server to update an event image, we'll send a request
                        to the conse rendezvous hyper server to verify the passed in JWT of the
                        admin and it was verified we'll allow the user to update the image
                    */
    
                    let storage = storage.as_ref().to_owned();
                    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
                    let event_id_img_key = format!("{event_id:}-img");

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
                    let get_event_img_path = multipartreq::store_file(
                        EVENT_UPLOAD_PATH, &format!("{}", event_id), 
                        "event", 
                        img).await;
                    let Ok(event_img_filepath) = get_event_img_path else{
            
                        let err_res = get_event_img_path.unwrap_err();
                        return err_res;
                    };


                    /* 
                        writing the event image filename to redis ram, by doing this we can 
                        retrieve the value from redis in conse hyper rendezvous server when we call 
                        the get event info api
                    */
                    let _: () = redis_conn.set(event_id_img_key.as_str(), event_img_filepath.as_str()).await.unwrap();
                
                    resp!{
                        &[u8], // the date type
                        &[], // the data itself
                        RENDEZVOUS_EVENT_IMG_UPDATED, // response message
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
    pub use super::reveal_role;
    pub use super::update_rendezvous_event_img;
}