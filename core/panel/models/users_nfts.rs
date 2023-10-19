


use crate::*;


#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Nft{
    pub id: i32,
    pub name: Vec<i32>,
    pub description: bool,
    pub image: String, // ipfs url of image
    pub extra: String, // ipfs url of extra data as json stringified
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserNftRequest{
    pub from_cid: String,
    pub amount: i64,
    pub tx_signature: String,
    pub hash_data: String,
}