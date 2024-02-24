



/* ------------------------------ */
/*   match making rating/ranking  */
/* ------------------------------ */
/*
    https://github.com/wildonion/zoomate + test_stream() api
    https://github.com/wildonion/gvm/blob/main/src/lib.rs

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


pub mod structures{

    // graph game (multithreaded graph, random, board, score, enemy, tokio, redis, actix, libp2p)
    struct Enemy{
        damage_rate: u8
    }

    struct Player<'s>{
        nickname: &'s str,
        score: u16,
    }

    struct Col{
        x: u8,
        y: u8
    }

    struct Row{
        x: u8,
        y: u8
    }

    struct Board<'b>{
        col: &'b [Col],
        row: &'b [Row]
    }

    struct Node<T>{
        pub value: T, 
        pub parent: std::sync::Arc<std::rc::Weak<Node<T>>>,
        pub children: std::sync::Arc<tokio::sync::Mutex<Vec<Node<T>>>>
    }

}

pub mod functions{
    
}