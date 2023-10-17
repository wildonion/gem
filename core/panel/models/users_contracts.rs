

use crate::*;



/* 

    diesel migration generate users_contracts ---> create users_contracts migration sql files
    diesel migration run                      ---> apply sql files to db 
    diesel migration redo                     ---> drop tables 

*/


pub struct UserContract{

    pub user_id: i32,
    pub collection: i32,
    pub c_address: String, // contract address

}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserMintRequest{
    pub from_cid: String,
    pub recipient: String,
    pub contract_address: String,
    pub amount: i64,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserNftBurnRequest{
    pub from_cid: String,
    pub token_id: String,
    pub contract_address: String,
    pub amount: i64,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserAdvertiseRequest{
    pub from_cid: String,
    pub contract_address: String,
    pub amount: i64,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserContractRequest{
    pub from_cid: String,
    pub amount: i64,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserAddNftToContractRequest{
    pub from_cid: String,
    pub token_id: String,
    pub contract_address: String,
    pub amount: i64,
    pub tx_signature: String,
    pub hash_data: String,
}


impl UserContract{

    pub async fn get_owner_by_contract_address(owner_contract_address: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserContract, PanelHttpResponse>{

            todo!()
            
    }
}