



/*

    https://github.com/wildonion/zoomate/blob/main/src/lib.rs
    https://github.com/wildonion/gvm/blob/main/src/lib.rs

    complete proc macros in lib.rs

*/



use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};
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
struct UserNotif<'info>{
    pub user_id: &'info str,
    pub notifs: Vec<NotifData<'info>>,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct NotifData<'info>{
    pub event_id: &'info str,
    pub fired_at: i64,
    pub seen: bool,
    pub topic: &'info str, // event name
    pub metadata: &'info str, // json string contains the actual data like fireing the player status (role and state changing) during the game 
}

impl<'info> UserNotif<'info>{
    fn set(&mut self, notif_data: NotifData<'info>) -> Self{
        self.notifs.push(notif_data);
        /* 
            call to clone() in `self.user_id.clone()` on a reference in this situation 
            does nothing the type `str` does not implement `Clone`, so calling `clone` 
            on `&str` copies the reference, which does not do anything and can be removed
        */
        let user_notif = UserNotif { user_id: self.user_id, notifs: self.notifs.clone(), updated_at: self.updated_at };
        UserNotif{
            ..user_notif /* filling all the fields with the user_notif ones */
        }
    }
    fn get(&mut self) -> Self{
        let this = UserNotif { ..Default::default() };
        this
    }
}

trait NotifExt<'info>{
    type Data;
    fn set_user_notif(&mut self, notif_data: NotifData<'info>) -> Self;
    fn get_user_notif(&self) -> Vec<NotifData<'info>>;
}

impl<'info> NotifExt<'info> for UserNotif<'info>{
    type Data = Self;

    fn get_user_notif(&self) -> Vec<NotifData<'info>> {
        self.notifs.clone()
    }

    fn set_user_notif(&mut self, new_notif: NotifData<'info>) -> Self { // since the set() method of the UserNotif instance is mutable this method must be mutable too
        self.set(new_notif)
    }

}

pub async fn push_notif(event_id: &str, fired_at: i64, seen: bool, topic: &str, metadata: &str){


    let usrer_notif = UserNotif{
        user_id: "",
        notifs: vec![
            NotifData{ 
                event_id, 
                fired_at, 
                seen, 
                topic, 
                metadata 
            }
        ],
        updated_at: 0,
    };


}