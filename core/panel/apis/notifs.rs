



/*
     ---------------------------------------------
    |         NOTIF SUBSCRIPTIONS WS APIS
    | --------------------------------------------
    |
    |

*/

use crate::*;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::events::{
    subscribers::notifs::role::{RoleNotifServer, UpdateNotifRoom},
    subscribers::session::WsNotifSession,
};
use actix::prelude::*;



/* 

    this route will be used to receive push notif from admin reveal roles, mmr 
    and ecq engines, here is the example connect address:
    
        ws://localhost:7442/subscribe/64b827fad916781c6d68948a/reveal-role-64b82757d916781c6d689488
        ws://localhost:7442/subscribe/64b827fad916781c6d68948a/mmr-64b82757d916781c6d689488
        ws://localhost:7442/subscribe/64b827fad916781c6d68948a/ecq-64b82757d916781c6d689488

    NOTE: we just have to make sure that the user is already inside the event 
          and did the reservation process for the event. 

    client must be connect to this route then we have a full duplex communication channel 
    also there is a path in this route which is the event room that must be connected to 
    and is the name of the notification room which can be one of the following:
    
        `ecq-{event_id}`, 
        `mmr-{event_id}`,
        `reveal-role-{event_id}`

    users after participating in an event we'll redirect them to the event page after that 
    the client must call this route with the passed in event id like so: /{user_id}/reveal-role-{notif_room}
    in which a new full duplex ws connection will be stablished to this server which will send 
    the subscribed redis topics to the channel that contains event peers

*/
#[get("/{user_id}/{notif_room}")]
async fn notif_subs(
    req: HttpRequest, 
    stream: web::Payload, 
    route_paths: web::Path<(String, String)>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ws_role_notif_server: web::Data<Addr<RoleNotifServer>>,
) -> PanelHttpResponse {


    /*
        this route requires conse mafia hyper server JWT since the user that wants to receive push notif
        is the user of mafia server thus its JWT is needed to authorize it to be able to use this route, hence
        the header must contains the Bearer key to extract the JWT from it
    */
    if let Some(header_value) = req.headers().get("Bearer"){

        let token = format!("Bearer {}", header_value.to_str().unwrap());
        
        /*
            @params: 
                - @toke          → JWT

            note that this token must be taken from the conse mafia hyper server
        */
        match mafia_passport!{ &token }{
            true => {
            
                let storage = storage.as_ref().to_owned();
                let redis_async_pubsubconn = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();

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

                        let resp = {
                                if notif_room.clone().starts_with("reveal-role-") || 
                                notif_room.starts_with("ecq-") ||
                                notif_room.starts_with("mmr-") {
                                /* starting ws connection for the passed in peer and the notif room */
                                ws::start(
                                    WsNotifSession{
                                        id: 0,
                                        hb: Instant::now(),
                                        peer_name: Some(user_id),
                                        notif_room: notif_room_str,
                                        ws_role_notif_actor_address,
                                        app_storage: storage.clone(),
                                        is_subscription_interval_started: false
                                    }, 
                                    &req, 
                                    stream
                                )
                            } else{

                                resp!{
                                    &[u8], // the data type
                                    &[], // response data
                                    WS_INVALID_PATH, // response message
                                    StatusCode::NOT_ACCEPTABLE, // status code
                                    None::<Cookie<'_>>, // cookie
                                }

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
    pub use super::notif_subs;
}