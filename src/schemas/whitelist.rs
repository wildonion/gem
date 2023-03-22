



use solana_sdk::{pubkey::Pubkey, program_pack::Pack};
use solana_client::{rpc_request, rpc_response, rpc_client::{self, RpcClient}}; //// self refers to the structure or module (rpc_client in our case) itself
use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};



///// the following will be used to load all the 
//// nft mint addresses inside the nfts.json into
//// this struct. 
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
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
    pub mint_addrs: Vec<String>, //// nft mint addresses that this owner owns
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
    pub mint_addrs: Vec<String>, //// number of unique burned nfts for this owner (nft owner + nft burn tx hash)
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

    pub async fn add_mint_addrs(&self, mint_addrs: Vec<String>, owner_index: usize) -> Option<Vec<String>>{
        let mut owner = self.owners[owner_index].clone();
        let already_burned = owner.mint_addrs.iter().any(|mint_addr| mint_addrs.contains(mint_addr));
        if already_burned{
            None //// we've found a mint_addr inside the owner.mint_addrs vector thus we must notify the user that please pass a unique vector since one of the nft address is already bunred
        } else{
            let mint_addrs_slice = mint_addrs.as_slice(); //// converting the passed in mint_addrs into the a string slice
            owner.mint_addrs.extend_from_slice(&mint_addrs_slice); //// extending the owner.mint_addrs vector from the passed in mint_addrs 
            Some(owner.mint_addrs)
        }
    }
}


#[derive(Deserialize)]
pub struct RpcTokenAccount{
    pub address: String,
}

pub async fn verify_owner(owner: String, mint_addrs: &[String], rpc_client: &RpcClient) -> bool{
    
    //// in rpc we can call the method name of 
    //// the actor object directly with a passed 
    //// in prams using rpc_request from another device 
    //// to get the utf8 response and map it 
    //// into another structure also in json rpc we 
    //// have to use the json codec and in rpc capnp 
    //// we have to use capnp codec
    let method_name = "getTokenLargestAccounts";
    let request = rpc_request::RpcRequest::Custom { method: method_name };
    let params = serde_json::json!(mint_addrs); //// since the solana rpc protocol is a json based rpc thus we have to pass the encoded data as a json 

    //// sending the rpc request to the solana rpc endpoint which is 
    //// inside the rpc_client, also we're deserializing the response
    //// came back from the json rpc server into a vector of RpcTokenAccount.
    //
    //// there are some predefined methdods for every 
    //// rpc actor object in which we can call them
    //// directly from other device through an rpc
    //// request and if the method has params we have 
    //// to pass the encoded param as either json or capnp.
    let res: Result<rpc_response::Response<Vec<RpcTokenAccount>>, Box<dyn std::error::Error>> = 
            rpc_client.send(request, params).map_err(|e| e.into()); //// map the error if there was any 
    let addr = res
            .unwrap()
            .value
            .first()
            .take() //// take() takes the some part and leaves a None if there is no some
            .unwrap()
            .address
            .parse()
            .unwrap();

    let mut account = rpc_client.get_account(&addr).unwrap();
    let token = spl_token::state::Account::unpack(&mut account.data).unwrap(); //// to borrow the data of the account mutably we must define the account as mutable
    let fetched_owner = token.owner;

    if owner == fetched_owner.to_string(){
        true 
    } else{
        false
    }
    
}