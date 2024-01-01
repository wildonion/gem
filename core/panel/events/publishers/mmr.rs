



/* ------------------------------ */
/*   match making rating/ranking  */
/* ------------------------------ */
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
    pub events: Vec<String>, 
    pub rank: u16, /* this is the stars field in users table */
}


impl PlayerRank{

    fn calculate(&self){

        // get all events info the `phases` field for each event 
        // then monitor the activities of the player in all data
        // we've got from the event 
    }

    fn update_rank(&self){

        // update stars field in users table
    }

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrentMatch{
    pub event_id: String,
    pub players: Vec<PlayerRank>,
    pub room_id: String,
    pub is_locked: bool
}
