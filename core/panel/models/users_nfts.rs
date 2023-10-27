


use crate::*;



#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserNft{
    pub id: i32,
    pub contract_address: String,
    pub current_owner_screen_cid: String,
    pub metadata: String, // json stringified like statistical data like nft statistics
    pub img_url: String,
    pub onchain_id: Option<String>,
    pub name: String,
    pub is_minted: bool,
    pub description: String,
    pub current_price: Option<i64>,
    pub is_listed: bool,
    pub comments: Vec<NftComment>,
    pub likes: Vec<NftLike>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NftComment{
    pub nft_onchain_id: String,
    pub content: String,
    pub owner_screen_cid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NftLike{
    pub nft_onchain_id: String,
    pub upvoter_screen_cids: Vec<String>,
    pub downvoter_screen_cids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserNftData{
    pub id: i32,
    pub contract_address: String,
    pub current_owner_screen_cid: String,
    pub metadata: String,
    pub img_url: String,
    pub onchain_id: Option<String>,
    pub name: String,
    pub is_minted: bool,
    pub description: String,
    pub current_price: Option<i64>,
    pub is_listed: bool,
    pub comments: Vec<NftComment>,
    pub likes: Vec<NftLike>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserNftData{
    pub id: i32,
    pub contract_address: String,
    pub buyer_screen_cid: Option<String>,
    pub current_owner_screen_cid: String,
    pub metadata: String,
    pub img_url: String,
    pub onchain_id: Option<String>, 
    pub name: String,
    pub is_minted: bool,
    pub description: String,
    pub current_price: Option<i64>,
    pub is_listed: bool,
    pub comments: Vec<NftComment>,
    pub likes: Vec<NftLike>,
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
            
        // if asset_info.is_minted is set to false means that is not stored on contract yet
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
        - add_nft_comment
        - like_nft
        - dilike_nft
    */
    pub async fn update(caller_screen_cid: &str, 
        asset_info: UpdateUserNftData, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
        
        // asset_info.onchain_id will be fulfilled after minting
        // condition: caller_screen_cid == asset_info.current_owner_screen_cid
        // if the nft is_listed field was set to true the nft can be sold to the user
        // if sell api gets called the is_listed will be set to false automatically
        // ...

        Ok(())

    }

}