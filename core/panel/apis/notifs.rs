




use crate::*;
use crate::events::redis::RedisSubscription;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::events::{
    ws::notifs::role::{RoleNotifServer, UpdateNotifRoom, SendNotif},
    ws::session::WsNotifSession,
};
use actix::prelude::*;



/* 

    client must be connect to this route then we have a full duplex communication channel 
    also there is a path in this route which is the event room that must be connected to 
    and is the name of the notification room which can be one of the following:
    
        `tasks`, 
        `task-verification-responses`, 
        `twitter-bot-response`, 
        `ecq-{event_id}`, 
        `mmr-{event_id}`,
        `reveal-role-{event_id}`

    users after participating in an event we'll redirect them to the event page after that 
    the client must call this route with the passed in event id like so: /reveal-role-{event_id}
    in which a new full duplex ws connection will be stablished to this server which will send 
    the subscribed redis topics to the channel that contains event peers.s 

*/
#[get("/{user_id}/{notif_room}")]
async fn notif_subs(
    req: HttpRequest, 
    stream: web::Payload, 
    route_paths: web::Path<(String, String)>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ws_role_notif_server: web::Data<Addr<RoleNotifServer>>,
    builtin_redis_actor: web::Data<Addr<RedisSubscription>>,
) -> Result<HttpResponse, actix_web::Error> {


    let io_buffer_size = env::var("IO_BUFFER_SIZE").expect("⚠️ no io buffer size variable set").parse::<u32>().unwrap() as usize;
    let redis_password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());


    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_async_pubsubconn = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();
    let redis_actor = storage.as_ref().clone().unwrap().get_redis_actor().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            let user_id = route_paths.0.to_owned();
            let notif_room = route_paths.1.to_owned();
            let ws_role_notif_actor_address = ws_role_notif_server.get_ref().to_owned();
            
            let get_messeses = redis_async_pubsubconn.subscribe(&notif_room).await;
            let Ok(mut messages) = get_messeses else{
                
                let e = get_messeses.unwrap_err();
                resp!{
                    &[u8], // the data type
                    &[], // response data
                    &e.to_string(), // response message
                    StatusCode::REQUEST_TIMEOUT, // status code
                    None::<Cookie<'_>>, // cookie
                }
            };

            /* 
                sending the update message to mutate the notif room before starting the session actor
                also make sure that the RoleNotifServer actor is already started when we're here
                otherwise by calling this route every time a new actor will be started and the previous 
                state will be lost.
            */
            let update_notif_room_result = ws_role_notif_actor_address
                .send(UpdateNotifRoom(notif_room.clone()))
                .await;

            let Ok(_) = update_notif_room_result else{
            
                resp!{
                    &[u8], // the data type
                    &[], // response data
                    WS_UPDATE_NOTIF_ROOM_ISSUE, // response message
                    StatusCode::REQUEST_TIMEOUT, // status code
                    None::<Cookie<'_>>, // cookie
                }
            };


            /* 
            
                ---------------------------------------------------------------------------------
                   SUBSCRIBING TO THE PASSED IN EVENT TOPIC USING ASYNC REDIS PUBSUB CONNECTION
                ---------------------------------------------------------------------------------
                get_messeses must be mutable so we can read from it thus the we have to borrow it mutably in each 
                iteration which can be done by calling as_mut() on the get_messeses, also since we're using the 
                get_messeses in a while loop thus in each iteration we must borrow the get_messeses using as_mut() method 
                in order to prevent its ownership from moving in each iteration,

                let get_messeses = redis_async_pubsubconn.subscribe(&notif_room).await;
                tokio::spawn(async move{

                    while let Some(message) = get_messeses.as_mut().unwrap().next().await{ /* iterating through the msg streams as they're coming to the stream channel and are not None */
                        
                        let resp_val = message.unwrap();
                        let message = String::from_resp(resp_val).unwrap();
                        notif_sender.send(message).await;

                    }
                    
                });


            */
    
            // tokio::spawn(async move{

            //     while let Some(message) = messages.next().await{ /* iterating through the msg streams as they're coming to the stream channel and are not None */
                    
            //         let resp_val = message.unwrap();
            //         let message = String::from_resp(resp_val).unwrap();
            //         notif_sender.send(message).await;

            //     }
                
            // });


            /* 
                --------------------------------
                 STARTING THE WEBSOCKET SESSION
                --------------------------------

            */

            let resp = if notif_room.clone().starts_with("reveal-role-"){
                /* starting ws connection for the passed in peer and the notif room */
                ws::start(
                    WsNotifSession{
                        id: 0,
                        hb: Instant::now(),
                        peer_name: Some(user_id),
                        notif_room,
                        ws_role_notif_actor_address
                    }, 
                    &req, 
                    stream
                )
            } else if notif_room.starts_with("ecq-"){

                todo!()

            } else if notif_room.starts_with("mmr-"){

                todo!()

            } else if notif_room.starts_with("twitter-bot-response"){

                todo!()

            } else if notif_room.starts_with("task-verification-responses"){

                todo!()

            } else if notif_room.starts_with("tasks"){

                todo!();

            } else{

                resp!{
                    &[u8], // the data type
                    &[], // response data
                    WS_INVALID_PATH, // response message
                    StatusCode::NOT_ACCEPTABLE, // status code
                    None::<Cookie<'_>>, // cookie
                }

            };

            /* 
                sending the ws connection response, we'll have either a successful 
                full duplex connection or response error 
            */
            resp

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
    pub use super::notif_subs;
}