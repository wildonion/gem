

/*
    > ----------------------------------------------------
    |             CHATROOM LAUNCHPAD WS APIS
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
 
use crate::events::{
    subscribers::notifs::mmr::{MmrNotifServer, UpdateNotifRoom as MmrUpdateNotifRoom},
    subscribers::notifs::ecq::{EcqNotifServer, UpdateNotifRoom as EcqUpdateNotifRoom},
    subscribers::notifs::role::{RoleNotifServer, UpdateNotifRoom as RoleUpdateNotifRoom},
    subscribers::session::WsNotifSession,
};
use actix::prelude::*;




/*          ------------------------- README -------------------------

                          WebSocket Chatroom Launchpad
    
    an specific user can join an specific chat room to start chat inside an specific 
    chatroom launchpad, here is the example connect address and make sure that client 
    is passing the panel JWT to the header request like `Bearer JWT`:

    Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJfaWQiOjMsInVzZXJfcm9sZSI6IlVzZXIiLCJ0b2tlbl90aW1lIjoxNzAwNDczOTMzNTQ3MjIxMDAwLCJleHAiOjE3MDMwNjU5MzMsImlhdCI6MTcwMDQ3MzkzM30.T1_JWQVLqj_jEC6LxCBF3KpXcWpzcVJxvYxqVT8wDSdOsrcekACo55z9yFhcmxyBN0sEtFaBrGCdKYtASQzFzw
    
    local API:
        ws://localhost:7442/subscribe/chatroomlp/{chatroomlp_id}/{user_screen_cid}/{tx_signature}/{hash_data}
    
    production APIs:
        `wss://notif.panel.conse.app/subscribe/chatroomlp/1/03fe4d2c2eb9ab44971e01d9cd928b4707a9d014381d75ec19f946b78a28164cc6/8ef4637573c6ef6170c817ad22fc4e45de4eae1b86fbe26f19986d49e9c4e24a3fe7d5f6fef58b2ae6a160ca058c41c401401ecc509f8afffe30035e0ad7451f1c/b051b639719983d5062cb8bdb5f57afffb4a634c8c8a6b9e957f583ee1087ea1`

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
                    let is_request_verified = kyced::verify_request(
                        _id, 
                        &user_cid, 
                        &tx_signature, 
                        &hash_data, 
                        None, /* no need to charge the user for this call */
                        connection
                    ).await;

                    let Ok(user) = is_request_verified else{
                        let error_resp = is_request_verified.unwrap_err();
                        return error_resp; /* terminate the caller with an actix http response object */
                    };

                    
                    // TODO - 
                    // users_clps schema
                    // chatroomlp validation find with id
                    // check that the user is already registered and paid for this chatroom
                    // check that there is a room with the passed in id to the api
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
                            peer_name: walletreq::evm::get_keccak256_from(user_cid.clone())
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
                        starting an actor based ws connection for the passed in peer 
                        and the notif room, by doing this we're handling every incoming
                        session connection asyncly and concurrently inside actor threadpool 
                        with this pattern different parts of the app can communicate with
                        WsLaunchpadSession structure asyncly and concurrently also the server
                        actor contains all of the session actors' addresses.
                    */
                    let resp = ws::start(
                        WsLaunchpadSession{
                            id: walletreq::evm::get_keccak256_from(user_cid.clone()),
                            hb: Instant::now(),
                            peer_name: Some(walletreq::evm::get_keccak256_from(user_cid)),
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
    pub use super::chatroomlp;
}