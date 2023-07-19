


/*   -----------------------------------------------------------------------------------
    | redis subscription actor to subscribe to the event 
    | ----------------------------------------------------------------------------------
    |
    |
*/

pub mod ecq;
pub mod role;
pub mod mmr;



use crate::{misc::*, apis::notifs::notif_subs};
use crate::*;
use actix::prelude::*;
use redis_async::resp::FromResp;
use crate::constants::{WS_REDIS_SUBSCIPTION_INTERVAL, WS_HEARTBEAT_INTERVAL};
use super::ws::notifs::role::{RoleNotifServer, NotifySessionsWithRedisSubscription, RedisDisconnect};


/* implementing Message traits for all type of messages that can be used by RedisSubscription actor */

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub notif_room: &'static str,
    pub peer_name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Unsubscribe;


/* RedisSubscription contains all the event rooms and sessions or peers that are connected to ws connection */
pub struct RedisSubscription{
    pub redis_async_pubsubconn: Arc<PubsubConnection>,
    pub role_notif_server_actor: Addr<RoleNotifServer>,
}

impl RedisSubscription{

    pub fn new(redis_async_pubsubconn: Arc<PubsubConnection>, 
        role_notif_server_actor: Addr<RoleNotifServer>,
    ) -> Self{

        RedisSubscription{
            redis_async_pubsubconn,
            role_notif_server_actor,
        }
    }
    
}

impl RedisSubscription{

    fn subscribe(&self, ctx: &mut Context<Self>, msg: Subscribe){

        /* cloning vars that are going to be captured by tokio::spawn(async move{}) */
        let cloned_notif_room = msg.notif_room.clone();
        let redis_async_pubsubconn = self.redis_async_pubsubconn.clone();
        let role_notif_server_actor = self.role_notif_server_actor.clone();
        let peer_name = msg.peer_name.clone();

        tokio::spawn(async move{
            
            info!("💡 --- peer [{}] is subscribing to event room: [{}]", peer_name, cloned_notif_room); /* using as_ref() to prevent from moving since unwrap() will take the ownership cause it has self in its first param */

            /* 🚨 !!! 
                we must receive asyncly from the redis subscription streaming 
                channel otherwise actor gets halted in here since 
            !!! 🚨 */
            let mut get_stream_messages = redis_async_pubsubconn
                .subscribe(&cloned_notif_room)
                .await
                .unwrap();
        
            /* iterating through the msg streams as they're coming to the stream channel while are not None */
            while let Some(message) = get_stream_messages.next().await{ 
                        
                let resp_val = message.unwrap();
                let message = String::from_resp(resp_val).unwrap();

                info!("💡 --- received revealed roles notif from admin");

                /* send payload to the role notif server actor using NotifySessionsWithRedisSubscription message */
                role_notif_server_actor
                    .send(NotifySessionsWithRedisSubscription{
                        notif_room: cloned_notif_room.to_string(),
                        payload: message,
                        last_subscription_at: chrono::Local::now().timestamp_nanos() as u64
                    }).await;

            }

            

        });

    }

}

/* since this is an actor it can communicates with other ws actor as well, by sending pre defined messages to them */
impl Actor for RedisSubscription{
    type Context = Context<RedisSubscription>;

    fn started(&mut self, ctx: &mut Self::Context) {
        
        info!("💡 --- panel redis actor started at [{}]", chrono::Local::now().timestamp_nanos());

    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        
        self.role_notif_server_actor.do_send(RedisDisconnect);
        Running::Stop
    }
}


/* handlers for all type of messages for RedisSubscription actor */

impl Handler<Subscribe> for RedisSubscription{

    type Result = ();

    fn handle(&mut self, msg: Subscribe, ctx: &mut Self::Context) -> Self::Result{

        self.subscribe(ctx, msg);
            
    }

}

impl Handler<Unsubscribe> for RedisSubscription{

    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, ctx: &mut Self::Context) -> Self::Result{

        self.redis_async_pubsubconn.unsubscribe("reveal-role-*");
            
    }

}

