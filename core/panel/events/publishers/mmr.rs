



/*
             https://github.com/wildonion/redis4

    - actix ws actor event and stream handler/loop using tokio spawn, 
        select, mpsc, mutex and tcp with redis and libp2p pubsub streams
    - event and stream handler to handle the incoming async task like ws 
        messages packets using actix StreamHandler and tokio tcp 
    - message handler to handle the message type which is going to 
        be sent between other actors
    - ws actor stream and event handlers are like:
        streaming over incoming bytes through the tokio tcp socket 
        to send them as the async task to tokio green threadpool using
        tokio spawn to handle them as an event using tokio select event 
        loop handler
    - players can get a higher rank by paying for the rank, playing with 
      more than 3 gods in a week, player ability limitation when the god 
      is updating the status 
    - an event manager (god) must mint a god card then he can set 
        a new proposal to make an event then those players whose 
        their ranks are matched together and have conse tokens will 
        be put inside the mmq to start voting in the event to mint 
        all the generated roles as a collection to start the game,
        players reserve event by upvoting in proposal using conse
        token and they'll earn new token after game (P2E)


    ------------------------------------------------
    networking(actor, ws, redis pubsub and streams):
    ------------------------------------------------
        event of async task handler, streamer, loop 
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

*/


use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};
use uuid::Uuid;





#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CollaborationQueue{
    pub players: Vec<PlayerRank>, // user pool that can be used to start a match between them
    pub event_id: String, 
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerRank{
    pub cid: String, /* crypto id usally pubkey */
    pub rank: u16, /* one of the criterion is player status during the game */
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrentMatch{
    pub event_id: String,
    pub players: Vec<PlayerRank>,
    pub room_id: String,
    pub is_locked: bool
}

/* 
    fire/emit/publish UserNotif events in ws server or using redis
    like using emit!(UserNotif{}) macro which emit and fire an
    event through the redis or ws streaming channel to clients 
    sub or listen to UserNotif events in ws client or redis
    in ws server then sends message to ws client using an event 
    loop or listener.
    update UserNotif on every data changes through its related api calls
    then fire the updated data event through the ws server so the client
    can subs using ws to the fired event 
*/
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

impl Nft{

    fn generate_event_hash<'t>() -> &'t str{
        let event_hash = "";
        event_hash
    }
}
