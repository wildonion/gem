


use crate::*;



#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Nft{
    pub id: i32,
    pub pastel_id: String, // base58-encoded public key
    pub tickets: Vec<String>, // list of all pastel tickets: offer, transfer, sell, buy, ...
    pub owner: String, // user screen_cid
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Nft{

    pub async fn insert(){

    }

}