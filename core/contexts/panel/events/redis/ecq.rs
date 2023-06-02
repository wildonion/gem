


// https://github.com/wildonion/redis4
// admins or gods can collaborate with others and share their events using redis based ecq 
// fire, emit or publish a redis event or topic like ecq event



use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};
use uuid::Uuid;





#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CollaborationQueue{
    pub players: Vec<Player>, //// user pool that can be used to start a match between them
    pub event_id: String, 
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Player{
    pub pub_key: String,
    pub rank: u16,
}

pub struct CurrentMatch{
    pub event_id: String,
    pub players: Vec<Player>,
    pub room_id: String,
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
    topic: String, //// json string contains the actual data like fireing the player status (role and state changing) during the game 
}

impl UserNotif{
    fn set(&mut self, notif_data: NotifData) -> Self{
        self.notifs.push(notif_data);
        UserNotif { user_id: self.user_id.clone(), notifs: self.notifs.clone(), updated_at: self.updated_at }
    }
}


//// in order to call the NotifExt methods on the
//// UserNotif struct the trait must be implemented 
//// for the UserNotif struct and imported inside
//// where we want to call the methods on the struct
//// instance.
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

    fn set_user_notif(&mut self, new_notif: NotifData) -> Self { //// since the set() method of the UserNotif instance is mutable this method must be mutable too
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
    pub royalties: Vec<Royalty>, //// Royalty struct must be public if this field is public since we want to access this field later which contains the Royalty instances
    pub events: Vec<UserNotif>
}