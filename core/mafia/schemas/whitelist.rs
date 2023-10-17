



use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file



//// the following will be used to load all the 
// nft mint addresses inside the addrs.json into
// this struct. 
#[derive(Serialize, Deserialize, Clone)]
pub struct Nft{
    pub mint_addrs: Vec<String>,
}


/*
  --------------------------------------------------------------------------------
| this struct will be used to serialize whitelist info into bson to insert into db
| --------------------------------------------------------------------------------
|
|
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct AddWhitelistInfo{
    pub name: String,
    pub owners: Vec<OwnerData>, // pda addresses (nft burn tx hash + nft owner)
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

/*
  ----------------------------------------------------------------------------------------
| this struct will be used to deserialize whitelist info json from client into this struct
| ----------------------------------------------------------------------------------------
|
|
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct InsertWhitelistRequest{
    pub owner: String, // nft owner
    pub mint_addrs: Vec<String>, // nft mint addresses that this owner owns
    pub name: String,
}

/*
  -----------------------------------------------------------------------------------------------
| this struct will be used to put whitelist info in it and serialize as json to send back to user
| -----------------------------------------------------------------------------------------------
|
|
*/
#[derive(Serialize, Deserialize, Clone)]
pub struct InsertWhitelistResponse{
    pub _id: Option<ObjectId>, // ObjectId is the bson type of _id inside the mongodb
    pub name: String,
    pub owner_list: OwnerData,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct OwnerData{
    pub mint_addrs: Vec<String>, // number of unique burned nfts for this owner (nft owner + nft burn tx hash)
    pub owner: String, // nft owner
    pub requested_at: Option<i64>,
}

/*
  ---------------------------------------------------------------------------------------------
| this struct will be used to deserialize whitelist info bson from the mongodb into this struct
| ---------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct WhitelistInfo{
    pub _id: Option<ObjectId>, // ObjectId is the bson type of _id inside the mongodb
    pub name: String,
    pub owners: Vec<OwnerData>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl WhitelistInfo{

    pub async fn add_mint_addrs(&self, mint_addrs: Vec<String>, owner_index: usize) -> Option<Vec<String>>{
        let mut owner = self.owners[owner_index].clone();
        let already_burned = owner.mint_addrs.iter().any(|mint_addr| mint_addrs.contains(mint_addr));
        if already_burned{
            None // we've found a mint_addr inside the owner.mint_addrs vector thus we must notify the user that please pass a unique vector since one of the nft address is already bunred
        } else{
            let mint_addrs_slice = mint_addrs.as_slice(); // converting the passed in mint_addrs into the a string slice
            owner.mint_addrs.extend_from_slice(&mint_addrs_slice); // extending the owner.mint_addrs vector from the passed in mint_addrs 
            Some(owner.mint_addrs)
        }
    }
}