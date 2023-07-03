


use crate::misc::*;
use crate::*;
use actix::prelude::*;


/* implementing Message traits for all type of messages that can be used by RoleNotifServer actor */

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe(pub String);

/// new chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize, // session id
}

/// join room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize, // client id or session id
    pub event_name: String, // event room name: `reveal-role-{event_id}`
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);



#[derive(Clone)]
pub(crate) struct RoleNotifServer{
    pub event: String, // `reveal-role-{event_id}`
    pub subscribed_at: i64, // time of redis subscription
    pub rooms: HashMap<String, HashSet<usize>>, // event rooms which is based on the event id or every event is a room
    pub sessions: HashMap<usize, Recipient<Message>>, // users in the event room
    pub app_storage: Option<Arc<Storage>>,
}

impl RoleNotifServer{

    pub fn new(app_storage: Option<Arc<Storage>>) -> Self{

        let mut rooms = HashMap::new();
        rooms.insert("notif".to_owned(), HashSet::new());
        RoleNotifServer{
            subscribed_at: 0,
            sessions: HashMap::new(),
            rooms,
            event: String::from(""),
            app_storage,
        }
    }
    
}

impl RoleNotifServer{

    /* send the passed in message to all session actors in a specific event room */
    fn send_message(&self, room: &str, message: &str, skip_id: usize){
        if let Some(sessions) = self.rooms.get(room){
            for id in sessions{
                if *id.to_string() == skip_id.to_string(){
                    if let Some(addr) = self.sessions.get(id){
                        addr.do_send(Message(message.to_owned())) /* do_send() will send the message to the actor */
                    }
                }
            }
        }
    }
}

impl Actor for RoleNotifServer{
    type Context = ws::WebsocketContext<RoleNotifServer>;
}


/* handlers for all type of messages for RoleNotifServer actor */

impl Handler<Subscribe> for RoleNotifServer{

    type Result = ();

    fn handle(&mut self, msg: Subscribe, ctx: &mut Self::Context) -> Self::Result{
        
        /* subscribe to redis topic then send this to all actors */

        todo!()

    }
}

impl Handler<Disconnect> for RoleNotifServer{

    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result{
        
        todo!()

    }
}

impl Handler<Connect> for RoleNotifServer{

    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result{
        
        todo!()

    }
}

impl Handler<Join> for RoleNotifServer{

    type Result = ();

    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) -> Self::Result{
        
        todo!()

    }
}