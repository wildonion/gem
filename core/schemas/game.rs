




use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file







// NOTE - each collection has some documents which can be deserailzed into a struct inside the rust
// NOTE - serializing from struct or json or bson into the utf8 bytes and deserializing from utf8 into json or struct or bson
// NOTE - to send some data back to the user we must serialize that data struct into the json and from there to utf8 to pass through the socket
// NOTE - to send fetched data from mongodb which is a bson object back to the user we must first deserialize the bson into the struct then serialize to json to serialize to utf8 to send back to the user which is a bson object back to the user we must first deserialize the bson into its related struct and then serialize it to json to send back to the user through the socket







/*
  --------------------------------------------------------------------------------------------
| this struct will be used to deserialize get god group info json from client into this struct
| --------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GetPlayerInfoRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub user_id: String, //// this is the id of the player took from the mongodb users collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub event_id: String,
}


/*
  ------------------------------------------------------------------------------------------
| this struct will be used to deserialize get player info json from client into this struct
| ------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GetGodGroupsRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub _id: String, //// this is the id of the god took from the mongodb users collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize role info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddRoleRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub name: String, //// role name
    pub rate: u8, //// role rate
    pub desc: String, //// role description
    pub abilities: u8, //// number of total abilities for a role, the default is 0
    pub side_id: String, //// this field can be None at initialization which is the moment that a participant reserve an event
    pub is_disabled: Option<bool>, //// whether this role has disabled or not
    pub created_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
    pub updated_at: Option<i64>, //// we set this field to Option cause we don't want to pass the updated time inside the request body thus it should be None initially, we'll fill it inside the server
}


/*
  ---------------------------------------------------------------------------------------------
| this struct will be used to deserialize last move card info json from client into this struct
| ---------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddLastMoveRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub name: String, //// role name
    pub rate: u8, //// role rate
    pub desc: String, //// role description
    pub is_disabled: Option<bool>,
    pub created_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
    pub updated_at: Option<i64>, //// we set this field to Option cause we don't want to pass the updated time inside the request body thus it should be None initially, we'll fill it inside the server
}


/*
  ---------------------------------------------------------------------------------------------
| this struct will be used to deserialize last move info bson from the mongodb into this struct
| ---------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct LastMoveInfo{
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub name: String,
    pub rate: u8,
    pub desc: String,
    pub is_disabled: Option<bool>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  ---------------------------------------------------------------------------------------------------
| this struct will be used to deserialize event last move info bson from the mongodb into this struct
| ---------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct EventLastMoveInfo{
    pub _id: String,
    pub name: String,
    pub rate: u8,
    pub desc: String,
    pub is_disabled: Option<bool>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  -----------------------------------------------------------------------------------------
| this struct will be used to deserialize role info bson from the mongodb into this struct
| -----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct RoleInfo{
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub name: String,
    pub rate: u8,
    pub desc: String,
    pub abilities: u8,
    pub side_id: Option<ObjectId>,
    pub is_disabled: Option<bool>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  -----------------------------------------------------------------------------------------
| this struct will be used to deserialize role info bson from the mongodb into this struct
| -----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct DeckRoleInfo{
    pub _id: String, //// the role id itself
    pub name: String,
    pub rate: u8,
    pub desc: String,
    pub abilities: u8,
    pub side_id: String, //// side id of the role
    pub is_disabled: Option<bool>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize role info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddDeckRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub deck_name: String,
    pub roles: Vec<DeckRoleInfo>,
    pub last_move_cards: Vec<EventLastMoveInfo>,
    pub is_disabled: Option<bool>, //// whether this deck has disabled or not
    pub created_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
    pub updated_at: Option<i64>, //// we set this field to Option cause we don't want to pass the updated time inside the request body thus it should be None initially, we'll fill it inside the server
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize role info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UpsertDeckRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub _id: Option<String>, //// this is the stringify object id of the deck; it can be optional to insert a new deck doc
    pub deck_name: String,
    pub roles: Vec<DeckRoleInfo>,
    pub last_move_cards: Vec<EventLastMoveInfo>,
    pub is_disabled: Option<bool>, //// whether this deck has disabled or not
    pub created_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
    pub updated_at: Option<i64>, //// we set this field to Option cause we don't want to pass the updated time inside the request body thus it should be None initially, we'll fill it inside the server
}


/*
  -----------------------------------------------------------------------------------------
| this struct will be used to deserialize deck info bson from the mongodb into this struct
| -----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DeckInfo{
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub deck_name: String,
    pub roles: Vec<DeckRoleInfo>,
    pub last_move_cards: Vec<EventLastMoveInfo>,
    pub is_disabled: Option<bool>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize group info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddGroupRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub name: String,
    pub owner: String,
    pub image_path: Option<String>,
    pub god_id: Option<String>, //// this is the owner id of this group which is the god id of the created group
    pub created_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
    pub updated_at: Option<i64>, //// we set this field to Option cause we don't want to pass the updated time inside the request body thus it should be None initially, we'll fill it inside the server
}


/*
  -----------------------------------------------------------------------------------------
| this struct will be used to deserialize group info bson from the mongodb into this struct
| -----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GroupInfo{
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub name: String,
    pub owner: String, //// this is the id of the user took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub image_path: Option<String>, //// since there might be no image at all set for the group
    pub god_id: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize group info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddGroupInfoToEvent{
    pub _id: String, //// ObjectId is the bson type of _id inside the mongodb
    pub name: String,
    pub owner: String, //// this is the id of the user took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub image_path: Option<String>,
    pub god_id: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  -------------------------------------------------------------------------------------
| this struct will be used to deserialize group info json from client into this struct
| -------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UpdateGroupRequest{
    pub _id: String, //// this is the id of the group took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub name: String,
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize group info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GetGroupRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub _id: String, //// this is the id of the event took from the mongodb events collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize deck info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GetDeckRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub _id: String, //// this is the id of the event took from the mongodb events collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  ------------------------------------------------------------------------------------------------------
| this struct will be used to put all available groups in it and serialize as json to send back to user
| ------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AvailableGroups{
    pub groups: Vec<GroupInfo>, //// fetch all groups
}


/*
  ------------------------------------------------------------------------------------------------------
| this struct will be used to put all available decks in it and serialize as json to send back to user
| ------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AvailableDecks{
    pub decks: Vec<DeckInfo>, //// fetch all none disabled decks
}


/*
  ------------------------------------------------------------------------------------------------------
| this struct will be used to put all available roles in it and serialize as json to send back to user
| ------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AvailableRoles{
    pub roles: Vec<RoleInfo>, //// fetch all none disabled roles
}


/*
  ------------------------------------------------------------------------------------------------------
| this struct will be used to put all available sides in it and serialize as json to send back to user
| ------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AvailableSides{
    pub sides: Vec<SideInfo>, //// fetch all none disabled sides
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize side info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddSideRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub name: String,
    pub is_disabled: Option<bool>, //// whether this side has disabled or not
    pub created_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
    pub updated_at: Option<i64>, //// we set this field to Option cause we don't want to pass the updated time inside the request body thus it should be None initially, we'll fill it inside the server
}


/*
  -----------------------------------------------------------------------------------------
| this struct will be used to deserialize side info bson from the mongodb into this struct
| -----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SideInfo{
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub is_disabled: Option<bool>,
    pub name: String,
}


/*
  --------------------------------------------------------------------------------------------
| this struct will be used to deserialize role disable info json from client into this struct
| --------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DisableRoleRequest{
    pub _id: String, //// this is the id of the role took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  ------------------------------------------------------------------------------------------------
| this struct will be used to deserialize last move disable info json from client into this struct
| ------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DisableLastMoveRequest{
    pub _id: String, //// this is the id of the role took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  --------------------------------------------------------------------------------------------
| this struct will be used to deserialize deck disable info json from client into this struct
| --------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DisableDeckRequest{
    pub _id: String, //// this is the id of the deck took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  --------------------------------------------------------------------------------------------
| this struct will be used to deserialize side disable info json from client into this struct
| --------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DisableSideRequest{
    pub _id: String, //// this is the id of the side took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  -----------------------------------------------------------------------------------------------------
| this struct will be used to deserialize player role abilities info json from client into this struct
| -----------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UpdatePlayerRoleAbilityRequest{
    pub user_id: String, 
    pub role_id: String,
    pub event_id: String,
    pub current_ability: Option<u8>,
}


/*
  ---------------------------------------------------------------------------------------------------
| this struct will be used to deserialize player role ability info json from client into this struct
| ---------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct InsertPlayerRoleAbilityRequest{
    pub user_id: String,
    pub role_id: String,
    pub event_id: String,
    pub current_ability: Option<u8>, //// on inserting a new record this field will be initialized empty and will be updated during the game by the God
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  ----------------------------------------------------------------------------------------------------------
| this struct will be used to deserialize player role abilities info bson from the mongodb into this struct
| ----------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct PlayerRoleAbilityInfo{
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub user_id: String, //// string type of ObjectId for user id 
    pub role_id: String, //// string type of ObjectId for role id
    pub event_id: String, //// string type of ObjectId for event id
    pub current_ability: Option<u8>, //// number of current abilities for this player
    pub created_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
    pub updated_at: Option<i64>, //// we set this field to Option cause we don't want to pass the updated time inside the request body thus it should be None initially, we'll fill it inside the server
}


/*
  --------------------------------------------------------------------------------------------
| this struct will be used to deserialize player scores info json from client into this struct
| --------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct InsertGodVoteOnPlayerInfoRequest{
    pub user_id: String,
    pub event_id: String,
    pub score: u8,
    pub issued_at: Option<i64>, 
}


/*
  -------------------------------------------------------------------------------------------------
| this struct will be used to deserialize player scores info bson from the mongodb into this struct
| -------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GodVotesOnPlayerInfo{
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub user_id: String, //// string type of ObjectId for user id 
    pub event_id: String, //// string type of ObjectId for role id
    pub score: u8,
    pub issued_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
}


/*
  --------------------------------------------------------------------------------------------------
| this struct will be used to deserialize get single god vote info json from client into this struct
| --------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GetGodVotePlayerInfoRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub user_id: String, //// this is the id of the player took from the mongodb users collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub event_id: String, //// this is the id of the player took from the mongodb users collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  --------------------------------------------------------------------------------------------
| this struct will be used to deserialize get god votes info json from client into this struct
| --------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GetGodVotesPlayerInfoRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub user_id: String, //// this is the id of the player took from the mongodb users collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  --------------------------------------------------------------------------------------------
| this struct will be used to deserialize player chain info json from client into this struct
| --------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct InsertPlayerChainToRequest{
    pub from_id: String,
    pub to_id: String,
    pub event_id: String,
    pub chained_at: Option<i64>, //// this must be filled inside the server
}


/*
  -------------------------------------------------------------------------------------------------
| this struct will be used to deserialize player chain info bson from the mongodb into this struct
| -------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct PlayerChainToInfo{
    pub _id: Option<ObjectId>,
    pub from_id: String, //// string type of ObjectId for from user id 
    pub to_id: String, //// string type of ObjectId for to user id
    pub event_id: String, 
    pub chained_at: Option<i64>, //// we set this field to Option cause we don't want to pass the chained time inside the request body thus it should be None initially, we'll fill it inside the server
}


/*
  -----------------------------------------------------------------------------------------------------------
| this struct will be used to put all available chain infos in it and serialize as json to send back to user
| -----------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AvailableChainInfos{
    pub chain_infos: Vec<PlayerChainToInfo>,
}


/*
  -----------------------------------------------------------------------------------------------------------
| this struct will be used to serialize player info after reservation into the json to send back to the user
| -----------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ReservePlayerInfoResponse{
  pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
  pub username: String,
  pub status: u8,
  pub role_id: Option<ObjectId>, //// this field can be None at initialization which is the moment that a participant reserve an event
  pub side_id: Option<ObjectId>, //// this field can be None at initialization which is the moment that a participant reserve an event
}


/*
  -----------------------------------------------------------------------------------------------------------
| this struct will be used to serialize player info after reservation into the json to send back to the user
| -----------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ReservePlayerInfoResponseWithRoleName{
  pub _id: ObjectId, //// ObjectId is the bson type of _id inside the mongodb
  pub username: String,
  pub status: u8,
  pub role_name: Option<String>,
  pub role_id: Option<ObjectId>, //// this field can be None at initialization which is the moment that a participant reserve an event
  pub side_id: Option<ObjectId>, //// this field can be None at initialization which is the moment that a participant reserve an event
}


/*
  -------------------------------------------------------------------------------------------
| this struct will be used to deserialize player info bson from the mongodb into this struct
| -------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct InsertPlayerInfoRequest{
  pub user_id: String, //// ObjectId is the bson type of _id inside the mongodb
  pub username: String,
  pub status: u8,
  pub role_name: String,
  pub role_id: String,
  pub side_id: String,
  pub chain_history: Vec<ChainInfo>,
  pub role_ability_history: Vec<RoleAbilityInfo>,
}


#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ChainInfo{
  pub to_id: String,
  pub chained_at: i64,
}


#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct RoleAbilityInfo{
  pub role_id: String,
  pub current_ability: Option<u8>,
  pub updated_at: Option<i64>,
}