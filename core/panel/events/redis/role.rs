


use mongodb::bson::oid::ObjectId;
use crate::*;


#[derive(Clone, Debug, Serialize, Default, Deserialize)]
pub struct Reveal{
    pub players: Vec<PlayerRoleInfo>,
    pub event_id: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct PlayerRoleInfo{
  pub _id: ObjectId, // ObjectId is the bson type of _id inside the mongodb
  pub username: String,
  pub status: u8,
  pub role_name: Option<String>,
  pub role_id: Option<ObjectId>, // this field can be None at initialization which is the moment that a participant reserve an event
  pub side_id: Option<ObjectId>, // this field can be None at initialization which is the moment that a participant reserve an event
}