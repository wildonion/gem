





use crate::*;



#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserCollection{
    pub id: i32,
    pub contract_address: String,
    pub nfts: Vec<i32>, // sql field: INTEGER[] DEFAULT ARRAY[]::INTEGER[]
    pub name: String,
    pub symbol: String,
    pub owner_address: String, // user screen_cid of the collection owner and on chain contract
    pub metadata_updatable: bool,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address: String,
    pub collection_background: String,
    pub metadata: String, // json stringified data
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserCollection{

    pub async fn insert(private_gallery_id: i32){

        // call this api https://docs.nftport.xyz/reference/deploy-nft-product-contract
        // insert the collection id into the user private gallery related to the passed in id
        // ...

    }

    pub async fn get_none_minted_nfts_for(screen_cid: &str, collection_name: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn get_minted_nfts_for(screen_cid: &str, collection_name: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn get_info_by_name(col_name: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn get_info_by_screen_cid(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn get_all_private_collections_for(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
        
        // retrieve collections that their nfts are not minted yet on contract
        // retrieve collections that their nfts' owner == screen_cid
        // ...

        Ok(())

    }

    pub async fn get_all_public_collections_for(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        // retrieve collections that their nfts are minted on contract
        // retrieve collections that their nfts' owner == screen_cid
        // ...

        Ok(())

    }

    pub async fn get_info_of(col_name: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn add_nf_to(col_name: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

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