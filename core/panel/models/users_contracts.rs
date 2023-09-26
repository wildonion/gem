

use crate::*;





/* 

    diesel migration generate users_contracts ---> create users_contracts migration sql files
    diesel migration run                      ---> apply sql files to db 
    diesel migration redo                     ---> drop tables 

*/


pub struct UserContract{

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

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortMintResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
    pub metadata_uri: String,
    pub mint_to_address: String
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUploadMetadataRequest{
    pub name: String,
    pub description: String,
    pub file_url: String,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortGetNftResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub token_id: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortTransferNftResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortBurnResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUploadMetadataResponse{
    pub response: String,
    pub metadata_uri: String,
    pub name: String,
    pub description: String,
    pub file_url: String,
    pub external_url: Option<String>,
    pub animation_url: Option<String>,
    pub custom_fields: Option<HashMap<String, String>>,
    pub attributes: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUploadFileToIpfsResponse{
    pub res: NftPortUploadFileToIpfsData
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUploadFileToIpfsData{
    pub response: String,
    pub ipfs_url: String,
    pub file_name: String,
    pub content_type: String,
    pub file_size: i64,
    pub file_size_mb: f64
}


impl UserContract{

    pub async fn get_owner_by_contract_address(owner_contract_address: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserContract, PanelHttpResponse>{

            todo!()
            
    }
}