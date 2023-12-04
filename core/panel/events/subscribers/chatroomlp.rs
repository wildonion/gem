

/*  > -----------------------------------------------------------------------------------------------------
    | chatroom launchpad server actor to communicate (send/receive messages) with sessions or peer actors 
    | -----------------------------------------------------------------------------------------------------
    | contains: message structures and their handlers
    | 
    | each session actor can be used to communicate using pre defined mesaages
    | to send message to other actors also server actor contains all session actor 
    | in which we can use the message sending pattern for pubsub push notif subscribing 
    | to an specific topic using redis in server actor, like if we want to 
    | notify all sessions about a new session since the server actor has all sessions
    | we should send a new session income message to the server actor once a new 
    | session gets connected to the socket and server can send to all session actors 
    | the new income message 
    |
*/


use crate::constants::WS_CLIENT_TIMEOUT;
use crate::misc::*;
use s3req::Storage;
use crate::*;
use actix::prelude::*;
use actix_broker::*;


/* implementing Message traits for all type of messages that can be used by ChatRoomLaunchpadServer actor to communicate with other actors */

/// new chat session is created
#[derive(Clone, Message)]
#[rtype(String)]
pub struct Connect {
    pub addr: Recipient<Message>, /* user or session actor address */
    pub chatroom_name: &'static str, // chatroom name: `chatroomlp-{clp_id}` to send message to and also user came to the room
    pub peer_name: String 
}

/// session is disconnected
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String, // session screen_cid
    pub chatroom_name: String // chatroom name: `chatroomlp-{clp_id}` to send message to and also user disconnected from this room
}

/// list rooms
#[derive(Clone, Message)]
#[rtype(result = "Rooms")]
pub struct ListRooms;

#[derive(MessageResponse)]
pub struct Rooms(pub HashMap<String, HashSet<String>>);

/// join room 
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: String, // client id or session screen_cid
    pub chatroom_name: &'static str, // chatroom name: `chatroomlp-{clp_id}`
}

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct Message(pub String);


/// update chat room
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct UpdateChatRoom{
    pub chat_room: &'static str,
    pub peer_name: String
}

/// notify all sessions
 #[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct NotifySessionsWithNewMessage{
    pub chat_room: String,
    pub session_id: String,
    pub new_message: String,
}

/* ChatRoomLaunchpadServer contains all the chatrooms and sessions or peers that are connected to ws connection */
#[derive(Clone, Default)]
pub struct ChatRoomLaunchpadServer{
    pub rooms: HashMap<String, HashSet<String>>, // a mapping between the room or event name and its peer ids
    pub sessions: HashMap<String, Recipient<Message>>, // a mapping between the peer id and its actor address
    pub app_storage: Option<Arc<Storage>>, /* this app storage contains instances of redis, mongodb and postgres dbs */
}

impl ChatRoomLaunchpadServer{

    pub fn new(app_storage: Option<Arc<Storage>>, ) -> Self{

        ChatRoomLaunchpadServer{
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            app_storage,
        }
    }

    /* ----------------------------- */
    /*   caching rooms using redis   */
    /* ----------------------------- */
    pub fn cache_room(&mut self){

        /* we'll use this to fetch users' info in a room */
        let redis_client = self.app_storage.as_ref().clone().unwrap().get_redis_sync().unwrap();
        let mut conn = redis_client.get_connection().unwrap();

        /* updating the rooms in redis */
        let current_rooms = self.rooms.clone();
        let serialized_rooms = serde_json::to_string_pretty(&current_rooms).unwrap();
        let _: () = conn.set("chatroomlp_server_actor_rooms", serialized_rooms).unwrap(); // writing to redis ram

    }
    
}

impl ChatRoomLaunchpadServer{

    /* send the passed in message to all session actors in a specific chatroom */
    fn send_message(&self, room: &str, message: &str, skip_id: String){
        if self.rooms.contains_key(room){
            if let Some(sessions) = self.rooms.get(room){
                for id in sessions{
                    if *id.to_string() != skip_id.to_string(){
                        if let Some(addr) = self.sessions.get(id){
                            /* 
                                a handler with generic Message must be implemented for each session 
                                or WsLaunchpadSession actor do_send() will send the message to the session 
                                or WsLaunchpadSession actor later we can handle it using the implemented handler 
                            */
                            addr.do_send(Message(message.to_owned())) 
                        }
                    }
                }
            }
        }
    }

}

impl SystemService for ChatRoomLaunchpadServer {}
/*
    by implementing the following trait we're giving the
    ChatRoomLaunchpadServer actor the ability to restart after failure
*/
impl Supervised for ChatRoomLaunchpadServer {}

/* since this is an actor it can communicates with other ws actor as well, by sending pre defined messages to them */
impl Actor for ChatRoomLaunchpadServer{
    /* 
        Context is a wrapper around the Self which can be used 
        as a little instance of the Self in some places that we 
        need to access the whole instance
    */
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        
        /*  ----------------------------------------------------------------------------------
            > subscribing to Join and NotifySessionsWithNewMessage messages once
            the server actor gets started,
            by loading actix_broker::BrokerSubscribe trait we have access to 
            the traits' methods inside each actor since it's already implemented 
            for each actor instance, and by calling subscribe_async() method 
            we can asynchronously subscribe to a message that this actor is 
            interested to, basically actix_broker is used to facilitate the sending 
            of some messages between the session and server actors where the session 
            does not require a response thus in the following the server actor 
            is subscribing to Disconnect and NotifySessionsWithNewMessage messages
            asyncly and automatically one a session gets disconnected from the room 
            and notified with a new message 

            > subscribing to Join message might show no session screen_cid cause this subscription
            is async and if the server is subscribing to this message before the session
            gets joined to chat the session screen_cid will be 0 like: 
                user with id: [0] connected to chatroom: [1]

            > subscribing to NotifySessionsWithNewMessage message causes client to see
            the incoming message in the room twice because we're notifying the server
            actor two times with new message, first one is in sending ws message logic 
            and the second one is in here

            > note that once server actor gets subscribed to these messages all the logs
            inside each message handler will be shown and sent to the client
        */
        self.subscribe_system_async::<Join>(ctx);
        // self.subscribe_system_async::<NotifySessionsWithNewMessage>(ctx);

    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        Running::Stop
    }
}


/* handlers for all type of messages for ChatRoomLaunchpadServer actor to communicate with other actors */

impl Handler<UpdateChatRoom> for ChatRoomLaunchpadServer{

    type Result = ();

    fn handle(&mut self, msg: UpdateChatRoom, ctx: &mut Self::Context) -> Self::Result{

        /* 
            insert the passed in room to the message object to current rooms of this actor,
            if it doesn't exist it means that it's the first time we're creating the room
            thus we insert an empty hash set of peer ids otherwise we don't.
        */
        self.rooms
            .entry(msg.chat_room.to_owned())
            .or_insert_with(HashSet::new);

        
        self.cache_room();

    }

}

impl Handler<Disconnect> for ChatRoomLaunchpadServer{

    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result{
        
        info!("💡 chatroomlp --- user with id: [{}] disconnected from the chatroom: [{}]", msg.id, msg.chatroom_name);
        let disconn_message = format!("chatroomlp::user with id: [{}] disconnected from the chatroom: [{}]", msg.id, msg.chatroom_name);
        let mut rooms = Vec::<String>::new(); /* allocating a new room */
        
        if self.sessions.remove(&msg.id).is_some(){
            /* 
                borrowing self.rooms mutably so we can mutate the self.sessions 
                when we mutate it's pointer, hence if we remove a session from 
                the session inside the self.rooms then the whole self.rooms 
                will be mutated, also since we're iteration over it, it's ownership
                will be moved in each iteration thus we must borrow it or take 
                a reference to it to prevent its ownership from moving
            */
            for (chatroom_name_room, sessions) in &mut self.rooms{
                if sessions.remove(&msg.id){
                    rooms.push(chatroom_name_room.to_owned()); /* to_owned() and clone() return Self */
                }
            }

            for chatroom_name_room in rooms{
                self.send_message(&chatroom_name_room, disconn_message.as_str(), msg.id.clone());
            }

            self.cache_room();
        }

    }
    
}

impl Handler<Connect> for ChatRoomLaunchpadServer{

    type Result = String; /* return type is the generated session screen_cid */

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result{
        
        // generating unique random session id 
        let mut r = rand::thread_rng();
        let random_unique_id = r.gen::<usize>();
        
        /* insert new session using the screen_cid */
        let unique_id = msg.peer_name.clone();
        self.sessions.insert(unique_id.clone(), msg.addr);

        /* add this session to the chatroom name */
        self.rooms
            .entry(msg.chatroom_name.to_string().clone())
            .and_modify(|session_ids|{ /* and_modify() will return a mutable reference to the type */
                /* 
                    since session_ids is a mutable reference to the value of self.rooms 
                    thus by mutating it the value of self.rooms will be mutated too
                */
                session_ids.insert(unique_id.clone());
            })
            .or_insert(HashSet::new());
        
        info!("💡 chatroomlp --- user with id: [{}] connected to chatroom: [{}]", unique_id, msg.peer_name);
        info!("💡 chatroomlp --- current rooms of chatroom server actor are: {:?}", self.rooms);

        let conn_message = format!("chatroomlp::connect::user with id: [{}] connected to chatroom: [{}]", unique_id, msg.peer_name);
        self.send_message(&msg.chatroom_name, conn_message.as_str(), msg.peer_name.clone());

        self.cache_room();

        unique_id /* session screen_cid */

    }

}


impl Handler<ListRooms> for ChatRoomLaunchpadServer{

    type Result = Rooms;

    fn handle(&mut self, msg: ListRooms, ctx: &mut Self::Context) -> Self::Result{
        let current_rooms = self.rooms.clone();
        Rooms(current_rooms)
    }
}

/* this handler will be triggered once a session actor started */
impl Handler<Join> for ChatRoomLaunchpadServer{ /* disconnect and connect again */

    type Result = ();

    /* in this handler we'll send disconnect message to old room and send join message to new room */
    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) -> Self::Result{
        
        let disconn_message = format!("chatroomlp::join::user with id: [{}] disconnected from the chatroom: [{}]", msg.id, msg.chatroom_name);
        let conn_message = format!("chatroomlp::join::user with id: [{}] connected to chatroom: [{}]", msg.id, msg.chatroom_name);

        /* 
            id will be moved later, we're borrowing it in here using ref 
            so ref id is a reference to or the borrowed form of the actual
            id field.
        */
        let Join { ref id, chatroom_name } = msg; // unpacking msg instance
        let mut rooms = Vec::<String>::new();

        /* removing session from all rooms of ChatRoomLaunchpadServer actor */
        for (event_room_name, sessions) in &mut self.rooms{
            if sessions.remove(id){ /* sessions will be updated since we have a mutable pointer to it */
                rooms.push(event_room_name.to_owned());
            }
        }
        

        /* send disconnect message to all rooms of ChatRoomLaunchpadServer actor and other user */
        for room in rooms{
            self.send_message(&room, &disconn_message, msg.id.clone());
        }

        /* insert the user into the chatroom */
        self.rooms  
            .entry(chatroom_name.to_string().clone())
            .or_insert_with(HashSet::new)
            .insert(id.to_owned());

        /* notify other session in that room that a user has connected */
        self.send_message(&chatroom_name, conn_message.as_str(), msg.id.clone());

        self.cache_room();

    }

}

/* this handler will be triggered once a user sent a new message to the server and the server will notify other peers */
impl Handler<NotifySessionsWithNewMessage> for ChatRoomLaunchpadServer{

    type Result = ();

    fn handle(&mut self, msg: NotifySessionsWithNewMessage, ctx: &mut Self::Context) -> Self::Result {

        let NotifySessionsWithNewMessage{ chat_room, session_id, new_message }
            = msg;

        /* notify all peers except the one who sent the message */
        self.send_message(&chat_room, &new_message, session_id);

    }
}