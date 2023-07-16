


/*   -----------------------------------------------------------------------------------
    | redis subscription actor to subscribe to the event 
    | ----------------------------------------------------------------------------------
    |
    |
*/

pub mod ecq;
pub mod role;
pub mod task;
pub mod mmr;



use crate::{misc::*, apis::notifs::notif_subs};
use crate::*;
use actix::prelude::*;
use crate::constants::{WS_REDIS_SUBSCIPTION_INTERVAL, WS_HEARTBEAT_INTERVAL};
use super::ws::notifs::role::{RoleNotifServer, NotifySessionsWithRedisSubscription, RedisDisconnect};


/* implementing Message traits for all type of messages that can be used by RedisSubscription actor */

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub notif_room: &'static str,
}


/* RedisSubscription contains all the event rooms and sessions or peers that are connected to ws connection */
pub(crate) struct RedisSubscription{
    pub redis_conn: redis::Connection,
    pub role_notif_server_actor: Addr<RoleNotifServer>,
    pub redis_actor: Addr<RedisActor>
}

impl RedisSubscription{

    pub fn new(redis_conn: redis::Connection, 
        role_notif_server_actor: Addr<RoleNotifServer>,
        redis_actor: Addr<RedisActor>
    ) -> Self{

        RedisSubscription{
            redis_conn,
            role_notif_server_actor,
            redis_actor
        }
    }
    
}

impl RedisSubscription{

    /* send the passed in message to all session actors in a specific event room */
    fn subscribe(&self, ctx: &mut Context<Self>, notif_room: &'static str){
 
        /* subscribing using redis actor which has async pubsub connection */
        self.redis_actor
            .send(Command(RespValue::Array(vec![
                RespValue::SimpleString("SUBSCRIBE".to_string()), 
                RespValue::SimpleString(notif_room.to_string())
            ])))
            .into_actor(self)
            .then(|res, actor, ctx|{

                match res{
                    Ok(resp_val_result) => {

                        match resp_val_result.unwrap(){

                            /* SUBSCRIBE command returns a vector of 3 types which are message, topic and message-type */
                            RespValue::Array(mut resp_val_vec) => {

                                /*‌ first pop() is the message that we're interested in */
                                let msg = resp_val_vec.pop();
                                if let Some(resp_val) = msg{
                                    match resp_val{

                                        /* getting the utf8 bytes slice of the message */
                                        RespValue::BulkString(bytes) => {

                                            /* decoding the message to get the payload */
                                            let payload = serde_json::from_slice(&bytes).unwrap();

                                            /* send payload to the role notif server actor using NotifySessionsWithRedisSubscription message */
                                            actor.role_notif_server_actor
                                            .do_send(NotifySessionsWithRedisSubscription{
                                                notif_room: notif_room.to_string(),
                                                payload,
                                                subscribed_at: chrono::Local::now().timestamp_nanos() as u64
                                            });
                                        },
                                        _ => ctx.stop()
                                    }
                                } else{
                                    ctx.stop()
                                }
                            
                            },
                            _ => ctx.stop()
                        }

                    },
                    Err(e) => {
                        ctx.stop()
                    }
                }

                fut::ready(())

            })
            .wait(ctx);


        

    }

}

/* since this is an actor it can communicates with other ws actor as well, by sending pre defined messages to them */
impl Actor for RedisSubscription{
    type Context = Context<RedisSubscription>;

    fn started(&mut self, ctx: &mut Self::Context) {
        
        info!("💡 --- builtin redis actor started at [{}]", chrono::Local::now().timestamp_nanos());

    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        
        self.role_notif_server_actor.do_send(RedisDisconnect);
        Running::Stop
    }
}


impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for RedisSubscription{

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

        
    }
}


/* handlers for all type of messages for RedisSubscription actor */

impl Handler<Subscribe> for RedisSubscription{

    type Result = ();

    fn handle(&mut self, msg: Subscribe, ctx: &mut Self::Context) -> Self::Result{

        /* 
            since ctx.run_interval() accepts a closure that its captured vars must last 
            staticly thus we must clone necessary data from self to prevent ownership losing 
            then use those cloned vars inside the closure by passing them into the closure,
            also the actor itself will be returned from the closure so we can use it for 
            other method call since the actor will be captured by the closure in run_interval()
            method.
        */

        ctx.run_interval(WS_HEARTBEAT_INTERVAL, move |actor, ctx|{
            actor.subscribe(ctx, &msg.notif_room);
        });
            
    }

}

