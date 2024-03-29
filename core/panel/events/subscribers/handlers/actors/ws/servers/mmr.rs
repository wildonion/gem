

/*  > -----------------------------------------------------------------------------------
    | mmr notif actor to communicate (send/receive messages) with session or peer actor 
    | ----------------------------------------------------------------------------------
    | contains: message structures and their handlers
    | topic   : `mmr-{event_objectid}`   
    |
    |
*/

use crate::constants::WS_CLIENT_TIMEOUT;
use crate::helpers::misc::*;
use s3req::Storage;
use crate::*;
use actix::prelude::*;




/* implementing Message traits for all type of messages that can be used by MmrNotifServer actor  */

/// new chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>, /* user or session actor address */
    pub event_name: &'static str, // event room name: `mmr-{event_id}` to send message to and also user came to the room
    pub peer_name: String 
}

/// session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize, // session id
    pub event_name: String // event room name: `mmr-{event_id}` to send message to and also user disconnected from this room
}

/// join room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize, // client id or session id
    pub event_name: &'static str, // event room name: `mmr-{event_id}`
}

/// join room for push notif
#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinForPushNotif {
    pub id: usize, // client id or session id
    pub event_name: &'static str, // event room name: `mmr-{event_id}`
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


#[derive(Clone)]
pub struct MmrNotifServer{

    pub rooms: HashMap<String, HashSet<usize>>, // event rooms which is based on the event id or every event is a room
    pub sessions: HashMap<usize, Recipient<Message>>, // user in the event room, a mapping between session id and their actor address
    pub push_notif_rooms: HashMap<String, HashSet<usize>>,
    pub last_subscription_at: u64,
    pub app_storage: Option<Arc<Storage>>, /* this app storage contains instances of redis, mongodb and postgres dbs so we have to make connections to use them */

}

impl MmrNotifServer{

    pub fn new(app_storage: Option<Arc<Storage>>, ) -> Self{

        MmrNotifServer{
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            push_notif_rooms: HashMap::new(),
            last_subscription_at: 0,
            app_storage,
        }
    }
}

impl MmrNotifServer{

    fn send_message(&self, room: &str, message: &str, skip_id: usize){
        if self.rooms.contains_key(room){
            if let Some(sessions) = self.rooms.get(room){
                for id in sessions{
                    if *id.to_string() != skip_id.to_string(){
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

    fn send_push_notif_message(&self, room: &str, message: &str, skip_id: usize){
        if self.push_notif_rooms.contains_key(room){
            if let Some(sessions) = self.push_notif_rooms.get(room){
                for id in sessions{
                    if *id.to_string() != skip_id.to_string(){
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
    
}

impl Actor for MmrNotifServer{

    type Context = Context<MmrNotifServer>;

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        
        /* 
            if the mmr notif server actor gets stopped we don't want to subscribe to redis any more
            since mmr notif server contains all rooms and sessions thus all of them MUST not receive 
            any push notif from redis any more.
        */
        
        let async_redis = self.app_storage
            .as_ref()
            .clone()
            .unwrap()
            .get_async_redis_pubsub_conn_sync()
            .unwrap();
        async_redis.unsubscribe("mmr-*");

        Running::Stop
    }
}


/* handlers for all type of messages for MmrNotifServer actor to communicate with other actors */
impl Handler<JoinForPushNotif> for MmrNotifServer{

    type Result = ();

    fn handle(&mut self, msg: JoinForPushNotif, ctx: &mut Self::Context) -> Self::Result {
        
        let JoinForPushNotif { id, event_name } = msg; // unpacking msg instance

        self.push_notif_rooms
            .entry(event_name.to_string().clone())
            .or_insert_with(HashSet::new)
            .insert(id);

    }
}

impl Handler<UpdateNotifRoom> for MmrNotifServer{

    type Result = ();

    fn handle(&mut self, msg: UpdateNotifRoom, ctx: &mut Self::Context) -> Self::Result{

        /* 
            insert the passed in room to the message object to current rooms of this actor,
            if it doesn't exist it means that it's the first time we're creating the room
            thus we insert an empty hash set of peer idds otherwise we don't.
        */
        self.rooms
            .entry(msg.notif_room.to_owned())
            .or_insert_with(HashSet::new);

        let redis_client = self.app_storage.as_ref().clone().unwrap().get_redis_sync().unwrap();
        let mut conn = redis_client.get_connection().unwrap();
        
        /* caching rooms using redis */
        let redis_result_rooms: RedisResult<String> = conn.get("mmr_notif_server_actor_rooms");
        let redis_rooms = match redis_result_rooms{
            Ok(data) => {
                let rooms_in_redis = serde_json::from_str::<HashMap<String, HashSet<usize>>>(data.as_str()).unwrap();
                rooms_in_redis
            },
            Err(e) => {
                /*
                    we're cloning the self.rooms since we can't move it to the current_rooms var
                    while it's behind a mutable reference cause self is behind a mutable reference 
                    in method param, in general heap data types will be moved by default when we 
                    put them into another var to avoid expensive runtime operations thus we can't 
                    move them if they're behind a shared or mutable pointer.
                */
                let current_rooms = self.rooms.clone(); 
                let serialized_rooms = serde_json::to_string(&current_rooms).unwrap();
                let _: () = conn.set("mmr_notif_server_actor_rooms", serialized_rooms).unwrap();
                current_rooms
            }
        };


        /* updating the rooms in redis */
        let serialized_rooms = serde_json::to_string(&redis_rooms).unwrap();
        let _: () = conn.set("mmr_notif_server_actor_rooms", serialized_rooms).unwrap(); // writing to redis ram


    }

}

impl Handler<Disconnect> for MmrNotifServer{

    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result{
        
        info!("💡 --- user with id: [{}] disconnected from the event room: [{}]", msg.id, msg.event_name);
        let disconn_message = format!("user with id: [{}] disconnected from the event room: [{}]", msg.id, msg.event_name);
        let mut rooms = Vec::<String>::new();
        
        if self.sessions.remove(&msg.id).is_some(){
            /* 
                borrowing self.rooms mutably so we can mutate the self.sessions 
                when we mutate it's pointer, hence if we remove a session from 
                the session inside the self.rooms then the whole self.rooms 
                will be mutated, also since we're iteration over it, it's ownership
                will be moved in each iteration thus we must borrow it or take 
                a reference to it to prevent its ownership from moving
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

impl Handler<Connect> for MmrNotifServer{

    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result{
        
        /* insert new session */
        let mut r = rand::thread_rng();
        let unique_id = r.gen::<usize>();
        self.sessions.insert(unique_id, msg.addr);

        /* add this session to the event room name */
        self.rooms
            .entry(msg.event_name.to_string().clone())
            .and_modify(|session_ids|{ /* and_modify() will return a mutable reference to the type */
                /* 
                    since session_ids is a mutable reference to the value of self.rooms 
                    thus by mutating it the value of self.rooms will be mutated too
                */
                session_ids.insert(unique_id);
            })
            .or_insert(HashSet::new());
        
        info!("💡 --- current rooms of mmr notif server actor are: {:?}", self.rooms);

        let conn_message = format!("user with id: [{}] and peer name: [{}] connected to event room: [{}]", unique_id, msg.peer_name, msg.event_name);
        info!("💡 --- user with id: [{}] and peer name: [{}] connected to event room: [{}]", unique_id, msg.peer_name, msg.event_name);
        self.send_message(&msg.event_name, conn_message.as_str(), 0);

        unique_id /* session id */

    }
}

impl Handler<Join> for MmrNotifServer{ /* disconnect and connect again */

    type Result = ();

    /* in this handler we'll send disconnect message to old room and send join message to new room */
    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) -> Self::Result{
        
        let disconn_message = format!("user with id: [{}] disconnected from the event room: [{}]", msg.id, msg.event_name);
        let conn_message = format!("user with id: [{}] connected to event room: [{}]", msg.id, msg.event_name);

        let Join { id, event_name } = msg; // unpacking msg instance
        let mut rooms = Vec::<String>::new();

        /* removing session from all rooms of MmrNotifServer actor */
        for (event_room_name, sessions) in &mut self.rooms{
            if sessions.remove(&id){
                rooms.push(event_room_name.to_owned());
            }
        }

        /* send disconnect message to all rooms of MmrNotifServer actor and other user */
        for room in rooms{
            self.send_message(&room, &disconn_message, 0);
        }

        /* insert the user into the event room */
        self.rooms  
            .entry(event_name.to_string().clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        /* notify other session in that room that a user has connected */
        self.send_message(&event_name, conn_message.as_str(), 0);


    }
}