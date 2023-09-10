

/*   -----------------------------------------------------------------------------------
    | ecq notif actor to communicate (send/receive messages) with session or peer actor 
    | ----------------------------------------------------------------------------------
    |
    | topic: `ecq-{event_objectid}`  
    |
*/


use crate::constants::WS_CLIENT_TIMEOUT;
use crate::misc::*;
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


        let cq_instance: events::publishers::mmr::CollaborationQueue = Default::default();
        let cq = events::publishers::mmr::CollaborationQueue{..Default::default()}; // filling all the fields with default values 

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