


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
    pub metadata_uri: String, /* an ipfs link contains metadata json file */
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

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
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
    pub caller_screen_cid: String,
    pub contract_address: i32,
    pub event_type: String,
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
    pub caller_screen_cid: String,
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
    pub metadata_uri: String,
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

    pub async fn find_by_current_owner(current_owner: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn find_by_id(asset_id: i32, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }
    

}

impl UserNft{

    pub async fn insert(asset_info: NewUserNftRequest, mut img: Multipart,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
            
        // if asset_info.is_minted is set to false means that is not stored on contract yet
        // upload img on pastel using sense and cascade apis: paste::sense::detect(), paste::cascade::upload()
        // spend token for gas fee and update listings
        // by default is_listed will be set to true since an nft goes to private collection by default 
        // which must be listed to be sold to friends have been invited by the gallery owner

        
        /* 
        
            // let col_info = UserCollection::find_by_contract_address(asset_info.contract_address).await.unwrap();
            // let gal_info = UserPrivateGallery::find_by_owner(col_info.owner_screen_cid).await.unwrap();
            if gal_info.owner_screen_cid != asset_info.caller_screen_cid{
                // can't put nft in a collection not owned by you
                // ...
            }
        
        */


        // update col 
        // update gal

        Ok(())

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn update(asset_info: UpdateUserNftRequest, mut img: Multipart,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        /* 
        
            // let nft_info = UserCollection::find_by_current_owner(asset_info.caller_screen_cid).await.unwrap();
            // if didn't find any nft matches with caller_screen_cid means 
            // that the caller is not current nft owner cause only owner 
            // can update the nft 

            // let col_info = UserCollection::find_by_contract_address(nft_info.contract_address).await.unwrap();
            // let gal_info = UserPrivateGallery::find_by_owner(col_info.owner_screen_cid).await.unwrap();
        
            // update col 
            // update gal 

        */


        match asset_info.event_type.as_str(){
            "mint" => {

                /* ------- charge user balance for gas fee ------- */
                // https://docs.nftport.xyz/reference/customizable-minting
                // asset_info.onchain_id will be fulfilled after minting
                // call nftport::mint_nft()
                todo!()
            },
            "transfer" => {

                /* ------- charge user balance for gas fee ------- */
                // https://docs.nftport.xyz/reference/transfer-minted-nft
                // call nftport::transfer_nft()
                todo!()
            },
            "sell" => {
                
                // update is_listed field
                todo!()
            },
            "buy" => {
                
                /* ------- charge user balance for gas fee ------- */
                // update balance field of royalties_address_screen_cid in each nft sell
                // if the nft is_listed field was set to true then nft can be sold out to the asset_info.buyer_screen_cid
                // transfer nft ownership to the asset_info.buyer_screen_cid
                /* consider royalties process of the contract based on in-app token */
                // call nftport::mint_nft()
                todo!()
            },
            "like" => {
                todo!()
            },
            "dislike" => {
                todo!()
            },
            "comment" => {
                todo!()
            },
            "onchain-update" => {

                /* ------- charge user balance for gas fee ------- */
                // upload img on pastel using sense and cascade apis: paste::sense::detect(), paste::cascade::upload()
                // onchain updates (fill the tx hash field) | https://docs.nftport.xyz/reference/update-minted-nft
                // - metadata_uri : contains json includes nft img url and extra json
                // - freeze_metadata
                // call nftport::update_nft()
                todo!()

            },
            _ => {
                todo!() // invalid event_type
            }
        }
        

    }

}