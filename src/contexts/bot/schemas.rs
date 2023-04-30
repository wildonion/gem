






use crate::*;


#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CatchUpData{
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub user_id: u64,
    pub channel_id: u64,
    pub guild_id: u64,
    pub catchup_request_at: String,
    pub catchup_from: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CatchUpDoc{
    pub user_id: u64,
    pub channel_id: u64,
    pub guild_id: u64,
    pub catchup_request_at: String,
    pub catchup_from: String,
    pub gpt_response: String,
    pub fetched_messages: String
}