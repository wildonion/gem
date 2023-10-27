





use crate::*;

use super::users_nfts::UserNftData;



#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserCollection{
    pub id: i32,
    pub contract_address: String,
    pub nfts: Vec<i32>,
    pub name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: bool,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: String, // json stringified data like collection statistics
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserCollectionData{
    pub id: i32,
    pub contract_address: String,
    pub nfts: Vec<UserNftData>,
    pub name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: bool,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserCollectionData{
    pub id: i32,
    pub contract_address: String,
    pub nfts: Vec<i32>,
    pub name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: bool,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
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

    pub async fn insert(private_gallery_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        // call this api https://docs.nftport.xyz/reference/deploy-nft-product-contract
        // insert the collection id into the user private gallery related to the passed in id
        // ...

        Ok(())

    }

    /* ---------------------------------------------------------------------------- */
    /* this method can be called to update an collection status like royalties info */
    /* ---------------------------------------------------------------------------- */
    /* supported apis (spend token for gas fee like update royalties info):
        - update_collection ---- https://docs.nftport.xyz/reference/update-nft-product-contract
    */
    pub async fn update(caller_screen_cid: &str,
        col_info: UpdateUserCollectionData, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCollectionData, PanelHttpResponse>{
        
        // update balance field of royalties_address_screen_cid in each nft sell
        // condition: caller_screen_cid == col_info.owner_screen_cid

        Ok(
            UserCollectionData::default()
        )

    }

}