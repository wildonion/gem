


use crate::*;



#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Gallery{
    pub id: i32,
    pub owner_cid: String, // user screen_cid
    pub pastel_id: String, // base58-encoded public key
    pub nfts: Vec<String>, // sql field: TEXT[] DEFAULT ARRAY[]::TEXT[] - pastel artwork ids
    pub is_private: bool,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Gallery{

    pub async fn insert(){

    }

}