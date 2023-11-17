


/*  > ------------------------------------------------
    |  websocket session actor to chat for launchpad 
    | ------------------------------------------------
    | contains: message structures and their handlers + WS realtime stream message handler
    |
    | with actors we can communicate between different parts of the app by sending async 
    | messages to each other through jobq channels, they also must have a handler for each 
    | type of incoming messages like redis streams and pubsub patterns with ws actors and 
    | tokio concepts (jobq channels, spawn, select, time interval) by streaming over io 
    | future object of bytes to register a push notif.
    |
*/

use crate::constants::{WS_CLIENT_TIMEOUT, SERVER_IO_ERROR_CODE, STORAGE_IO_ERROR_CODE, WS_SUBSCRIPTION_INTERVAL};
use crate::{misc::*, s3::*, constants::WS_HEARTBEAT_INTERVAL};
use crate::*;
use actix::prelude::*;
use crate::events::subscribers::chatroomlp::{
    
    ChatRoomLaunchpadServer, Disconnect as ChatRoomLaunchpadServerDisconnectMessage,
    Connect as ChatRoomLaunchpadServerConnectMessage, NotifySessionsWithNewMessage,
    Join as ChatRoomLaunchpadServerJoinMessage, Message as WsMessage
    
};




pub(crate) struct WsLaunchpadSession{
    pub id: usize, // unique session id
    pub hb: Instant, // client must send ping at least once per 10 seconds (CLIENT_TIMEOUT), otherwise we drop connection.
    pub chat_room: &'static str, // user has joined in to this room 
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
                actor.ws_chatroomlp_actor_address.do_send(ChatRoomLaunchpadServerDisconnectMessage{id: actor.id, chatroom_name: actor.chat_room.to_owned()}); /* sending disconnect message to the ChatRoomLaunchpadServer actor with the passed in session id and the event name room */
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

        /* tell the ChatRoomLaunchpadServer actor asyncly that this session wants to connect to you */
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

    }


    /* the session is about to be stopped */
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        
        /* 
            sending disconnect message to the ChatRoomLaunchpadServer actor with the passed in 
            session id and the event name room, once an actor is stopped its state will
            be cleaned thus we basically we must don't have access to its internal states
            like the actor fields.
        */
        self.ws_chatroomlp_actor_address.do_send(ChatRoomLaunchpadServerDisconnectMessage{id: self.id, chatroom_name: self.chat_room.to_owned()}); 
        Running::Stop /* return the Stop variant */

    }


}

/* 
    a message handler to send Message type strings to a session 
    we're sending message in ChatRoomLaunchpadServer::send_message()
    method to all sessions thus each session must have a handler for 
    Message struct
*/
impl Handler<WsMessage> for WsLaunchpadSession{

    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context){
        ctx.text(msg.0);
    }
}

/* stream, listener or event handler to handle the incoming websocket byte packets in realtime */
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsLaunchpadSession{

     /* the handler method to handle the incoming websocket messages by decoding them */
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

                self.ws_chatroomlp_actor_address.do_send(ChatRoomLaunchpadServerJoinMessage{ id: self.id, chatroom_name: self.chat_room });
                let joined_msg = format!("joined in chatroom launchpad [{}]", self.chat_room);
                ctx.text(joined_msg);

                let this_actor = self.ws_chatroomlp_actor_address.clone();
                let chatroom_name = self.chat_room.to_string().clone();
                let session_id = self.id;
                let new_message = text.clone();

                /* sending the message asyncly to all session in that room */
                tokio::spawn(async move{
                    this_actor
                        .send(
                            NotifySessionsWithNewMessage{
                                chat_room: chatroom_name,
                                session_id,
                                new_message: new_message.to_string(),
                            }
                        ).await.unwrap();
                });


                // TODO - store text in db
                // ...

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