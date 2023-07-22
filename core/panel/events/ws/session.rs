


/*   -----------------------------------------------------------------------------------
    | websocket session actor to receive push notif subscription from redis subscriber 
    | ----------------------------------------------------------------------------------
    |
    |
*/

use crate::constants::{WS_CLIENT_TIMEOUT, SERVER_IO_ERROR_CODE, WS_REDIS_SUBSCIPTION_INTERVAL, STORAGE_IO_ERROR_CODE, WS_SUBSCRIPTION_INTERVAL};
use crate::events::redis::role::PlayerRoleInfo;
use crate::events::ws::notifs::role::{NotifySessionsWithRedisSubscription, NotifySessionWithRedisSubscription};
use crate::{misc::*, constants::WS_HEARTBEAT_INTERVAL};
use crate::*;
use actix::dev::channel;
use actix::prelude::*;
use diesel::sql_types::ops::Add;
use redis_async::resp::FromResp;
use super::notifs::{
    role::{
        Message as RoleMessage, 
        RoleNotifServer, Disconnect as RoleNotifServerDisconnectMessage,
        Connect as RoleNotifServerConnectMessage, JoinForPushNotif as RoleNotifServerJoinMessage
    }
};




/// redis subscription
#[derive(Message)]
#[rtype(result = "()")]
pub struct NotifySession{
    pub notif_room: String,
    pub payload: String,
    pub peer_name: String,
}



/* a session or peer data, RoleNotifServer actor contains all session instances from the following struct */
pub(crate) struct WsNotifSession{
    pub id: usize, // unique session id
    pub hb: Instant, // client must send ping at least once per 10 seconds (CLIENT_TIMEOUT), otherwise we drop connection.
    pub notif_room: &'static str, // user has joined in to this room 
    pub peer_name: Option<String>, // user mongodb id
    pub ws_role_notif_actor_address: Addr<RoleNotifServer>, // the role notif actor server address,
    pub redis_async_pubsubconn: Arc<PubsubConnection>,
    pub is_subscription_interval_started: bool
}


impl WsNotifSession{

    pub async fn role_subscription(notif_room: &'static str, session_id: usize,
        peer_name: String, redis_async_pubsubconn: Arc<PubsubConnection>,
        ws_role_notif_actor_address: Addr<RoleNotifServer>){

        /* cloning vars that are going to be captured by tokio::spawn(async move{}) cause we need their owned types */
        let cloned_notif_room = notif_room.clone();
        let redis_async_pubsubconn = redis_async_pubsubconn.clone();
        let ws_role_notif_actor_address = ws_role_notif_actor_address.clone();
        let peer_name = peer_name.clone();

        tokio::spawn(async move{

            info!("💡 --- peer [{}] is subscribing to event room: [{}]", peer_name, notif_room);

            /* 🚨 !!! 
                we must receive asyncly from the redis subscription streaming 
                channel otherwise actor gets halted in here since using sync 
                redis and actor redis cause the current thread gets halted
                because they're all blocking operations.
            !!! 🚨 */
            let get_stream_messages = redis_async_pubsubconn
                .subscribe(&cloned_notif_room)
                .await;
            
            let Ok(mut get_stream_messages) = get_stream_messages else{

                use error::{ErrorKind, StorageError::RedisAsync, PanelError};
                let e = get_stream_messages.unwrap_err();
                let error_content = e.to_string().as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(RedisAsync(e)));
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                return ();

            };
        
            /* iterating through the msg streams as they're coming to the stream channel while are not None */
            while let Some(message) = get_stream_messages.next().await{ 

                info!("💡 --- received revealed roles notif from admin");
                
                let resp_val = message.unwrap();
                let stringified_player_roles = String::from_resp(resp_val).unwrap();
                let decoded_player_roles = serde_json::from_str::<Vec<PlayerRoleInfo>>(&stringified_player_roles).unwrap();

                /* sending the received roles to each session separately as a notification */
                for player_info in decoded_player_roles{
                    /* making sure that we're sending the role of this peer to the current session */
                    if player_info._id.to_string() == peer_name{
                        ws_role_notif_actor_address
                            .send(NotifySessionWithRedisSubscription{
                                notif_room: cloned_notif_room.to_string(), /* the event object id  */
                                role_name: player_info.role_name.unwrap(),
                                session_id,
                            }).await;
                    }
                }

            }

        });

    }

    /* client heartbeat */
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>){ /* ctx also contains the instance of the WsNotifSession struct */
        /* 
        
            actor is the WsNotifSession which can be accessible inside the closure 
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
                actor.ws_role_notif_actor_address.do_send(RoleNotifServerDisconnectMessage{id: actor.id, event_name: actor.notif_room.to_owned()}); /* sending disconnect message to the RoleNotifServer actor with the passed in session id and the event name room */
                ctx.stop(); /* stop the ws service */

                return;
            }
                        
        });
        ctx.pong(b""); /* sending empty bytes back to the peer */
    }

}


/* since this is an actor it can communicates with other ws actor as well, by sending pre defined messages to them */
impl Actor for WsNotifSession{

    /* 
        this must be ws::WebsocketContext<WsNotifSession> since 
        ws method accepts an actor with ws::WebsocketContext object 
    */
    type Context = ws::WebsocketContext<WsNotifSession>; /* creating a context object of ws::WebsocketContext struct from the WsNotifSession actor */

    /* once the session actor is started we'll do the following logics */
    fn started(&mut self, ctx: &mut Self::Context){ /* ctx is a mutable reference to the Self::Context */

        /* check the heartbeat of the this session */
        self.hb(ctx);

        let session_actor_address = ctx.address();
        let event_name_room = self.notif_room;
        let peer_name = self.peer_name.as_ref().unwrap();

        /* tell the RoleNotifServer actor asyncly that this session wants to connect to you */
        self.ws_role_notif_actor_address
            .send(RoleNotifServerConnectMessage{addr: session_actor_address.recipient(), event_name: &event_name_room, peer_name: peer_name.clone()}) 
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

    }


    /* the session is about to be stopped */
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        
        /* 
            sending disconnect message to the RoleNotifServer actor with the passed in 
            session id and the event name room, once an actor is stopped its state will
            be cleaned thus we basically we must don't have access to its internal states
            like the actor fields.
        */
        self.ws_role_notif_actor_address.do_send(RoleNotifServerDisconnectMessage{id: self.id, event_name: self.notif_room.to_owned()}); 
        Running::Stop /* return the Stop variant */

    }


}

/* handle messages from RoleNotifServer, we simply send it to peer websocket */
impl Handler<RoleMessage> for WsNotifSession {
   
    type Result = ();

    fn handle(&mut self, msg: RoleMessage, ctx: &mut Self::Context){
        ctx.text(msg.0);
    }
}

impl Handler<NotifySession> for WsNotifSession{

    type Result = ();

    fn handle(&mut self, msg: NotifySession, ctx: &mut Self::Context) -> Self::Result{
        
        info!("💡 --- sending revealed roles notif to peer [{}] in room: [{}]", msg.peer_name, msg.notif_room);
        ctx.text(msg.payload);
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
                 
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();

                let error_instance = PanelError::new(*SERVER_IO_ERROR_CODE, error_content, ErrorKind::Server(Ws(e)));
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
                self.hb = Instant::now(); /* updating the last heartbeat */
            },
            ws::Message::Text(text) => {

                let m = text.trim();
                if m.starts_with("/"){
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0]{

                        /* join the event notif room to subscribe to redis topics */
                        "/join" => {

                            self.ws_role_notif_actor_address.do_send(RoleNotifServerJoinMessage{id: self.id, event_name: self.notif_room});
                            let joined_msg = format!("ready to receive push notif subscriptions constantly from admin in event room [{}]", self.notif_room);
                            ctx.text(joined_msg);

                            /* 
                                if the interval is not already started we'll start it and set the flag to true 
                                otherwise we won't do this on second /join command, which prevents from adding 
                                more interval to the actor state.
                            */
                            if !self.is_subscription_interval_started{
                                
                                info!("💡 --- starting role subscription interval in the background for peer [{}] in room: [{}]", self.peer_name.as_ref().unwrap(), self.notif_room.clone());
                                
                                /* 
                                    start subscription interval for this joined session, since ctx is not Send 
                                    we couldn't put the interval part inside the tokio::spawn()
                                */
                                ctx.run_interval(WS_SUBSCRIPTION_INTERVAL, |actor, ctx|{
                                    
                                    actor.is_subscription_interval_started = true;
                                    
                                    let notif_room = actor.notif_room;
                                    let redis_async_pubsubconn = actor.redis_async_pubsubconn.clone();
                                    let ws_role_notif_actor_address = actor.ws_role_notif_actor_address.clone();
                                    let peer_name = actor.peer_name.clone();
                                    let session_id = actor.id;
                                    
                                    tokio::spawn(async move{
                                        /* starting subscription loop in the background asyncly */
                                        WsNotifSession::role_subscription(
                                            notif_room, 
                                            session_id,
                                            peer_name.unwrap(), 
                                            redis_async_pubsubconn, 
                                            ws_role_notif_actor_address
                                        ).await;
                                    });
                                
                                });
                            }

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