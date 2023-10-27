


use crate::*;



#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserNft{
    pub id: i32,
    pub contract_address: String, // contract address contains the collection info
    pub current_owner_screen_cid: String, // the screen_cid of current owner of this nft
    pub metadata: String, // json stringified like statistical data like nft statistics
    pub img_url: String,
    pub onchain_id: Option<String>, // fulfilled after minting
    pub name: String,
    pub is_minted: bool, // if it's false means that is not stored on contract yet
    pub description: String,
    pub current_price: Option<i64>,
    pub is_listed: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserNftData{
    pub id: i32,
    pub contract_address: String,
    pub current_owner_screen_cid: String, // the screen_cid of current owner of this nft
    pub metadata: String, // json stringified like statistical data like nft statistics
    pub img_url: String,
    pub onchain_id: Option<String>, // fulfilled after minting
    pub name: String,
    pub is_minted: bool, // if it's false means that is not stored on contract yet
    pub description: String,
    pub current_price: Option<i64>,
    pub is_listed: bool,
    pub created_at: String,
    pub updated_at: String,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserNft{

    pub async fn get_info_of(asset_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

}

impl UserNft{

    pub async fn insert(asset_info: UserNftData, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
        
        // upload on pastel using sense and cascade apis: paste::sense::detect(), paste::cascade::upload()
        // spend token for gas fee and update listings
        // by default is_listed will be set to true since an nft goes to private collection by default 
        // which must be listed to be sold to friends have been invited by the gallery owner
        // ...

        Ok(())

    }

    /* -------------------------------------------------------------------------- */
    /* this method can be called to update an nft status like minting and listing */
    /* -------------------------------------------------------------------------- */
    /* supported apis (spend token for gas fee like update listings):
        - mint_nft           ---- https://docs.nftport.xyz/reference/customizable-minting
        - transfer_nft       ---- https://docs.nftport.xyz/reference/transfer-minted-nft
        - update_nft         ---- https://docs.nftport.xyz/reference/update-minted-nft
        - sell_nft
        - buy_nft
    */
    pub async fn update(asset_info: UserNftData, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        // ...

        Ok(())

    }

}