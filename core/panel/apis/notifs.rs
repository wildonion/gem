



/*
    > ----------------------------------------------------
    |         NOTIF AND CHAT SUBSCRIPTIONS WS APIS
    | ----------------------------------------------------
    |
    |

*/

use crate::*;
use crate::events::subscribers::chatroomlp::ChatRoomLaunchpadServer;
use crate::events::subscribers::chatroomlp::UpdateChatRoom;
use crate::events::subscribers::sessionlp::WsLaunchpadSession;
use crate::models::users::User;
use crate::models::users::UserChatRoomLaunchpadRequest;
use crate::models::users::UserRole;
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use s3::*;
use wallexerr::Wallet;
use crate::events::{
    subscribers::notifs::mmr::{MmrNotifServer, UpdateNotifRoom as MmrUpdateNotifRoom},
    subscribers::notifs::ecq::{EcqNotifServer, UpdateNotifRoom as EcqUpdateNotifRoom},
    subscribers::notifs::role::{RoleNotifServer, UpdateNotifRoom as RoleUpdateNotifRoom},
    subscribers::session::WsNotifSession,
};
use actix::prelude::*;



/*          ------------------------- README -------------------------

                        WebSocket Push Notif Subscription
    
    an specific user can join an specific event room to subscribe to what ws actors 
    will be sent in different parts of the app so this route will be used to receive 
    push notif from admin reveal roles, mmr and ecq engines, here is the example connect 
    address and make sure that client is passing the rendezvous server JWT to the header 
    request like `Bearer JWT`:
    
    local API:
        ws://localhost:7442/subscribe/
    
    production APIs:
        `wss://notif.panel.conse.app/subscribe/64b827fad916781c6d68948a/reveal-role-64b82757d916781c6d689488`
        `wss://notif.panel.conse.app/subscribe/64b827fad916781c6d68948a/mmr-64b82757d916781c6d689488`
        `wss://notif.panel.conse.app/subscribe/64b827fad916781c6d68948a/ecq-64b82757d916781c6d689488`

    NOTE: we just have to make sure that the user is already inside the event 
            and did the reservation process for the event.

    client must be connect to this route then we have a full duplex communication channel 
    also there is a path in this route which is the event room that must be connected to 
    and is the name of the notification room which can be one of the following:
    
    `ecq-{event_id}`,        ----- /join-ecq
    `mmr-{event_id}`,        ----- /join-mmr
    `reveal-role-{event_id}` ----- /join-roles

    users after participating in an event we'll redirect them to the event page 
    after that the client must call this route with the passed in event id like 
    so: `/{user_id}/reveal-role-{notif_room}` in which a new full duplex ws connection 
    will be stablished to this server which will send the subscribed redis topics 
    to the channel that contains event peers.

*/
#[get("/{user_id}/{notif_room}")]
async fn notif_subs(
    req: HttpRequest, 
    stream: web::Payload, 
    route_paths: web::Path<(String, String)>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ws_role_notif_server: web::Data<Addr<RoleNotifServer>>,
    ws_mmr_notif_server: web::Data<Addr<MmrNotifServer>>,
    ws_ecq_notif_server: web::Data<Addr<EcqNotifServer>>,
) -> PanelHttpResponse {


    /*
        this route requires conse rendezvous hyper server JWT since the user that wants to receive push notif
        is the user of rendezvous server thus his JWT is needed to authorize him to be able to use this route, 
        hence the header must contains the Bearer key to extract the JWT from it 
    */
    if let Some(header_value) = req.headers().get("Bearer"){

        let token = format!("Bearer {}", header_value.to_str().unwrap());
        
        /*
            @params: 
                - @token          → JWT

            note that this token must be taken from the conse rendezvous hyper server
        */
        match rendezvous_passport!{ &token }{
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
                        let ws_mmr_notif_actor_address = ws_mmr_notif_server.get_ref().to_owned();
                        let ws_ecq_notif_actor_address = ws_ecq_notif_server.get_ref().to_owned();


                        /* 
                            ------------------------------------------------------------
                             UPDATING NOTIF ROOMS OF ALL ACTORS WITH THE PASSED IN ROOM
                            ------------------------------------------------------------
                            sending the update message to mutate the notif room before starting the session actor
                            also make sure that the RoleNotifServer, MmrNotifServer and EcqNotifServer actors are 
                            already started when we're here otherwise by calling this route every time a new actor 
                            will be started and the previous state will be lost thus rooms won't be stored and the 
                            new one will be replace the old one
                        */
                        let update_role_notif_room_result = ws_role_notif_actor_address
                            .send(RoleUpdateNotifRoom{
                                notif_room: notif_room_str, 
                                peer_name: user_id.clone()
                            })
                            .await;

                        let Ok(_) = update_role_notif_room_result else{
                        
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                WS_UPDATE_NOTIF_ROOM_ISSUE, // response message
                                StatusCode::REQUEST_TIMEOUT, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

                        let update_mmr_notif_room_result = ws_mmr_notif_actor_address
                            .send(MmrUpdateNotifRoom{
                                notif_room: notif_room_str, 
                                peer_name: user_id.clone()
                            })
                            .await;

                        let Ok(_) = update_mmr_notif_room_result else{
                        
                            resp!{
                                &[u8], // the data type
                                &[], // response data
                                WS_UPDATE_NOTIF_ROOM_ISSUE, // response message
                                StatusCode::REQUEST_TIMEOUT, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        };

                        let update_ecq_notif_room_result = ws_ecq_notif_actor_address
                            .send(EcqUpdateNotifRoom{
                                notif_room: notif_room_str, 
                                peer_name: user_id.clone()
                            })
                            .await;

                        let Ok(_) = update_ecq_notif_room_result else{
                        
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
                                /* 
                                    starting an actor based ws connection for the passed in peer 
                                    and the notif room, by doing this we're handling every incoming
                                    session connection asyncly and concurrently inside actor threadpool 
                                    with this pattern different parts of the app can communicate with
                                    WsNotifSession structure asyncly and concurrently also the server
                                    actor contains all of the session actors' addresses.
                                */
                                ws::start(
                                    WsNotifSession{
                                        id: 0,
                                        hb: Instant::now(),
                                        peer_name: Some(user_id),
                                        notif_room: notif_room_str,
                                        ws_role_notif_actor_address,
                                        ws_mmr_notif_actor_address,
                                        ws_ecq_notif_actor_address,
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

/*          ------------------------- README -------------------------

                          WebSocket Chatroom Launchpad
    
    an specific user can join an specific chat room to start chat inside an specific 
    chatroom launchpad, here is the example connect address and make sure that client 
    is passing the panel JWT to the header request like `Bearer JWT`:

    Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJfaWQiOjQsInVzZXJfcm9sZSI6IlVzZXIiLCJ0b2tlbl90aW1lIjoxNzAwMjM4MjUwOTIwNzMzMDAwLCJleHAiOjE3MDI4MzAyNTAsImlhdCI6MTcwMDIzODI1MH0.t5z961iVrMfAVuNdcIZ6LNhIszpcj75YCHv97vdDIOJoYBSC2iZW8TiMxMSJZHqdrOkf7saOrkycSR8eTKITqA
    
    local API:
        ws://localhost:7442/subscribe/chatroomlp/{chatroomlp_id}/{user_screen_cid}/{tx_signature}/{hash_data}
    
    production APIs:
        `ws://localhost:7442/subscribe/chatroomlp/1/035e339b6b7adf2a771a9d3386244526316f9877069755e288ec2d6b1b06fd25ba/035e339b6b7adf2a771a9d3386244526316f9877069755e288ec2d6b1b06fd25ba/035e339b6b7adf2a771a9d3386244526316f9877069755e288ec2d6b1b06fd25ba`

*/
#[get("/chatroomlp/{chatroomlp_id}/{user_cid}/{tx_signature}/{hash_data}")]
#[passport(user)]
async fn chatroomlp(
    req: HttpRequest, 
    stream: web::Payload, 
    clpucid: web::Path<(i32, String, String, String)>,
    payload: Multipart,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ws_chatroomlp_actor_address: web::Data<Addr<ChatRoomLaunchpadServer>>,
) -> PanelHttpResponse {

    
    let arced_payload = std::sync::Arc::new(tokio::sync::Mutex::new(payload));
    let (json_data, files) = misc::extract_multipart(arced_payload).await.unwrap();
    
    let storage = storage.as_ref().to_owned();
    let redis_async_pubsubconn = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();

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

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
            
            let connection = &mut pg_pool.get().unwrap();

            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match User::passport(req.clone(), granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    let clpucid = clpucid.to_owned();
                    let chat_room = clpucid.0;
                    let user_cid = clpucid.1;
                    let tx_signature = clpucid.2;
                    let hash_data = clpucid.3;
                    let chat_room_str = string_to_static_str(format!("{}", chat_room));
                    let ws_chatroomlp_actor_address = ws_chatroomlp_actor_address.get_ref().to_owned();

                    
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*   -=-=-=-=-=- USER MUST BE KYCED -=-=-=-=-=-  */
                    /*   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=  */
                    /*
                        followings are the param 
                        must be passed to do the 
                        kyc process on request data
                        @params:
                            - _id              : user id
                            - from_cid         : user crypto id
                            - tx_signature     : tx signature signed
                            - hash_data        : sha256 hash of data generated in client app
                            - deposited_amount : the amount of token must be deposited for this call
                    */
                    // let is_request_verified = kyced::verify_request(
                    //     _id, 
                    //     &user_cid, 
                    //     &tx_signature, 
                    //     &hash_data, 
                    //     None, /* no need to charge the user for this call */
                    //     connection
                    // ).await;

                    // let Ok(user) = is_request_verified else{
                    //     let error_resp = is_request_verified.unwrap_err();
                    //     return error_resp; /* terminate the caller with an actix http response object */
                    // };


                    
                    // TODO - 
                    // chatroomlp validation find with id
                    // check that the user is already registered for this chatroom
                    // ...



                    /* 
                        ------------------------------------------------------------
                            UPDATING CHAT ROOM ACTOR WITH THE PASSED IN ROOM
                        ------------------------------------------------------------
                        sending the update message to mutate the notif room before starting the session actor
                        also make sure that the ChatRoomLaunchpadServer actor is already started when we're here 
                        otherwise by calling this route every time a new actor will be started and the previous 
                        state will be lost thus rooms won't be stored and the new one will be replace the old one
                    */
                    let update_chat_room_result = ws_chatroomlp_actor_address
                        .send(UpdateChatRoom{
                            chat_room: chat_room_str, 
                            peer_name: Wallet::generate_keccak256_from(user_cid.clone())
                        })
                        .await;

                    let Ok(_) = update_chat_room_result else{
                    
                        resp!{
                            &[u8], // the data type
                            &[], // response data
                            WS_UPDATE_CLP_ROOM_ISSUE, // response message
                            StatusCode::REQUEST_TIMEOUT, // status code
                            None::<Cookie<'_>>, // cookie
                        }
                    };


                    /* 
                        --------------------------------
                        STARTING THE WEBSOCKET SESSION
                        --------------------------------

                    */

                    /* 
                        starting an actor based ws connection for the passed in peer 
                        and the notif room, by doing this we're handling every incoming
                        session connection asyncly and concurrently inside actor threadpool 
                        with this pattern different parts of the app can communicate with
                        WsLaunchpadSession structure asyncly and concurrently also the server
                        actor contains all of the session actors' addresses.
                    */
                    let resp = ws::start(
                        WsLaunchpadSession{
                            id: 0,
                            hb: Instant::now(),
                            peer_name: Some(Wallet::generate_keccak256_from(user_cid)),
                            chat_room: chat_room_str,
                            ws_chatroomlp_actor_address,
                            app_storage: storage.clone(),
                        }, 
                        &req, 
                        stream
                    );

                    /* 
                        sending the ws connection response, we'll have either a successful 
                        full duplex connection or response error 
                    */
                    resp
                        
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
    /* 
        all of the following routes accept a payload streamer
        which will be used to extract the utf8 bytes from the 
        payload asyncly
    */
    pub use super::notif_subs;
    pub use super::chatroomlp;
}