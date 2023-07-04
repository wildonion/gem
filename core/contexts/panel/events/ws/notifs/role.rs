

/* role notif actor to communicate (send/receive messages) with session or peer actor */

use crate::misc::*;
use crate::*;
use actix::prelude::*;


/* implementing Message traits for all type of messages that can be used by RoleNotifServer actor */

/// new chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>, /* user or session actor address */
    pub event_name: String, // event room name: `reveal-role-{event_id}` to send message to and also user came to the room
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
    pub event_name: String, // event room name: `reveal-role-{event_id}`
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);



/* RoleNotifServer contains all the event rooms and sessions or peers that are connected to ws connection */
#[derive(Clone)]
pub(crate) struct RoleNotifServer{
    pub event: String, // `reveal-role-{event_id}`
    pub rooms: HashMap<String, HashSet<usize>>, // event rooms which is based on the event id or every event is a room
    pub sessions: HashMap<usize, Recipient<Message>>, // user in the event room
    pub app_storage: Option<Arc<Storage>>,
}

impl RoleNotifServer{

    pub fn new(app_storage: Option<Arc<Storage>>) -> Self{

        let mut rooms = HashMap::new();
        rooms.insert("notif".to_owned(), HashSet::new());
        RoleNotifServer{
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
                        /* 
                            a handler with generic Message must be implemented for each session 
                            or WsNotifSession actor do_send() will send the message to the session 
                            or WsNotifSession actor later we can handle it using the implemented handler 
                        */
                        addr.do_send(Message(message.to_owned())) 
                    }
                }
            }
        }
    }
}

/* since this is an actor it can communicates with other ws actor as well, by sending pre defined messages to them */
impl Actor for RoleNotifServer{
    type Context = Context<RoleNotifServer>;
}


/* handlers for all type of messages for RoleNotifServer actor */

impl Handler<Disconnect> for RoleNotifServer{

    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result{
        
        info!("user with id: [{}] disconnected from the event room: [{}]", msg.id, msg.event_name);

        let disconn_message = format!("user with id: [{}] disconnected from the event room: [{}]", msg.id, msg.event_name);
        let mut rooms = Vec::<String>::new();
        
        if self.sessions.remove(&msg.id).is_some(){
            /* 
                borrowing self.rooms mutably so we can mutate the self.sessions 
                when we mutate it's pointer, hence if we remove a session from 
                the session inside the self.rooms then the whole self.rooms 
                will be mutated
            */
            for (event_name_room, sessions) in &mut self.rooms{
                if sessions.remove(&msg.id){
                    rooms.push(event_name_room.to_owned()); /* to_owned() and clone() return Self */
                }
            }

            for event_name_room in rooms{
                self.send_message(&event_name_room, disconn_message.as_str(), 0);
            }
        }
    }
}

impl Handler<Connect> for RoleNotifServer{

    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result{
        
        /* 
            at the time of connecting to the ws server a peer has name 
            and don't have id since the id will be generated once the user
            gets slided into a room
        */
        let conn_message = format!("peer with name: [{}] came into the event room: [{}]", msg.peer_name, msg.event_name);

        info!("peer with name: [{}] came into the event room: [{}]", msg.peer_name, msg.event_name);

        self.send_message(&msg.event_name, conn_message.as_str(), 0);
        
        /* insert new session */
        let mut r = rand::thread_rng();
        let unique_id = r.gen::<usize>();
        self.sessions.insert(unique_id, msg.addr);

        /* add this session to the event room name */
        self.rooms
            .entry(msg.event_name)
            .or_insert_with(HashSet::new) // we must pass a Fn type to this method not by calling it like HashSet::new()
            .insert(unique_id);

        unique_id /* session id */

    }
}

impl Handler<Join> for RoleNotifServer{ /* disconnect and connect again */

    type Result = ();

    /* in this handler we'll send disconnect message to old room and send join message to new room */
    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) -> Self::Result{
        
        let disconn_message = format!("user with id: [{}] disconnected from the event room: [{}]", msg.id, msg.event_name);
        let conn_message = format!("user with id: {:?} came into the event room: [{}]", msg.id, msg.event_name);

       
        let Join { id, event_name } = msg; // unpacking msg instance
        let mut rooms = Vec::<String>::new();

        /* removing session from all rooms */
        for (event_room_name, sessions) in &mut self.rooms{
            if sessions.remove(&id){
                rooms.push(event_room_name.to_owned());
            }
        }

        /* send disconnect message to all rooms and other user */
        for room in rooms{
            self.send_message(&room, &disconn_message, 0);
        }

        /* insert the user into the event room */
        self.rooms  
            .entry(event_name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        /* notify other session in that room that a user has connected */
        self.send_message(&event_name, conn_message.as_str(), 0);

    }
}