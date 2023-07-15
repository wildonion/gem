


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
    pub notif_room: String,
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
    fn subscribe(&self, ctx: &mut Context<Self>, notif_room: &str){

        /* subscribing using none async redis */
        // let mut pubsub = self.redis_conn.as_pubsub();
        // pubsub.subscribe(notif_room.to_owned()).unwrap();
        // let msg = pubsub.get_message().unwrap();
        // let payload: String = msg.get_payload().unwrap();
 
        /* subscribing using redis actor which has async pubsub connection */
        let cloned_notif_room = notif_room.clone();
        self.redis_actor
            .send(Command(RespValue::Array(vec![
                RespValue::SimpleString("SUBSCRIBE".to_string()), 
                RespValue::SimpleString(cloned_notif_room.to_string())
            ])))
            .into_actor(self)
            .then(move |res, actor, ctx|{

                let payload = match res{
                    Ok(resp_val_result) => {

                        match resp_val_result.unwrap(){
                            RespValue::Array(mut resp_val_vec) => {
                                
                                let msg = resp_val_vec.pop();
                                if let Some(resp_val) = msg{
                                    match resp_val{
                                        RespValue::BulkString(bytes) => {
                                            let payload = serde_json::from_slice(&bytes).unwrap();
                                            payload
                                        },
                                        _ => String::from("")
                                    }
                                } else{
                                    String::from("")
                                }
                            
                            },
                            _ => String::from("")
                        }

                    },
                    Err(e) => {
                        "".to_string()
                    }
                };

                /* send payload to the role notif server actor using NotifySessionsWithRedisSubscription message */
                let cloned_notif_room = cloned_notif_room.clone();
                actor.role_notif_server_actor
                    .do_send(NotifySessionsWithRedisSubscription{
                        notif_room: cloned_notif_room.to_string().clone(),
                        payload,
                        subscribed_at: chrono::Local::now().timestamp_nanos() as u64
                    });

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
            then use those cloned vars inside the closure by passing them into the closure 
        */
        let notif_room = msg.notif_room.clone();
        // ctx.run_interval(WS_HEARTBEAT_INTERVAL, move |actor, ctx|{
            self.subscribe(ctx, &notif_room);
        // });
            
    }

}

