




use crate::*;
use crate::events::redis::RedisSubscription;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::events::{
    ws::notifs::role::{RoleNotifServer, UpdateNotifRoom},
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
    builtin_redis_actor: web::Data<Addr<RedisSubscription>>
) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_async_pubsubconn = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();
    let builtin_redis_actor = builtin_redis_actor.get_ref().clone();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();
            let user_id = route_paths.0.to_owned();
            let notif_room = route_paths.1.to_owned();
            let notif_room_str = string_to_static_str(notif_room.clone());
            let ws_role_notif_actor_address = ws_role_notif_server.get_ref().to_owned();

            /* 
                sending the update message to mutate the notif room before starting the session actor
                also make sure that the RoleNotifServer actor is already started when we're here
                otherwise by calling this route every time a new actor will be started and the previous 
                state will be lost.
            */
            let update_notif_room_result = ws_role_notif_actor_address
                .send(UpdateNotifRoom{
                    notif_room: notif_room_str, 
                    builtin_redis_actor, 
                    peer_name: user_id.clone()
                })
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
                        notif_room: notif_room_str,
                        ws_role_notif_actor_address,
                        redis_async_pubsubconn,
                        is_subscription_interval_started: false
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