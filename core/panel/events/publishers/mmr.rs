



/*

    https://github.com/wildonion/penpineappleapplepen/blob/main/src/lib.rs
    https://github.com/wildonion/gvm/blob/main/src/lib.rs

    complete proc macros in lib.rs

*/



use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};
use tiny_keccak::keccak256;
use uuid::Uuid;





#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerRank{
    pub screen_cid: String, /* crypto id usally public address */
    pub rank: u16, /* one of the criterion is player status during the game */
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrentMatch{
    pub event_id: String,
    pub players: Vec<PlayerRank>,
    pub room_id: String,
    pub is_locked: bool
}

// fire/emit/publish UserNotif events through the ws channels
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserNotif{
    user_id: String,
    notifs: Vec<NotifData>,
    updated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
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
    fn get(&mut self) -> Self{
        let this = UserNotif { ..Default::default() };
        this
    }
}

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