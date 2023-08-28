



/* 

    `mmr-{event_id}`

*/

use actix::Context;
use crate::*;
use actix::prelude::*;




#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect{

}


/* handlers for all type of messages for MmrNotifServer actor to communicate with other actors */
impl Handler<Connect> for MmrNotifServer{

    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result{
           
    }

}


#[derive(Clone)]
pub struct MmrNotifServer{
    
}

impl Actor for MmrNotifServer{

    type Context = Context<MmrNotifServer>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MmrNotifServer{


    /* the handler method to handle the incoming websocket messages by decoding them */
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context){

    }

}