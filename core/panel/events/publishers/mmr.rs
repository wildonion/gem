



/* ------------------------------ */
/*   match making rating/ranking  */
/* ------------------------------ */
/*
    https://github.com/wildonion/zoomate + test_stream() api
    https://github.com/wildonion/gvm

    scoring, ranking and suggestion algos for nfts, galleries, users, collections
    based on stars field of the user, how much he has interacted with other users' stuffs

    use gvm to feed vm the top users data so they can get ranked and suggested to 
    each other based on their stars field
*/



use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};
use uuid::Uuid;
use crate::*;
use self::models::users::UserData;


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserRank{
    pub screen_cid: String, /* crypto id usally public address */
    pub events: Vec<String>, 
    pub rank: u16, /* this is the stars field in users table */
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Queue{
    pub players: Vec<UserData>,
    pub event_id: i32, // the event that contains participants
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CurrentMatch{
    pub event_id: String,
    pub users: Vec<UserRank>,
}