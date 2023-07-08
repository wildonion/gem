




use crate::*;
use crate::events::redis::Subscribe;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::events::ws::notifs::role::{RoleNotifServer, UpdateNotifRoom, NotifySessionsWithRedisSubscription};
use crate::events::ws::session::WsNotifSession;
use crate::events::redis::RedisSubscription;
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
    session: Session,
    stream: web::Payload, 
    route_paths: web::Path<(String, String)>,
    storage: web::Data<Option<Arc<Storage>>>, // db shared state data
    redis_actor: web::Data<Addr<RedisSubscription>>,
    ws_role_notif_server: web::Data<Addr<RoleNotifServer>>,
) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            let user_id = route_paths.0.to_owned();
            let notif_room = route_paths.1.to_owned();
            let ws_role_notif_actor_address = ws_role_notif_server.get_ref().to_owned();

            /* 
                sending the update message to mutate the notif room before starting the session actor
                
                ----- make sure that the RoleNotifServer actor is already started when we're here
                ----- otherwise by calling this route every time a new actor will be started and 
                ----- the previous state will be lost.
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


            /* sending subscribe message to redis actor to run the subscription interval for the passed in notif room */
            let redis_subscribe_result = redis_actor
                .send(Subscribe{notif_room: notif_room.clone()})
                .await;

            let Ok(_) = redis_subscribe_result else{
        
                resp!{
                    &[u8], // the data type
                    &[], // response data
                    WS_SUBSCRIPTION_INTERVAL_ISSUE, // response message
                    StatusCode::REQUEST_TIMEOUT, // status code
                    None::<Cookie<'_>>, // cookie
                }
            };


            let get_conn = redis_client.get_connection();
            let Ok(mut conn) = get_conn else{

                /* custom error handler */
                use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                let conn_err = get_conn.err().unwrap();
                let msg_content = [0u8; 32];
                let error_content = &conn_err.to_string();
                msg_content.to_vec().extend_from_slice(error_content.as_bytes());

                let redis_error_code = conn_err.code().unwrap().parse::<u16>().unwrap();
                let error_instance = PanelError::new(redis_error_code, msg_content, ErrorKind::Storage(Redis(conn_err)));
                let error_buffer = error_instance.write_sync(); /* write to file also returns the full filled buffer */
                
                panic!("panicked at redis get sync connection at {}", chrono::Local::now());

            };


            let resp = if notif_room.clone().starts_with("reveal-role-"){
                /* starting ws connection for the passed in peer and the notif room */
                ws::start(
                    WsNotifSession{
                        id: 0,
                        hb: Instant::now(),
                        peer_name: Some(user_id),
                        notif_room,
                        app_storage: storage.clone(),
                        redis_actor: redis_actor.get_ref().clone(),
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

            /* sending the ws connection response, we'll have either a successful full duplex connection or response error */
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