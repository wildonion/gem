

use crate::*;





/* 

    diesel migration generate users_contracts ---> create users_contracts migration sql files
    diesel migration run                      ---> apply sql files to db 
    diesel migration redo                     ---> drop tables 

*/

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserMintRequest{
    pub from_cid: String,
    pub recipient: String,
    pub contract_address: String,
    pub amount: i64,
    /* 
        this must be generated inside the client by signing the whole 
        data body of this struct using the client private key 
    */
    pub tx_signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserNftBurnRequest{
    pub from_cid: String,
    pub token_id: String,
    pub contract_address: String,
    pub amount: i64,
    /* 
        this must be generated inside the client by signing the whole 
        data body of this struct using the client private key 
    */
    pub tx_signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserAdvertiseRequest{
    pub from_cid: String,
    pub contract_address: String,
    pub amount: i64,
    /* 
        this must be generated inside the client by signing the whole 
        data body of this struct using the client private key 
    */
    pub tx_signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserContractRequest{
    pub from_cid: String,
    pub amount: i64,
    /* 
        this must be generated inside the client by signing the whole 
        data body of this struct using the client private key 
    */
    pub tx_signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, PartialEq)]
pub struct NewUserAddNftToContractRequest{
    pub from_cid: String,
    pub token_id: String,
    pub contract_address: String,
    pub amount: i64,
    /* 
        this must be generated inside the client by signing the whole 
        data body of this struct using the client private key 
    */
    pub tx_signature: String,
}