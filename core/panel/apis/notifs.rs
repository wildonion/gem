




use crate::*;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::events::{
    ws::notifs::role::{RoleNotifServer, UpdateNotifRoom, SendNotif},
    ws::session::WsNotifSession,
    redis::RedisSubscription
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
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (redis, postgres and mongodb)
    redis_actor: web::Data<Addr<RedisActor>>,
    async_redis: web::Data<PubsubConnection>,
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
            let redis_actor = redis_actor.get_ref().clone();

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

            
            let get_msg = async_redis.psubscribe(&notif_room).await;
            let Ok(mut subscribed_msgs) = get_msg else{
                
                resp!{
                    &[u8], // the data type
                    &[], // response data
                    WS_INVALID_SUBSCRIPTION_TYPE, // response message
                    StatusCode::REQUEST_TIMEOUT, // status code
                    None::<Cookie<'_>>, // cookie
                }
            };

            /* iterating through the msg streams as they're coming to the stream channel */
            while let Some(message) = subscribed_msgs.next().await{
                match message{
                    Ok(message) => {

                        match message{
                            RespValue::SimpleString(message) => {
                                
                                info!("--- sending subscribed revealed roles message to role notif server actor: [{}]", notif_room.clone());
                                let sent_notif = ws_role_notif_actor_address
                                    .send(SendNotif{
                                        event_room: notif_room.clone(), 
                                        notif: message, 
                                        subscribed_at: chrono::Local::now().timestamp_nanos() as u64
                                    }).await;
    
                            }
                            _ => { /* not interested to other variants :) */
                                
                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    WS_INVALID_SUBSCRIPTION_TYPE, // response message
                                    StatusCode::NOT_ACCEPTABLE, // status code
                                    None::<Cookie<'_>>, // cookie
                                }
    
                            }, 
                        }

                    },
                    Err(e) => {

                    }
                }
            }


            /* sending command to redis actor to authorize the this ws client */
            let redis_auth_resp = redis_actor
                .send(Command(resp_array!["AUTH", "geDteDd0Ltg2135FJYQ6rjNYHYkGQa70"])).await;
        
            let Ok(_) = redis_auth_resp else{
                
                let mailbox_err = redis_auth_resp.unwrap_err();
                resp!{
                    &[u8], // the data type
                    &[], // response data
                    &mailbox_err.to_string(), // response message
                    StatusCode::NOT_ACCEPTABLE, // status code
                    None::<Cookie<'_>>, // cookie
                }
            };

            /* 

                since tokio::spawn accepts a closure which captures the env vars into its scope 
                hence we must clone those vars that we need them into the tokio::spawn closure
                in order not to lose their ownership in later scopes,

                async and concurrent push notif handler using:
                
                    tokio::spawn
                    tokio::sync::mpsc
                    redis subscription
                    actix actor

                we can spawn a task inside tokio spawn like subscribing to redis 
                once we subscribed then we can send it to an mpsc jobq channel 
                and in another tokio spawn we can listen to the incoming data
                from the sender of the channel and do whatever we want with that 
                like sending the received data to the actor

                let notif_room_cloned = notif_room.clone();
                let redis_actor_cloned = redis_actor.clone();
                tokio::spawn(async move{
                    
                    /* 
                        sending subscribe message to redis actor to run the subscription interval for the passed 
                        in notif room, by sending this message the actor will run an interval which sends a new 
                        NotifySessionsWithRedisSubscription every 5 seconds to send the result payload of the
                        subscription process to the role notif server actor and from there to all sessions inside
                        the related event room, so with this pattern we only have one subscriber which is the redis
                        actor itself subscribing constantly to the passed in event room.
                    */
                       
                    let redis_subscribe_result = redis_actor_cloned
                        .send(Subscribe{notif_room: notif_room_cloned.clone()})
                        .await;
    
                    resp!{
                        &[u8], // the data type
                        &[], // response data
                        WS_SUBSCRIPTION_INTERVAL_ISSUE, // response message
                        StatusCode::REQUEST_TIMEOUT, // status code
                        None::<Cookie<'_>>, // cookie
                    }
                        
                    
                });
                
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