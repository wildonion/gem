


use crate::constants::{WS_CLIENT_TIMEOUT, SERVER_IO_ERROR_CODE, WS_REDIS_SUBSCIPTION_INTERVAL};
use crate::{misc::*, constants::WS_HEARTBEAT_INTERVAL};
use crate::*;
use actix::prelude::*;
use super::notifs::{
    role::{
        Message as RoleMessage, 
        RoleNotifServer, Disconnect as RoleNotifServerDisconnectMessage,
        Connect as RoleNotifServerConnectMessage, Join as RoleNotifServerJoinMessage
    }
};



/* a session or peer data, RoleNotifServer actor contains all session instances from the following struct */
#[derive(Clone)]
pub(crate) struct WsNotifSession{
    pub id: usize, // unique session id
    pub hb: Instant, // client must send ping at least once per 10 seconds (CLIENT_TIMEOUT), otherwise we drop connection.
    pub notif_room: String, // user has joined in to this room 
    pub subscribed_at: i64, // time of redis subscription 
    pub peer_name: Option<String>, // user mongodb id
    pub app_storage: Option<Arc<Storage>>,
    pub ws_role_notif_actor_address: Addr<RoleNotifServer> // the role notif actor server address
}


impl WsNotifSession{

    /* client heartbeat */
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>){ /* ctx also contains the instance of the WsNotifSession struct */
        /* 
            actor is the WsNotifSession which can be accessible inside the closure 
            also we're checking every 5 seconds that if the last hearbeat of the client
            was greater than WS_CLIENT_TIMEOUT seconds then we simply send disconnect 
            message to all session in all rooms
        */
        ctx.run_interval(WS_HEARTBEAT_INTERVAL, |actor, ctx|{
            if Instant::now().duration_since(actor.hb) > WS_CLIENT_TIMEOUT{
                
                info!("websocket client heartbeat failed, disconnecting!");
                actor.ws_role_notif_actor_address.do_send(RoleNotifServerDisconnectMessage{id: actor.id, event_name: actor.notif_room.to_owned()}); /* sending disconnect message to the RoleNotifServer actor with the passed in session id and the event name room */
                ctx.stop(); /* stop the ws service */

                return;
            }
        });
        ctx.pong(b""); /* sending empty bytes back to the peer */
    }

    async fn subscribe(&self, ctx: &mut ws::WebsocketContext<Self>){

        /* running the subscription task in the background every 5 seconds */
        ctx.run_interval(WS_REDIS_SUBSCIPTION_INTERVAL, |actor, ctx|{

            let redis_client = actor.app_storage.as_ref().clone().unwrap().get_redis_sync().unwrap();
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

            /* subscribing to redis topic */
            let mut pubsub = conn.as_pubsub();
            pubsub.subscribe(actor.notif_room.to_owned()).unwrap();

            let msg = pubsub.get_message().unwrap();
            let payload: String = msg.get_payload().unwrap();

            actor.subscribed_at = chrono::Local::now().timestamp_nanos(); /* actor is a mutable reference to the WsNotifSession actor */

            /* sending reveal role topic data */
            ctx.text(payload);


        });

        ctx.pong(b""); /* sending empty bytes back to the peer, in handling Pong message we'll generate a new heartbeat */

    }
}


/* since this is an actor it can communicates with other ws actor as well, by sending pre defined messages to them */
impl Actor for WsNotifSession{

    type Context = ws::WebsocketContext<WsNotifSession>; /* creating a context object from the WsNotifSession actor */

    /* once the session actor is started we'll do the following logics */
    fn started(&mut self, ctx: &mut Self::Context){ /* ctx is a mutable reference to the Self::Context */
        
        /* check the heartbeat of the this session */
        self.hb(ctx); 

        let session_actor_address = ctx.address();
        let event_name_room = self.notif_room.to_owned();
        let peer_name = self.peer_name.as_ref().unwrap();

        /* tell the RoleNotifServer actor asyncly that this session wants to connect to you */
        self.ws_role_notif_actor_address
            .send(RoleNotifServerConnectMessage{addr: session_actor_address.recipient(), event_name: event_name_room.to_owned(), peer_name: peer_name.clone()}) 
            .into_actor(self) /* convert the future object of send() method into an actor future */
            .then(|res, actor, ctx|{
                /* 
                    RoleNotifServerConnectMessage message handler will return 
                    the unique session id of the added session actor into the room 
                */
                match res{ 
                    /* 
                        update the session id of this session actor with the returned 
                        id of the RoleNotifServerConnectMessage message handler 
                    */
                    Ok(res) => actor.id = res, 
                    _ => ctx.stop(),
                }
                fut::ready(()) /* custom future and stream implementation in Actix */
            })
            .wait(ctx);

        /* subscribe to the redis topic for this notif room */
        let fut = Box::pin(async{
            self.subscribe(ctx).await;
        });

        let actor_future = fut.into_actor(self);
        ctx.wait(actor_future); /* calling wait on a mutable pointer of ctx to await on the actor future to be solved later */

    }


    /* the session is about to be stopped */
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.ws_role_notif_actor_address.do_send(RoleNotifServerDisconnectMessage{id: self.id, event_name: self.notif_room.to_owned()}); /* sending disconnect message to the RoleNotifServer actor with the passed in session id and the event name room */
    
        Running::Stop /* return the Stop variant */

    }


}

/* handle messages from RoleNotifServer, we simply send it to peer websocket */
impl Handler<RoleMessage> for WsNotifSession {
   
    type Result = ();

    fn handle(&mut self, msg: RoleMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/* event listener or streamer to receive ws message */
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsNotifSession{

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

        let msg = match msg{
            Ok(msg) => msg,
            Err(e) => {

                /* custom error handler */
                use error::{ErrorKind, ServerError::{ActixWeb, Ws}, PanelError};
                let msg_content = [0u8; 32];
                let error_content = &e.to_string();
                msg_content.to_vec().extend_from_slice(error_content.as_bytes());

                let error_instance = PanelError::new(*SERVER_IO_ERROR_CODE, msg_content, ErrorKind::Server(Ws(e)));
                let error_buffer = error_instance.write_sync(); /* write to file also returns the full filled buffer */

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
                self.hb = Instant::now(); /* updating the last heartbeat */
            },
            ws::Message::Text(text) => {

                let m = text.trim();
                if m.starts_with("/"){
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0]{

                        /* join the event notif room to subscribe to redis topics */
                        "/join" => {

                            let event_room_name = self.notif_room.to_owned();
                            self.ws_role_notif_actor_address.do_send(RoleNotifServerJoinMessage{id: self.id, event_name: event_room_name.clone()});
                            let joined_msg = format!("joined event room: [{}] to receive push notif subscriptions", event_room_name);
                            ctx.text(joined_msg);

                        },
                        _ => ctx.text(format!("unknown command")),
                    }
                } else{

                    ctx.text(format!("can't send none slash command!"));
                }
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