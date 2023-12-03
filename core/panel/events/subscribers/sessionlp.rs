


/*  > ------------------------------------------------
    |  websocket session actor to chat for launchpad 
    | ------------------------------------------------
    | contains: message structures and their handlers + WS realtime stream message handler
    |
    | with actors we can communicate between different parts of the app by sending async 
    | messages to other actors through jobq channels, they also must have a handler for each 
    | type of incoming messages.
    |
*/

use crate::constants::{WS_CLIENT_TIMEOUT, SERVER_IO_ERROR_CODE, STORAGE_IO_ERROR_CODE, WS_SUBSCRIPTION_INTERVAL};
use crate::models::users::{User, UserWalletInfoResponse};
use crate::models::chatdb::UserChat;
use crate::{misc::*, constants::WS_HEARTBEAT_INTERVAL};
use crate::*;
use s3req::Storage;
use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web::dev::Payload;
use crate::events::subscribers::chatroomlp::{
    
    ChatRoomLaunchpadServer, Disconnect as ChatRoomLaunchpadServerDisconnectMessage,
    Connect as ChatRoomLaunchpadServerConnectMessage, NotifySessionsWithNewMessage,
    Join as ChatRoomLaunchpadServerJoinMessage, Message as WsMessage
    
};



#[derive(Clone)]
pub(crate) struct WsLaunchpadSession{
    pub id: String, // unique session id or screen_cid
    pub hb: Instant, // client must send ping at least once per 10 seconds (CLIENT_TIMEOUT), otherwise we drop connection.
    pub chat_room: &'static str, // user has joined in to this room 
    pub r1pubkey: String,
    pub r1signature: String,
    pub peer_name: Option<String>, // user mongodb id
    pub ws_chatroomlp_actor_address: Addr<ChatRoomLaunchpadServer>, // the mmr notif actor server address,
    pub app_storage: Option<Arc<Storage>>,
}


impl WsLaunchpadSession{

    /* client heartbeat */
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>){ /* ctx also contains the instance of the WsLaunchpadSession struct */
        /* 
        
            actor is the WsLaunchpadSession which can be accessible inside the closure 
            also we're checking every 5 seconds that if the last hearbeat of the client
            was greater than WS_CLIENT_TIMEOUT seconds then we simply send disconnect 
            message to all session in all rooms

        
            since the second param of the run_interval() method is a closure which 
            captures the env vars into its scope thus the closure params must return
            the self or the actor instance and the ctx types to use them inside the 
            closure scope. 
        
        */
        ctx.run_interval(WS_HEARTBEAT_INTERVAL, |actor, ctx|{
                        
            if Instant::now().duration_since(actor.hb) > WS_CLIENT_TIMEOUT{
                
                error!("🚨 --- websocket client heartbeat failed, disconnecting!");
                actor.ws_chatroomlp_actor_address.do_send(ChatRoomLaunchpadServerDisconnectMessage{id: actor.id.clone(), chatroom_name: actor.chat_room.to_owned()}); /* sending disconnect message to the ChatRoomLaunchpadServer actor with the passed in session id and the event name room */
                ctx.stop(); /* stop the ws service */

                return;
            }
                        
        });
        
        ctx.pong(b""); /* sending empty bytes back to the peer */

    }

}


/* since this is an actor it can communicates with other ws actor as well, by sending pre defined messages to them */
impl Actor for WsLaunchpadSession{

    /* 
        this must be ws::WebsocketContext<WsLaunchpadSession> since 
        ws method accepts an actor with ws::WebsocketContext object 
    */
    type Context = ws::WebsocketContext<WsLaunchpadSession>; /* creating a context object of ws::WebsocketContext struct from the WsLaunchpadSession actor */

    /* once the session actor is started we'll do the following logics */
    fn started(&mut self, ctx: &mut Self::Context){ /* ctx is a mutable reference to the Self::Context */

        /* check the heartbeat of the this session */
        self.hb(ctx);

        let session_actor_address = ctx.address();
        let chatroom_name_room = self.chat_room;
        let peer_name = self.peer_name.as_ref().unwrap();

        /* 
            tell the ChatRoomLaunchpadServer actor asyncly that this session wants to 
            connect to you and assign a unique id to it
        */
        self.ws_chatroomlp_actor_address
            .send(ChatRoomLaunchpadServerConnectMessage{addr: session_actor_address.recipient(), chatroom_name: &chatroom_name_room, peer_name: peer_name.clone()}) 
            .into_actor(self) /* convert the future object of send() method into an actor future */
            .then(|res, actor, ctx|{
                /* 
                    ChatRoomLaunchpadServerConnectMessage message handler will return 
                    the unique session id of the added session actor into the room 
                */
                match res{ 
                    /* 
                        update the session id of this session actor with the returned 
                        id of the ChatRoomLaunchpadServerConnectMessage message handler 
                    */
                    Ok(res) => actor.id = res, 
                    _ => ctx.stop(),
                }
                fut::ready(()) /* custom future and stream implementation in Actix */
            })
            .wait(ctx);


        // ---------- PUBLISHING ChatRoomLaunchpadServerJoinMessage topic
        // ----------------------------------------------------------------------
        /* 
            publish ChatRoomLaunchpadServerJoinMessage message asyncly, so later on server actor can subscribe to 
            once the server actor gets subscribed to this clients can see:
            user with id: [0] connected to chatroom: [1]
            user with id: [0] disconnected from the chatroom: [1]

            we can also have the following code instead of publishing:
            self.ws_chatroomlp_actor_address.do_send(
                ChatRoomLaunchpadServerJoinMessage{ 
                    id: self.id.clone(), 
                    chatroom_name: self.chat_room 
                }
            );
        */
        self.issue_system_async(ChatRoomLaunchpadServerJoinMessage{ id: self.id.clone(), chatroom_name: self.chat_room });
        let joined_msg = format!("sessionlp::user with id [{}] joined in chatroom launchpad [{}]", self.peer_name.clone().unwrap(), self.chat_room);
        ctx.text(joined_msg);
        // ----------------------------------------------------------------------

    }


    /* the session is about to be stopped */
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        
        /* 
            sending disconnect message to the ChatRoomLaunchpadServer actor with the passed in 
            session id and the event name room, once an actor is stopped its state will
            be cleaned thus we basically we must don't have access to its internal states
            like the actor fields.
        */
        self.ws_chatroomlp_actor_address.do_send(ChatRoomLaunchpadServerDisconnectMessage{id: self.id.clone(), chatroom_name: self.chat_room.to_owned()}); 
        Running::Stop /* return the Stop variant */

    }


}

/* 
    a message handler to send Message type strings to a session 
    we'll send message in ChatRoomLaunchpadServer::send_message()
    method to all sessions with this handler thus each session 
    must implement a handler for Message struct
*/
impl Handler<WsMessage> for WsLaunchpadSession{

    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context){
        ctx.text(msg.0);
    }
}

/* stream, listener or event handler to handle the incoming websocket byte packets in realtime */
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsLaunchpadSession{

     /* 
        the handler method to handle the incoming websocket messages by decoding them,
        when stream resolves its next item, handle() is called with that item, Self::Context
        is of type WebsocketContext<WsLaunchpadSession>
    */
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

        let msg = match msg{
            Ok(msg) => msg,
            Err(e) => {

                /* custom error handler */
                use error::{ErrorKind, ServerError::{ActixWeb, Ws}, PanelError};
                 
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();

                let error_instance = PanelError::new(*SERVER_IO_ERROR_CODE, error_content, ErrorKind::Server(Ws(e)), "WsLaunchpadSession::StreamHandler::handle");
                let error_buffer = error_instance.write_sync(); /* write to file also returns the full filled buffer from the error  */

                ctx.stop();
                return;
            }
        };

        match msg{
            ws::Message::Ping(msg) => {
                self.hb = Instant::now(); /* updating the last heartbeat */
                ctx.pong(&msg);
            },
            ws::Message::Pong(_) => {
                /* once we received the pong message we'll update the last heartbeat */
                self.hb = Instant::now(); /* updating the last heartbeat */
            },
            ws::Message::Text(text) => {

                /* ---- decrypting the message ---- */
                let mut r1_wallet = walletreq::secp256r1::generate_new_wallet();
                let get_decrypted_new_message = r1_wallet.self_verify_secp256r1_signature(&self.r1signature, &self.r1pubkey);
                if get_decrypted_new_message.is_err(){
                    let error = get_decrypted_new_message.unwrap_err();
                    return ctx.text(error.to_string());
                }

                let message_vec = get_decrypted_new_message.unwrap();
                // converting vector decoded message into static slice so it can live long enough for the entire app
                let message_vec_slice = misc::vector_to_static_slice(message_vec); 
                let new_message = std::str::from_utf8(message_vec_slice).unwrap();

                /* handling slash commands */
                let m = text.trim();
                if m.starts_with("/"){
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0]{

                        /* --------------------------------- */
                        /*          GET ONLINE USERS         */
                        /* --------------------------------- */
                        "/info" => {

                            /* get all room from redis storage */
                            let storage = self.app_storage.as_ref().to_owned(); /* as_ref() returns shared reference */
                            let mut redis_client = storage.as_ref().clone().unwrap().get_redis_sync().unwrap().to_owned();
                            let pg_pool = storage.clone().unwrap().as_ref().get_pgdb_sync().unwrap();
                            let connection = &mut pg_pool.get().unwrap();
                            let redis_result_rooms_string: String = redis_client.get("chatroomlp_server_actor_rooms").unwrap();
                            
                            /* 
                                structure of all rooms is like a mapping between the room name and its peer ids: 
                                HashMap<String, HashSet<String>>
                            */
                            let rooms_in_redis = serde_json::from_str::<HashMap<String, HashSet<String>>>(redis_result_rooms_string.as_str()).unwrap();
                            let users_in_this_event = rooms_in_redis.get(self.chat_room).unwrap();
                            
                            let users = users_in_this_event
                                .into_iter()
                                .map(|u|{
                                    let user_info = User::find_by_screen_cid_none_async(u, connection).unwrap();
                                    UserWalletInfoResponse{
                                        username: user_info.username,
                                        avatar: user_info.avatar,
                                        mail: user_info.mail,
                                        screen_cid: user_info.screen_cid,
                                        stars: user_info.stars,
                                        created_at: user_info.created_at.to_string(),
                                        bio: user_info.bio,
                                        banner: user_info.banner,
                                    }

                                })
                                .collect::<Vec<UserWalletInfoResponse>>();

                            let json_stringified_users = serde_json::to_string_pretty(&users).unwrap();

                            /* 
                                sending to this peer, also note that the param must implements
                                ByteString trait thus the param must be in form of byte string
                                text() method takes a mutable pointer to the self or the instance
                                of WebsocketContext<WsLaunchpadSession>
                            */
                            ctx.text(format!("online events: {}", rooms_in_redis.len()));
                            ctx.text(format!("online users in this event: {}", json_stringified_users));

                        },
                        _ => ctx.text(format!("unknown command")),
                    }
                } 

                let im_this_actor: Self = self.clone();
                let server_actor = self.ws_chatroomlp_actor_address.clone();
                let chatroom_name = self.chat_room.to_string().clone();
                let session_id = self.id.clone();

                /* sending the message asyncly to all session in that room in a separate thread */
                tokio::spawn(async move{
                    
                    let notify_msg = NotifySessionsWithNewMessage{
                        chat_room: chatroom_name,
                        session_id: session_id,
                        new_message: new_message.to_string(),
                    };

                    /* 
                        sending NotifySessionsWithNewMessage to server actor asyncly, then
                        server actor will notify all the sessions in this room with new messagex
                    */
                    server_actor
                        .send(
                            notify_msg.clone()
                        ).await.unwrap();
                    
                    // ---------- PUBLISHING NotifySessionsWithNewMessage topic
                    // ----------------------------------------------------------------------
                    /*  
                        instead of sending different message to all server actors separately we can publish and 
                        issuing that message asyncly so those server actors that are interested to that message
                        can subscribe to that so in here we're notifying server about a new message by publishing 
                        NotifySessionsWithNewMessage message asyncly, so later on server actor can subscribe to, 
                        subscribing to NotifySessionsWithNewMessage message causes client to see the incoming 
                        message in the room twice because we're notifying the server actor two times with new 
                        message, first one is above and the other one is in ChatRoomLaunchpadServer::started() 
                        method.
                    */
                    // this_actor
                    //     .issue_system_async(notify_msg);
                    // ----------------------------------------------------------------------
                    
                });

                /* --------- storing texts in db in a separate thread --------- */
                let app_storage = self.app_storage.clone();
                let chatroom_name = self.chat_room.to_string().clone();
                let session_id = self.id.clone();

                tokio::spawn(async move{

                    /*  >------------------------------------------------------------------------------
                        unwrapping process of app_storage and creating pg connection must be done inside 
                        the tokio::spawn since we can't move pg pool into this scope because self will be 
                        moved too which will be dropped once the ws::Message::Text{} arm gets executed 
                        which won't last across .awaits inside the tokio::spawn sicne tokio::spawn is another 
                        and separate thread also note that when we're solving a future inside tokio:::spawn 
                        or another thread all the types used inside the future scopes must be Send so we 
                        can share them safely also they must have valid lifetime across .await or before 
                        and after the future since futures will be pinned into the ram until they get solved 
                        and executed and rust however drop the lifetime of the type once it goes 
                        out of the scope.
                    */
                    let pool_conn = app_storage.unwrap();
                    let connection = &mut pool_conn.get_pgdb_sync().unwrap().get().unwrap();

                    /* 
                        all the types passed in to the UserChat::store must be Send 
                        in this tokio::spawn green threadpool so tokio executor can
                        share them safely between threads
                    */
                    if let Err(resp) = UserChat::store(
                        chatroom_name.parse::<i32>().unwrap(),
                        &session_id,
                        new_message,
                        connection
                    ).await{

                        error!("WsLaunchpadSession::StreamHandler::handle::UserChat::store error: {:?}", resp);
                    }

                });

            },
            ws::Message::Binary(_) => info!("unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }

    }

}