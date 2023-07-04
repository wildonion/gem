




use crate::*;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::events::ws::notifs::role::RoleNotifServer;
use crate::events::ws::session::WsNotifSession;
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
    storage: web::Data<Option<Arc<Storage>>>, // db shared state data
    ws_role_notif_server: web::Data<RoleNotifServer>,
) -> Result<HttpResponse, actix_web::Error> {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            let user_id = route_paths.0.to_owned();
            let notif_room = route_paths.1.to_owned();
            let ws_role_notif_actor_address = ws_role_notif_server.get_ref().clone().start();


            let resp = if notif_room.starts_with("reveal-role-"){
                /* starting reveal role ws connection for the passed in peer and the notif room */
                ws::start(
                    WsNotifSession{
                        id: 0,
                        hb: Instant::now(),
                        peer_name: Some(user_id),
                        subscribed_at: 0,
                        notif_room,
                        app_storage: storage.clone(),
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