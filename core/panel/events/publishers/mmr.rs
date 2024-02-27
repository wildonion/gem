



/* ------------------------------ */
/*   match making rating/ranking  */
/* ------------------------------ */
/*
    https://github.com/wildonion/zoomate + test_stream() api
    https://github.com/wildonion/gvm

    complete proc macros in lib.rs as a plugin
    scoring, ranking and suggestion algos for nfts, galleries, users, collections
    based on stars field of the user, how much he has interacted with other users' stuffs

*/



use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};
use uuid::Uuid;



#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserRank{
    pub screen_cid: String, /* crypto id usally public address */
    pub events: Vec<String>, 
    pub rank: u16, /* this is the stars field in users table */
}


impl UserRank{

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
    pub users: Vec<UserRank>,
}