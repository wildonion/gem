

/*   -----------------------------------------------------------------------------------
    | mmr notif actor to communicate (send/receive messages) with session or peer actor 
    | ----------------------------------------------------------------------------------
    |
    | topic: `mmr-{event_objectid}`   
    |
*/

use crate::constants::WS_CLIENT_TIMEOUT;
use crate::misc::*;
use crate::*;
use actix::prelude::*;




/* implementing Message traits for all type of messages that can be used by MmrNotifServer actor  */

/// new chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>, /* user or session actor address */
    pub event_name: &'static str, // event room name: `reveal-role-{event_id}` to send message to and also user came to the room
    pub peer_name: String 
}

/// session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize, // session id
    pub event_name: String // event room name: `reveal-role-{event_id}` to send message to and also user disconnected from this room
}

/// join room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize, // client id or session id
    pub event_name: &'static str, // event room name: `reveal-role-{event_id}`
}

/// join room for push notif
#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinForPushNotif {
    pub id: usize, // client id or session id
    pub event_name: &'static str, // event room name: `reveal-role-{event_id}`
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);


/// update notif room
#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateNotifRoom{
    pub notif_room: &'static str,
    pub peer_name: String
}


/* handlers for all type of messages for MmrNotifServer actor to communicate with other actors */
impl Handler<Connect> for MmrNotifServer{

    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result{

        todo!()
    }

}


#[derive(Clone)]
pub struct MmrNotifServer{
    
}

impl Actor for MmrNotifServer{

    type Context = Context<MmrNotifServer>;
}