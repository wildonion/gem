


pub mod ecq;
pub mod role;
pub mod task;
pub mod mmr;




/* role notif actor to communicate (send/receive messages) with session or peer actor */

use crate::misc::*;
use crate::*;
use actix::prelude::*;
use crate::constants::{WS_REDIS_SUBSCIPTION_INTERVAL, WS_HEARTBEAT_INTERVAL};
use super::ws::notifs::role::{RoleNotifServer, NotifySessionsWithRedisSubscription, RedisDisconnect};


/* implementing Message traits for all type of messages that can be used by RedisSubscription actor */

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub notif_room: String,
}


/* RedisSubscription contains all the event rooms and sessions or peers that are connected to ws connection */
pub(crate) struct RedisSubscription{
    pub redis_conn: redis::Connection,
    pub role_notif_server_actor: Addr<RoleNotifServer>,
}

impl RedisSubscription{

    pub fn new(redis_conn: redis::Connection, role_notif_server_actor: Addr<RoleNotifServer>) -> Self{

        RedisSubscription{
            redis_conn,
            role_notif_server_actor
        }
    }
    
}

impl RedisSubscription{

    /* send the passed in message to all session actors in a specific event room */
    fn subscribe(&mut self, ctx: &mut Context<Self>, notif_room: &str){

        let mut pubsub = self.redis_conn.as_pubsub();
        
        /* subscribing to redis topic */
        pubsub.subscribe(notif_room.to_owned()).unwrap();

        let msg = pubsub.get_message().unwrap();
        let payload: String = msg.get_payload().unwrap();


        /* send payload to the role notif server actor using NotifySessionsWithRedisSubscription message */
        self.role_notif_server_actor
            .do_send(NotifySessionsWithRedisSubscription{
                notif_room: notif_room.to_string().clone(),
                payload,
                subscribed_at: chrono::Local::now().timestamp_nanos() as u64
            });

    }

}

/* since this is an actor it can communicates with other ws actor as well, by sending pre defined messages to them */
impl Actor for RedisSubscription{
    type Context = Context<RedisSubscription>;

    fn started(&mut self, ctx: &mut Self::Context) {
        
        // ...

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

        /* 
            since ctx.run_interval() accepts a closure that its captured vars must last 
            staticly thus we must clone necessary data from self to prevent ownership losing 
            then use those cloned vars inside the closure by passing them into the closure 
        */
        let notif_room = msg.notif_room.clone();
        ctx.run_interval(WS_HEARTBEAT_INTERVAL, move |actor, ctx|{
            actor.subscribe(ctx, &notif_room);
        });
        
            
    }

}

