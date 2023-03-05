




use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};








/*
  --------------------------------------------------------------------------------
| this struct will be used to serialize whitelist info into bson to insert into db
| --------------------------------------------------------------------------------
|
|
*/
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
pub struct AddWhitelistInfo{
    pub name: String,
    pub owners: Vec<OwnerData>, //// pda addresses (nft mint + nft owner)
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
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
pub struct InsertWhitelistRequest{
    pub owner: String, //// nft owner
    pub pda: String, //// pda address (nft mint + nft owner)
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
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub name: String,
    pub owner_list: OwnerData,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Default)]
pub struct OwnerData{
    pub pdas: Vec<String>, //// number of unique burned nfts for this owner (nft owner + nft mint)
    pub owner: String, //// nft owner
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
    pub _id: Option<ObjectId>, //// ObjectId is the bson type of _id inside the mongodb
    pub name: String,
    pub owners: Vec<OwnerData>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl WhitelistInfo{

    pub async fn add_pda(&self, pda: String, owner_index: usize) -> Option<Vec<String>>{
        let mut owner = self.owners[owner_index].clone();
        let found_pda = owner.pdas.clone().into_iter().position(|p| p == pda.clone());
        if found_pda == None{
            owner.pdas.push(pda.clone());
            Some(owner.pdas)                   
        } else{
            None
        }
    }
}