




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
    pub owners: Vec<OwnerData>, //// pda addresses (nft burn tx hash + nft owner)
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
    pub pdas: Vec<String>, //// pda address (nft burn tx hash + nft owner)
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
    pub pdas: Vec<String>, //// number of unique burned nfts for this owner (nft owner + nft burn tx hash)
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

    pub async fn add_pdas(&self, pdas: Vec<String>, owner_index: usize) -> Option<Vec<String>>{
        let mut owner = self.owners[owner_index].clone();
        let already_pda = owner.pdas.iter().all(|pda| pdas.contains(pda));
        if already_pda{
            None //// we've found a pda inside the owner.pdas vector thus we must notify the user that please pass a unique vector
        } else{
            let pdas_slice = pdas.as_slice(); //// converting the passed in pdas into the slice
            owner.pdas.extend_from_slice(&pdas_slice); //// extending the owner.pdas vector from the passed in pdas 
            Some(owner.pdas)
        }
    }
}