


use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};
use uuid::Uuid;




// fire/emit/publish UserNotif events in ws/rpc/zmq server 
// sub or listen to UserNotif events in ws/rpc/zmq client using an event loop or listener


// update UserNotif on every data changes through its related api calls
// then fire the updated data event through the ws server so the client
// can subs to the fired event 
#[derive(Serialize, Deserialize, Clone)]
pub struct NotifData{
    pub is_active: bool,
    pub notifs: Vec<Notif>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Notif{
    pub id: Uuid, //// borsh is not implemented for Uuid
    pub seen: bool,
    pub data_id: String,
    pub data_owner: String,
    pub fired_at: Option<i64>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserNotif{
    pub item_sold: NotifData,
    pub bid_activity: NotifData, // When someone bids on one of your items
    pub price_change: NotifData, // When an item you made an offer on changes in price
    pub auction_expiration: NotifData, // When a timed auction you created ends
    pub outbid: NotifData, // When an offer you placed is exceeded by another user
    pub owned_item_updates: NotifData, // When a significant update occurs for one of the items you have purchased on dortzio
    pub successfull_purchase: NotifData, // Occasional updates from the dortzio team
    pub min_bid_tresh: NotifData, // Receive notifications only when you receive offers with a value greater than or equal to this amount of ETH.
    pub updated_at: Option<i64>,
}