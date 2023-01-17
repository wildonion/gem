




use crate::schemas::game::{InsertPlayerInfoRequest, ReservePlayerInfoResponseWithRoleName, AddGroupInfoToEvent, EventLastMoveInfo};
use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use uuid::Uuid;






// NOTE - each collection has some documents which can be deserailzed into a struct inside the rust
// NOTE - serializing from struct or json or bson into the utf8 bytes and deserializing from utf8 into json or struct or bson
// NOTE - to send some data back to the user we must serialize that data struct into the json and from there to utf8 to pass through the socket
// NOTE - to send fetched data from mongodb which is a bson object back to the user we must first deserialize the bson into the struct then serialize to json to serialize to utf8 to send back to the user which is a bson object back to the user we must first deserialize the bson into its related struct and then serialize it to json to send back to the user through the socket








#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Simd{
    pub input: u32,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Voter{
    pub nft_owner_wallet_address: String,
    pub is_upvote: bool,
    pub score: u32, // NOTE - this is the number of event NFTs that this owner owns
}



#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CastVoteRequest{
    pub _id: String, //// this is the id of the event took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub voter: Voter,
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Phase{
    pub day: Vec<InsertPlayerInfoRequest>, //// vector of all user infos at the end of the day that their status has changed
    pub mid_day: Vec<InsertPlayerInfoRequest>, //// vector of all user infos at the end of the mid day that their status has changed
    pub night: Vec<InsertPlayerInfoRequest>, //// vector of all user infos at the end of the night that their status has changed
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize even info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GetEventRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub _id: String, //// this is the id of the event took from the mongodb events collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize even info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GetPlayerEventsRequest{
    pub _id: String, //// this is the id of the event took from the mongodb events collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  ---------------------------------------------------------------------------------------
| this struct will be used to deserialize payment info json from client into this struct
| ---------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)] //// the Default trait must be implemented for all types of each field 
pub struct InsertPhaseRequest{
    pub event_id: String, //// this is the id of the event took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub phase: Phase, //// this is the new phase that the god is trying to insert into the current phases; means we passed a complete day in our game
}


/*
  -----------------------------------------------------------------------------------------------
| this struct will be used to put an event info in it and serialize as json to send back to user
| -----------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct InsertPhaseResponse{
    pub _id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub deck_id: String,
    pub phases: Option<Vec<Phase>>,
    pub max_players: Option<u8>,
    pub players: Option<Vec<ReservePlayerInfoResponseWithRoleName>>,
    pub is_expired: Option<bool>,
    pub is_locked: Option<bool>,
    pub expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  -----------------------------------------------------------------------------------------------
| this struct will be used to put an event info in it and serialize as json to send back to user
| -----------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ReserveEventResponse{
    pub _id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub deck_id: String,
    pub phases: Option<Vec<Phase>>,
    pub max_players: Option<u8>,
    pub players: Option<Vec<ReservePlayerInfoResponseWithRoleName>>,
    pub is_expired: Option<bool>,
    pub is_locked: Option<bool>,
    pub expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}



/*
  ---------------------------------------------------------------------------------------
| this struct will be used to deserialize payment info json from client into this struct
| ---------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddPaymentRequest{
    pub username: String,
    pub user_id: String, //// this is the id of the user took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub event_id: String, //// this is the id of the event took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub authority: String,
    pub successful_payment_id: Uuid, //// this is the successful payment id 
    pub verification_code: Option<String>, //// it might be None by canceling the payment process
    pub ref_id: Option<String>, //// it might be None by canceling the payment process
    pub card_pan: Option<String>, //// it might be None by canceling the payment process
    pub card_hash: Option<String>, //// it might be None by canceling the payment process
    pub fee_type: Option<String>, //// it might be None by canceling the payment process
    pub fee: Option<String>, //// it might be None by canceling the payment process
    pub requested_at: i64,
    pub paid_at: Option<i64>, //// it might be None by canceling the payment process
}


/*
  --------------------------------------------------------------------------------------------
| this struct will be used to deserialize mock payment info json from client into this struct
| --------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct MockReservationRequest{
    pub event_id: String, //// this is the id of the event took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub requested_at: i64, //// this the tiem of the request coming from the client in unix timestamp foramt
}


/*
  --------------------------------------------------------------------------------------------
| this struct will be used to deserialize payment info bson from the mongodb into this struct
| --------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct PaymentInfo{
    pub _id: Option<ObjectId>,
    pub username: String,
    pub user_id: String,
    pub event_id: String,
    pub authority: String,
    pub successful_payment_id: Uuid,
    pub verification_code: Option<String>,
    pub ref_id: Option<String>,
    pub card_pan: Option<String>,
    pub card_hash: Option<String>,
    pub fee_type: Option<String>,
    pub fee: Option<String>,
    pub requested_at: i64,
    pub paid_at: Option<i64>,
}


/*
  -----------------------------------------------------------------------------------------
| this struct will be used to deserialize event info bson from the mongodb into this struct
| -----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct EventInfo{
    pub _id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub deck_id: String,
    pub entry_price: String,
    pub group_info: Option<AddGroupInfoToEvent>,
    pub image_path: Option<String>,
    pub creator_wallet_address: Option<String>,
    pub upvotes: Option<u16>,
    pub downvotes: Option<u16>,
    pub voters: Option<Vec<Voter>>,
    pub phases: Option<Vec<Phase>>,
    pub max_players: Option<u8>,
    pub players: Option<Vec<ReservePlayerInfoResponseWithRoleName>>,
    pub is_expired: Option<bool>,
    pub is_locked: Option<bool>,
    pub started_at: Option<i64>,
    pub expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  ------------------------------------------------------------------------------------------------
| this struct will be used to deserialize player event info bson from the mongodb into this struct
| ------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct PlayerEventInfo{
    pub _id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub deck_id: String,
    pub entry_price: String,
    pub group_info: Option<AddGroupInfoToEvent>,
    pub image_path: Option<String>,
    pub creator_wallet_address: Option<String>,
    pub upvotes: Option<u16>,
    pub downvotes: Option<u16>,
    pub voters: Option<Vec<Voter>>,
    pub phases: Option<Vec<Phase>>,
    pub max_players: Option<u8>,
    pub is_expired: Option<bool>,
    pub is_locked: Option<bool>,
    pub started_at: Option<i64>,
    pub expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  -------------------------------------------------------------------------------------------------
| this struct will be used to deserialize explore event info bson from the mongodb into this struct
| -------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ExploreEventInfo{
    pub _id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub entry_price: String,
    pub group_info: Option<AddGroupInfoToEvent>,
    pub image_path: Option<String>,
    pub creator_wallet_address: Option<String>,
    pub upvotes: Option<u16>,
    pub downvotes: Option<u16>,
    pub voters: Option<Vec<Voter>>,
    pub max_players: Option<u8>,
    pub is_expired: Option<bool>,
    pub is_locked: Option<bool>,
    pub started_at: Option<i64>,
    pub expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  -----------------------------------------------------------------------------------------
| this struct will be used to deserialize event info bson from the mongodb into this struct
| -----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct RevealEventInfo{
    pub _id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub deck_id: String,
    pub entry_price: String,
    pub group_info: Option<AddGroupInfoToEvent>,
    pub image_path: Option<String>,
    pub creator_wallet_address: Option<String>,
    pub upvotes: Option<u16>,
    pub downvotes: Option<u16>,
    pub voters: Option<Vec<Voter>>,
    pub phases: Option<Vec<Phase>>,
    pub max_players: Option<u8>,
    pub players: Option<Vec<ReservePlayerInfoResponseWithRoleName>>,
    pub is_expired: Option<bool>,
    pub is_locked: Option<bool>,
    pub started_at: Option<i64>,
    pub expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  -------------------------------------------------------------------------------------
| this struct will be used to deserialize event info json from client into this struct
| -------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddEventRequest{
    pub title: String,
    pub content: String,
    pub deck_id: String, //// this is the id of the selected deck for this event took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub entry_price: String,
    pub group_info: Option<AddGroupInfoToEvent>,
    pub image_path: Option<String>,
    pub creator_wallet_address: Option<String>, //// it might be None at initializing stage inside the add api
    pub upvotes: Option<u16>, // NOTE - we set this field to Option cause we don't want to pass the upvotes inside the request body, we'll fill it inside the server
    pub downvotes: Option<u16>, // NOTE - we set this field to Option cause we don't want to pass the downvotes inside the request body, we'll fill it inside the server
    pub voters: Option<Vec<Voter>>, // NOTE - we set this field to Option cause we don't want to pass the voters inside the request body, we'll update it later on
    pub phases: Option<Vec<Phase>>, // NOTE - we set this field to Option cause we don't want to pass the phases inside the request body, we'll update it later on
    pub max_players: Option<u8>, // NOTE - number of maximum players for this event
    pub players: Option<Vec<ReservePlayerInfoResponseWithRoleName>>, // NOTE - vector of all players which has participated for this event
    pub is_expired: Option<bool>,
    pub is_locked: Option<bool>, // NOTE - we set this field to Option cause we don't want to pass the is_expired inside the request body, we'll update it once a event reached the deadline
    pub started_at: Option<i64>,
    pub expire_at: Option<i64>, // NOTE - we set this field to Option cause we don't want to pass the expire_at inside the request body, we'll update it while we want to create a new event object
    pub created_at: Option<i64>, // NOTE - we set this field to Option cause we don't want to pass the created time inside the request body, we'll fill it inside the server
    pub updated_at: Option<i64>, // NOTE - we set this field to Option cause we don't want to pass the updated time inside the request body, we'll fill it inside the server
}


/*
  ------------------------------------------------------------------------------------------------------
| this struct will be used to put all available events in it and serialize as json to send back to user
| ------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AvailableEvents{
    pub events: Vec<EventInfo>,
}


/*
  -----------------------------------------------------------------------------------------------------------
| this struct will be used to put all player expired events in it and serialize as json to send back to user
| -----------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct PlayerExpiredEvents{
    pub events: Vec<EventInfo>,
}


/*
  -------------------------------------------------------------------------------------
| this struct will be used to deserialize expire info json from client into this struct
| -------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ExpireEventRequest{
    pub _id: String, //// this is the id of the event took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  -------------------------------------------------------------------------------------
| this struct will be used to deserialize lock info json from client into this struct
| -------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct LockEventRequest{
    pub _id: String, //// this is the id of the event took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  -------------------------------------------------------------------------------------
| this struct will be used to deserialize cancel info json from client into this struct
| -------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct PlayerCancelEventRequest{
    pub event_id: String,
    pub user_id: String,
}


/*
  -------------------------------------------------------------------------------------
| this struct will be used to deserialize delete info json from client into this struct
| -------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DeleteEventRequest{
    pub _id: String, //// this is the id of the event took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}



impl EventInfo{ //// methods without &self takes the ownership of their instances and will move their lifetime

    pub async fn add_voter(self, voter: Voter) -> Vec<Voter>{ //// we don't take a reference to self cause we can't dereference a shared reference (&T) and if we do that then cannot borrow `*voters` as mutable, cause it is behind a `&` reference and `voters` is a `&` reference, so the data it refers to cannot be borrowed as mutable cause we have to define the first argument as &mut self
        let mut voters = self.voters.unwrap();
        let index = voters.iter().position(|v| v.nft_owner_wallet_address == voter.nft_owner_wallet_address); //// this owner has alreay voted to this event
        if index == None{
            voters.push(voter);
        }
        voters
    }
}


impl EventInfo{ //// we have to define the following method for the EventInfo struct cause we want to call this method on a document of fetched from the events collection which is an instance of the EventInfo struct 

    pub async fn add_phase(self, new_phase: Phase) -> Vec<Phase>{ //// new phase is of type Phase struct which contains list of all player infos in which their status has changed during the game for day, mid-day and night - //// we don't take a reference to self cause we can't dereference a shared reference (&T) and if we do that then cannot borrow `*voters` as mutable, cause it is behind a `&` reference and `voters` is a `&` reference, so the data it refers to cannot be borrowed as mutable cause we have to define the first argument as &mut self
        let mut current_phases = self.phases.unwrap();
        current_phases.push(new_phase);
        current_phases
    } 

}


impl EventInfo{

    pub async fn add_player(self, player_info: ReservePlayerInfoResponseWithRoleName) -> Vec<ReservePlayerInfoResponseWithRoleName>{ //// we don't take a reference to self cause we can't dereference a shared reference (&T) and if we do that then cannot borrow `*voters` as mutable, cause it is behind a `&` reference and `voters` is a `&` reference, so the data it refers to cannot be borrowed as mutable cause we have to define the first argument as &mut self
      let mut current_players = self.players.unwrap();
      let index = current_players.iter().position(|p| p._id == player_info._id); //// this user has already participated in this event
      if index == None && current_players.len() < self.max_players.unwrap() as usize{
        current_players.push(player_info)
      }
      current_players
    }

}