

/*   -----------------------------------------------------------------------------------
    | role notif actor to communicate (send/receive messages) with session or peer actor 
    | ----------------------------------------------------------------------------------
    |
    |
*/


use crate::constants::{WS_CLIENT_TIMEOUT, WS_REDIS_SUBSCIPTION_INTERVAL};
use crate::misc::*;
use crate::*;
use actix::prelude::*;


/* implementing Message traits for all type of messages that can be used by RoleNotifServer actor */

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

/// redis is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct RedisDisconnect;

/// join room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize, // client id or session id
    pub event_name: &'static str, // event room name: `reveal-role-{event_id}`
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);


/// update notif room
#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateNotifRoom(pub String);

/// redis subscription
#[derive(Message)]
#[rtype(result = "()")]
pub struct NotifySessionsWithRedisSubscription{
    pub notif_room: String,
    pub payload: String,
    pub subscribed_at: u64,
}



/* RoleNotifServer contains all the event rooms and sessions or peers that are connected to ws connection */
#[derive(Clone)]
pub(crate) struct RoleNotifServer{
    pub rooms: HashMap<String, HashSet<usize>>, // event rooms which is based on the event id or every event is a room
    pub sessions: HashMap<usize, Recipient<Message>>, // user in the event room, a mapping between session id and their actor address
    pub subscribed_at: u64,
    pub app_storage: Option<Arc<Storage>>, /* this app storage contains instances of redis, mongodb and postgres dbs so we have to make connections to use them */
}

impl RoleNotifServer{

    pub fn new(app_storage: Option<Arc<Storage>>) -> Self{

        RoleNotifServer{
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            subscribed_at: 0,
            app_storage,
        }
    }
    
}

impl RoleNotifServer{

    fn subscribe(&mut self, ctx: &mut Context<Self>, notif_room: &'static str){

        info!("💡 --- starting subscribing to event room: [{}]", notif_room);
        let redis_client = self.app_storage.as_ref().clone().unwrap().get_redis_sync().unwrap();
        let redis_actor = self.app_storage.as_ref().clone().unwrap().get_redis_actor_sync().unwrap();
        let redis_password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());

        let notif_room = notif_room.clone();
        let mut redis_conn = redis_client.get_connection().unwrap();


        let mut pubsub = redis_conn.as_pubsub();
        pubsub.subscribe(notif_room).unwrap();
        let msg = pubsub.get_message().unwrap();
        let payload: String = msg.get_payload().unwrap();


        self.subscribed_at = chrono::Local::now().timestamp_nanos() as u64;
        self.send_message(notif_room, payload.as_str(), 0);


        /* 
        
        let redis_auth_resp = redis_actor
            .send(Command(RespValue::Array(vec![
                RespValue::SimpleString("AUTH".to_string()), 
                RespValue::SimpleString(redis_password.to_string())
            ])));

        /* subscribing using redis actor which has async pubsub connection */
        redis_actor
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

                                            /* notify sessions with the subscribed topic related the to event room */
                                            actor.subscribed_at = chrono::Local::now().timestamp_nanos() as u64;
                                            actor.send_message(notif_room, payload, 0);
                                            
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
        */

    }

    /* send the passed in message to all session actors in a specific event room */
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

}

/* since this is an actor it can communicates with other ws actor as well, by sending pre defined messages to them */
impl Actor for RoleNotifServer{
    type Context = Context<RoleNotifServer>;
}


/* handlers for all type of messages for RoleNotifServer actor */

impl Handler<NotifySessionsWithRedisSubscription> for RoleNotifServer{

    type Result = ();

    fn handle(&mut self, msg: NotifySessionsWithRedisSubscription, ctx: &mut Self::Context) -> Self::Result{
        
        self.subscribed_at = msg.subscribed_at;
        info!("💡 --- sending subscribed revealed roles to room: [{}]", msg.notif_room);
        self.send_message(&msg.notif_room, &msg.payload, 0);
    }

}

impl Handler<UpdateNotifRoom> for RoleNotifServer{

    type Result = ();

    fn handle(&mut self, msg: UpdateNotifRoom, ctx: &mut Self::Context) -> Self::Result{
        
        /* insert the passed in room to the message object to the rooms of this actor */
        self.rooms
            .entry(msg.0.to_owned())
            .or_insert_with(HashSet::new);

    }

}

impl Handler<Disconnect> for RoleNotifServer{

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

impl Handler<RedisDisconnect> for RoleNotifServer{

    type Result = ();

    fn handle(&mut self, msg: RedisDisconnect, ctx: &mut Self::Context) -> Self::Result {
        
        info!("💡 --- redis actor is disconnected");
        let disconn_message = format!("push notif subscription actor is not available");
        
        /* 
            since self.rooms is behind a mutable reference and is 
            moving in each iteration of the loop, thus we must 
            borrow it in each iteration to prevent its ownership 
            from moving 
        */
        let rooms = &self.rooms;
        for room in rooms{
            self.send_message(&room.0, &disconn_message, 0);
        }
    }
}

impl Handler<Connect> for RoleNotifServer{

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
        
        info!("💡 --- current rooms of role notif server actor are: {:?}", self.rooms);

        let conn_message = format!("user with id: [{}] connected to event room: [{}]", unique_id, msg.event_name);
        info!("💡 --- user with id: [{}] connected to event room: [{}]", unique_id, msg.event_name);
        self.send_message(&msg.event_name, conn_message.as_str(), 0);
        
        unique_id /* session id */

    }
}

impl Handler<Join> for RoleNotifServer{ /* disconnect and connect again */

    type Result = ();

    /* in this handler we'll send disconnect message to old room and send join message to new room */
    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) -> Self::Result{
        
        let disconn_message = format!("user with id: [{}] disconnected from the event room: [{}]", msg.id, msg.event_name);
        let conn_message = format!("user with id: [{}] connected to event room: [{}]", msg.id, msg.event_name);

        let Join { id, event_name } = msg; // unpacking msg instance
        let mut rooms = Vec::<String>::new();

        /* removing session from all rooms of RoleNotifServer actor */
        for (event_room_name, sessions) in &mut self.rooms{
            if sessions.remove(&id){
                rooms.push(event_room_name.to_owned());
            }
        }

        /* send disconnect message to all rooms of RoleNotifServer actor and other user */
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