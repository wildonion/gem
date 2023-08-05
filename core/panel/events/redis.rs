


/*   -----------------------------------------------------------------------------------
    | redis subscription actor to subscribe to the event 
    | ----------------------------------------------------------------------------------
    |
    |
*/

pub mod ecq;
pub mod role;
pub mod mmr;
pub mod gvm;


use crate::{misc::*, apis::notifs::notif_subs};
use crate::*;
use actix::prelude::*;
use redis_async::resp::FromResp;
use crate::constants::WS_HEARTBEAT_INTERVAL;
use super::ws::notifs::role::{RoleNotifServer, NotifySessionsWithRedisSubscription, RedisDisconnect};


/* implementing Message traits for all type of messages that can be used by RedisSubscription actor */

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

impl Handler<Unsubscribe> for RedisSubscription{

    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, ctx: &mut Self::Context) -> Self::Result{

        self.redis_async_pubsubconn.unsubscribe("reveal-role-*");
            
    }

}

