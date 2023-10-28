





use crate::{*, schema::users_collections};

use super::users_nfts::UserNftData;


/* 

    diesel migration generate users_collections ---> create users_collections migration sql files
    diesel migration run                        ---> apply sql files to db 
    diesel migration redo                       ---> drop tables 

*/
#[derive(Queryable, Selectable, Serialize, Deserialize, Insertable, Identifiable, Debug, PartialEq, Clone)]
#[diesel(table_name=users_collections)]
pub struct UserCollection{
    pub id: i32,
    pub contract_address: String,
    pub nfts: serde_json::Value, /* pg key, value based json binary object */
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: bool,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: serde_json::Value, /* pg key, value based json binary object */
    pub col_description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserCollectionData{
    pub id: i32,
    pub contract_address: String,
    pub nfts: serde_json::Value,
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: bool,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: serde_json::Value,
    pub col_description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserCollectionRequest{
    pub contract_address: String,
    pub nfts: serde_json::Value,
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: bool,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: serde_json::Value,
    pub col_description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NewUserCollectionRequest{
    pub gallery_id: i32,
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: bool,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: serde_json::Value,
    pub col_description: String,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserCollection{

    pub async fn get_all_none_minted_nfts_for(screen_cid: &str, collection_name: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserNftData>, PanelHttpResponse>{
        
        // get all collections that their nfts are not minted yet and 
        // are belong to the passed in screen_cid and are related 
        // to the passed in collection name
        // ...
        
        Ok(
            vec![UserNftData::default()]
        )

    }

    pub async fn get_all_minted_nfts_for(screen_cid: &str, collection_name: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserNftData>, PanelHttpResponse>{
        
        // get all collections that their nfts are minted and
        // are belong to the passed in screen_cid and are related 
        // to the passed in collection name
        // ...
        
        Ok(
            vec![UserNftData::default()]
        )

    }

    pub async fn get_info_by_name(col_name: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        Ok(
            UserCollectionData::default()
        )

    }

    pub async fn get_info_by_screen_cid(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        Ok(
            UserCollectionData::default()
        )

    }

    pub async fn get_all_private_collections_for(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{
        
        // retrieve collections that their nfts are not minted yet on contract
        // and their nfts' current_owner_screen_cid == screen_cid
        // ...

        Ok(
            vec![UserCollectionData::default()]
        )

    }

    pub async fn get_all_public_collections_for(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{

        // retrieve collections that their nfts are minted on contract
        // and their nfts' owner == screen_cid
        // ...

        Ok(
            vec![UserCollectionData::default()]
        )

    }

    pub async fn get_info_of(col_name: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        Ok(
            UserCollectionData::default()
        )

    }

    pub async fn get_nfts_of(col_name: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        // let collection_data = Self::get(col_name).await;
        // let nfts = collection_data.nfts
        //     .into_iter()
        //     .map(|nft_id| {
        //         let nft_data = UserNft::get(nft_id).await;
        //         nft_data
        //     })
        //     .collect::<UserNftData>();

        Ok(())

    }

}

impl UserCollection{

    pub async fn insert(new_col_info: NewUserCollectionRequest, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        // call this api https://docs.nftport.xyz/reference/deploy-nft-product-contract
        // insert the collection id into the user private gallery related to the passed in id
        
        
        // UserPrivateGallery::update(new_col_info.owner_screen_cid, new_col_info, fetched_gallery_info, connection).await;
        // ...

        Ok(())

    }

    /* ---------------------------------------------------------------------------- */
    /* this method can be called to update an collection status like royalties info */
    /* ---------------------------------------------------------------------------- */
    /* supported apis (spend token for gas fee like update royalties info):
        - update_collection ---- https://docs.nftport.xyz/reference/update-nft-product-contract
    */
    pub async fn update(caller_screen_cid: &str, new_nft_data: Option<serde_json::Value>,
        col_info: UpdateUserCollectionRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCollectionData, PanelHttpResponse>{
        
        // update balance field of royalties_address_screen_cid in each nft sell
        // condition: caller_screen_cid == col_info.owner_screen_cid
        // insert new nft data into the collection

        // if new_nft_data.is_some(){
        // let mut decoded_nfts = serde_json::from_value::<Vec<UserNftData>>(col_info.nfts).unwrap();
        // decoded_nfts.push(serde_json::to_value(&new_nft_data));
        // update col record
        // }

        Ok(
            UserCollectionData::default()
        )

    }

}