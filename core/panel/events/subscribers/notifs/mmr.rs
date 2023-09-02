

/*   -----------------------------------------------------------------------------------
    | mmr notif actor to communicate (send/receive messages) with session or peer actor 
    | ----------------------------------------------------------------------------------
    |
    | topic: `mmr-{event_objectid}`   
    |
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

/* stream or event handler to handle the incoming websocket packets in realtime */
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MmrNotifServer{


    /* the handler method to handle the incoming websocket messages by decoding them */
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context){

    }

}