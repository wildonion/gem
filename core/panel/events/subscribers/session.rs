


/*   ------------------------------------------------------------------------------------------------------------------
    | websocket session actor to receive push notif subscription from redis subscriber of role, mmr and ecq publishers 
    | -----------------------------------------------------------------------------------------------------------------
    |
    | with actors we can communicate between different parts of the app by sending async 
    | messages to each other through jobq channels, they also must have a handler for each 
    | type of incoming messages like redis streams and pubsub patterns with ws actors and 
    | tokio concepts (jobq channels, spawn, select, time interval) by streaming over io 
    | future object of bytes to register a push notif.
    |
*/

use crate::constants::{WS_CLIENT_TIMEOUT, SERVER_IO_ERROR_CODE, STORAGE_IO_ERROR_CODE, WS_SUBSCRIPTION_INTERVAL};
use crate::events::publishers::role::PlayerRoleInfo;
use crate::events::subscribers::notifs::role::{NotifySessionsWithRedisSubscription, NotifySessionWithRedisSubscription};
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


/* 
    since it's macro it must be complied before any main codes 
    so we can use it on top of the structures, 
*/
#[derive(Message)]
/*
    the following macro will implement the message Handler trait for the 
    NotifySession struct with the result of type () at compile time
*/
#[rtype(result = "()")] 
pub struct NotifySession{
    pub notif_room: String,
    pub payload: String,
    pub peer_name: String,
}



/* a session or peer data, RoleNotifServer actor contains all session instances from the following struct in its rooms */
pub(crate) struct WsNotifSession{
    pub id: usize, // unique session id
    pub hb: Instant, // client must send ping at least once per 10 seconds (CLIENT_TIMEOUT), otherwise we drop connection.
    pub notif_room: &'static str, // user has joined in to this room 
    pub peer_name: Option<String>, // user mongodb id
    pub ws_role_notif_actor_address: Addr<RoleNotifServer>, // the role notif actor server address,
    pub app_storage: Option<Arc<Storage>>,
    pub is_subscription_interval_started: bool
}


impl WsNotifSession{

    /* 
        role subscription process is done using the redis async subscriber 
        which subscribes asyncly to the incoming future io object streams 
        from the passed in channel 
    */
    pub async fn role_subscription(notif_room: &'static str, session_id: usize,
        peer_name: String, redis_async_pubsubconn: Arc<PubsubConnection>,
        ws_role_notif_actor_address: Addr<RoleNotifServer>){

        /* cloning vars that are going to be captured by tokio::spawn(async move{}) cause we need their owned types */
        let cloned_notif_room = notif_room.clone();
        let redis_async_pubsubconn = redis_async_pubsubconn.clone();
        let ws_role_notif_actor_address = ws_role_notif_actor_address.clone();
        let peer_name = peer_name.clone();

        tokio::spawn(async move{

            info!("💡 --- peer [{}] is subscribing to event room: [{}] at time [{}]", peer_name, notif_room, chrono::Local::now().timestamp_nanos());

            /* 🚨 !!! 
                we must receive asyncly from the redis subscription streaming 
                channel otherwise actor gets halted in here since using sync 
                redis and actor redis cause the current thread gets halted
                because they'll receive in a blocking manner, thus we must 
                use tokio::spawn() to do so.
            !!! 🚨 */
            let get_stream_messages = redis_async_pubsubconn
                .subscribe(&cloned_notif_room)
                .await;
            
            let Ok(mut get_stream_messages) = get_stream_messages else{

                use error::{ErrorKind, StorageError::RedisAsync, PanelError};
                let e = get_stream_messages.unwrap_err();
                let error_content = e.to_string().as_bytes().to_vec();  
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(RedisAsync(e)));
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                return ();

            };
        
            /* 
                iterating through the msg future object streams as they're 
                coming to the stream channel, we select the some ones
            */
            while let Some(message) = get_stream_messages.next().await{ 

                info!("💡 --- received revealed roles notif from admin at time: [{}]", chrono::Local::now().timestamp_nanos());
                
                let resp_val = message.unwrap();
                let stringified_player_roles = String::from_resp(resp_val).unwrap();
                let decoded_player_roles = serde_json::from_str::<Vec<PlayerRoleInfo>>(&stringified_player_roles).unwrap();

                /* sending the received roles to each session separately as a notification */
                for player_info in decoded_player_roles{
                    /* making sure that we're sending the role of this peer to the current session */
                    if player_info._id.to_string() == peer_name{
                        if let Err(why) = ws_role_notif_actor_address
                            .send(NotifySessionWithRedisSubscription{
                                notif_room: cloned_notif_room.to_string(), /* the event object id  */
                                role_name: player_info.role_name.unwrap(),
                                session_id,
                            }).await
                            {
                                error!("🚨 --- can't notify the peer cause by this mailbox error: [{}]", why);
                            }
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

/* notification event handler */
impl Handler<NotifySession> for WsNotifSession{

    type Result = ();

    fn handle(&mut self, msg: NotifySession, ctx: &mut Self::Context) -> Self::Result{
        
        info!("💡 --- sending revealed roles notif to peer [{}] in room: [{}]", msg.peer_name, msg.notif_room);
        ctx.text(msg.payload);
    }

}

/* event listener, handler or streamer to receive and handle ws byte messages */
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
                /* once we received the pong message we'll update the last heartbeat */
                self.hb = Instant::now(); /* updating the last heartbeat */
            },
            ws::Message::Text(text) => {

                let m = text.trim();
                if m.starts_with("/"){
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0]{

                        /* ------------------------------- */
                        /*    GET ONLINE ROOMS OR EVENTS   */
                        /* ------------------------------- */
                        "/events" => {

                            /* get all room from redis storage */
                            let mut redis_conn = self.app_storage.as_ref().clone().unwrap().get_redis_sync().unwrap().to_owned();
                            let redis_result_rooms_string: String = redis_conn.get("role_notif_server_actor_rooms").unwrap();
                            let rooms_in_redis = serde_json::from_str::<HashMap<String, HashSet<usize>>>(redis_result_rooms_string.as_str()).unwrap();
                            
                            /* sending to this peer */
                            ctx.text(format!("online events: {}", rooms_in_redis.len()));

                        },
                        /* ------------------------------- */
                        /*    JOIN TO RECEIVE PUSH NOTIF   */
                        /* ------------------------------- */
                        /* join the event notif room to subscribe to redis topics */
                        "/join-roles" => {

                            /* communicating with role notif server actor (notifs/role.rs) */

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
                                    info!("💡 --- subscribing to roles at interval [{}]", chrono::Local::now().timestamp_nanos());
                                    
                                    /* cloning the types that they need to be captured inside tokio::spawn() */
                                    let notif_room = actor.notif_room;
                                    let redis_async_pubsubconn = actor.app_storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn_sync().unwrap();
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

                            info!("💡 --- role subscription interval is already started, will notify this session if the role changes");

                        },
                        /* ------------------------------------*/
                        /*    JOIN TO RECEIVE ECQ PUSH NOTIF   */
                        /* ------------------------------------*/
                        "/join-ecq" => {
                            
                            /* communicating with ecq notif server actor (notifs/ecq.rs) */
                            
                            todo!()
                        
                        },
                        /* ------------------------------------*/
                        /*    JOIN TO RECEIVE MMR PUSH NOTIF   */
                        /* ------------------------------------*/
                        "/join-mmr" => {

                            /* communicating with mmr notif server actor (notifs/mmr.rs) */
                            /* 
                                mmq setup:
                                select 1 random player from the self.notif_room room
                                from the role notif server except self.peer_name 

                            */
                            
                            todo!()

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