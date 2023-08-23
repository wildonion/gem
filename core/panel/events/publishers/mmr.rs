



/*

    https://github.com/wildonion/redis4
    
    actix ws actor event and stream handler/loop using tokio spawn, select, mpsc, mutex and tcp

    ------------------------------------------------
    networking(actor, ws, redis pubsub and streams):
    ------------------------------------------------
        event or async task handler, streamer, loop 
        inside std::thread::scope and tokio::spawn based 
        tokio tcp stream or mmq streaming over future 
        bytes using tokio and ws actor and redis pubsub 
        and streams by streaming over incoming bytes 
        inside the tokio gread threadpool and pass them 
        to other threads using tokio::sync::mpsc, actor, 
        select, spawn, mutex, pubsub, tcp stream, hex, serding 
        )to_string vs from utf8)
        tokio::spawn(async move{
            while let Ok(data) = streamer.recv().await{
                /* decode the bytes to a struct; see redis4 repo */
                let decoded;
                sender.send(decoded)
            }
        });


    mmr, mmq and ecq notif server actor setup for players
    redis pubsub streaming structure for publishing ecq (for registered events) and 
    mmr (for event suggestion to players) topics inside `core/panel/events/publishers`
    folder along with their actor notifs structure inside 
    `core/panel/events/subscribers` folder

    ws actor stream and event handlers are like:
        streaming over incoming bytes through the tokio tcp socket 
        to send them as the async task to tokio green threadpool using
        tokio spawn to handle them as an event using tokio select event 
        loop handler

*/


use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};
use uuid::Uuid;





#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CollaborationQueue{
    pub players: Vec<Player>, // user pool that can be used to start a match between them
    pub event_id: String, 
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player{
    pub pub_key: String,
    pub rank: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrentMatch{
    pub event_id: String,
    pub players: Vec<Player>,
    pub room_id: String,
    pub is_locked: bool
}


fn serding(){
    #[derive(Serialize, Deserialize, Clone)]
    struct HexStringEx{
        pub name: String,
    }
    let mut instance = HexStringEx{name: "wildonion".to_string()};
    let string = serde_json::to_string(&instance).unwrap();
    let bytes = string.as_bytes();
    let hex_string = hex::encode(bytes);

    let rev_bytes = hex::decode(hex_string).unwrap();
    let rev_instance = serde_json::from_slice::<HexStringEx>(&rev_bytes).unwrap();

    let instance_name_encoded = rev_instance.name.as_bytes();
    let instance_name_decoded = std::str::from_utf8(instance_name_encoded).unwrap().to_string();
}

// fire/emit/publish UserNotif events in ws/rpc/zmq server or using redis
// like using emit!(UserNotif{}) macro which emit and fire an
// event through the redis streaming channel to clients 
// sub or listen to UserNotif events in ws/rpc/zmq client
// using an event loop or listener.
// update UserNotif on every data changes through its related api calls
// then fire the updated data event through the ws server so the client
// can subs using ws to the fired event 
#[derive(Serialize, Deserialize, Clone)]
pub struct UserNotif{
    user_id: String,
    notifs: Vec<NotifData>,
    updated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NotifData{
    fired_at: Option<i64>,
    seen: bool,
    topic: String, // json string contains the actual data like fireing the player status (role and state changing) during the game 
}

impl UserNotif{
    fn set(&mut self, notif_data: NotifData) -> Self{
        self.notifs.push(notif_data);
        let user_notif = UserNotif { user_id: self.user_id.clone(), notifs: self.notifs.clone(), updated_at: self.updated_at };
        UserNotif{
            ..user_notif /* filling all the fields with the user_notif ones */
        }
    }
}


// in order to call the NotifExt methods on the
// UserNotif struct the trait must be implemented 
// for the UserNotif struct and imported inside
// where we want to call the methods on the struct
// instance.
pub trait NotifExt{
    type Data;
    fn set_user_notif(&mut self, notif_data: NotifData) -> Self;
    fn get_user_notif(&self) -> Vec<NotifData>;
}

impl NotifExt for UserNotif{
    type Data = Self;

    fn get_user_notif(&self) -> Vec<NotifData> {
        self.notifs.clone()
    }

    fn set_user_notif(&mut self, new_notif: NotifData) -> Self { // since the set() method of the UserNotif instance is mutable this method must be mutable too
        self.set(new_notif)
    }

}

#[derive(Serialize, Deserialize, Clone)]

pub struct Royalty{
    pub wallet_address: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Clone)]

pub struct Nft{
    pub owner: String,
    pub royalties: Vec<Royalty>, // Royalty struct must be public if this field is public since we want to access this field later which contains the Royalty instances
    pub events: Vec<UserNotif>
}