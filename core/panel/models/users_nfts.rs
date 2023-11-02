


use crate::*;
use crate::schema::users_nfts::dsl::*;
use crate::schema::users_nfts;

/* 

    diesel migration generate users_nfts ---> create users_nfts migration sql files
    diesel migration run                 ---> apply sql files to db 
    diesel migration redo                ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_nfts)]
pub struct UserNft{
    pub id: i32,
    pub contract_address: String,
    pub current_owner_screen_cid: String,
    pub metadata_uri: String,
    pub onchain_id: Option<String>,
    pub nft_name: String,
    pub is_minted: bool,
    pub nft_description: String,
    pub current_price: i64,
    pub is_listed: bool,
    pub freeze_metadata: Option<bool>,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub comments: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub likes: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub tx_hash: Option<String>,
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
    pub metadata_uri: String,
    pub extra: Option<serde_json::Value>,
    pub onchain_id: Option<String>,
    pub nft_name: String,
    pub is_minted: bool,
    pub nft_description: String,
    pub current_price: i64,
    pub is_listed: bool,
    pub freeze_metadata: Option<bool>,
    pub comments: Option<serde_json::Value>,
    pub likes: Option<serde_json::Value>,
    pub tx_hash: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserNftRequest{
    pub nft_id: i32,
    pub collection_id: i32,
    pub contract_address: String,
    pub buyer_screen_cid: Option<String>,
    pub current_owner_screen_cid: String,
    pub metadata_uri: String,
    pub extra: Option<serde_json::Value>,
    pub onchain_id: Option<String>, 
    pub nft_name: String,
    pub is_minted: bool,
    pub nft_description: String,
    pub current_price: i64,
    pub is_listed: bool,
    pub freeze_metadata: Option<bool>,
    pub comments: Option<serde_json::Value>,
    pub likes: Option<serde_json::Value>,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, AsChangeset)]
#[diesel(table_name=users_nfts)]
pub struct UpdateUserNft{
    pub contract_address: String,
    pub current_owner_screen_cid: String,
    pub metadata_uri: String,
    pub extra: Option<serde_json::Value>,
    pub onchain_id: Option<String>, 
    pub nft_name: String,
    pub is_minted: bool,
    pub nft_description: String,
    pub current_price: i64,
    pub is_listed: bool,
    pub freeze_metadata: Option<bool>,
    pub comments: Option<serde_json::Value>,
    pub likes: Option<serde_json::Value>,
    pub tx_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NewUserNftRequest{
    pub collection_id: i32,
    pub contract_address: String,
    pub metadata_uri: String,
    pub current_owner_screen_cid: String,
    pub nft_name: String,
    pub nft_description: String,
    pub current_price: i64,
    pub freeze_metadata: Option<bool>,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Insertable)]
#[diesel(table_name=users_nfts)]
pub struct InsertNewUserNftRequest{
    pub contract_address: String,
    pub current_owner_screen_cid: String,
    pub nft_name: String,
    pub nft_description: String,
    pub current_price: i64,
    pub freeze_metadata: Option<bool>,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserNft{

    pub async fn get_public_info_of(asset_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

}

impl UserNft{

    pub async fn insert(asset_info: NewUserNftRequest, mut img: Multipart,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
            
        // if asset_info.is_minted is set to false means that is not stored on contract yet
        // upload on pastel using sense and cascade apis: paste::sense::detect(), paste::cascade::upload()
        // spend token for gas fee and update listings
        // by default is_listed will be set to true since an nft goes to private collection by default 
        // which must be listed to be sold to friends have been invited by the gallery owner
        
        // ...

        // update col record nfts
        // then update gal record with updated col

        Ok(())

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn update(caller_screen_cid: &str, asset_info: UpdateUserNftRequest, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
        
        // update balance field of royalties_address_screen_cid in each nft sell
        /* consider royalties process of the contract based on in-app token */
        // asset_info.onchain_id will be fulfilled after minting
        // condition: caller_screen_cid == asset_info.current_owner_screen_cid
        // if the nft is_listed field was set to true the nft can be sold to the user
        // if sell api gets called the is_listed will be set to false automatically
        // ...

        // if new_nft_data.is_some(){
        // let mut decoded_nfts = serde_json::from_value::<Vec<UserNftData>>(col_info.nfts).unwrap();
        // decoded_nfts.push(serde_json::to_value(&new_nft_data));
        // update col record
        // }
        // let nft_comments = serde_json::from_value::<NftComment>(asset_info.comments).unwrap();
        // let nft_likes = serde_json::from_value::<NftLike>(asset_info.comments).unwrap();

        // update col record nfts
        // then update gal record with updated col

        // onchain updates (fill the tx hash field) | https://docs.nftport.xyz/reference/update-minted-nft
        // - metadata_uri : contains json includes nft img url and extra json
        // - freeze_metadata

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

        Ok(())

    }

}