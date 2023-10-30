





use crate::misc::{Response, Limit};
use crate::{*, constants::COLLECTION_NOT_FOUND};
use super::users_nfts::UserNftData;
use crate::schema::users_collections::dsl::*;
use crate::schema::users_collections;


/* 

    diesel migration generate users_collections ---> create users_collections migration sql files
    diesel migration run                        ---> apply sql files to db 
    diesel migration redo                       ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_collections)]
pub struct UserCollection{
    pub id: i32,
    pub contract_address: String,
    pub nfts: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub col_description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserCollectionData{
    pub id: i32,
    pub contract_address: String,
    pub nfts: Option<serde_json::Value>,
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: Option<serde_json::Value>,
    pub col_description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserCollectionRequest{
    pub contract_address: String,
    pub nfts: Option<serde_json::Value>,
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: Option<serde_json::Value>,
    pub col_description: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, AsChangeset)]
#[diesel(table_name=users_collections)]
pub struct UpdateUserCollection{
    pub contract_address: String,
    pub nfts: Option<serde_json::Value>,
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: Option<serde_json::Value>,
    pub col_description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NewUserCollectionRequest{
    pub gallery_id: i32,
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: Option<serde_json::Value>,
    pub col_description: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Insertable)]
#[diesel(table_name=users_collections)]
pub struct InsertNewUserPrivateGalleryRequest{
    pub col_name: String,
    pub symbol: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub metadata: Option<serde_json::Value>,
    pub col_description: String,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserCollection{


    pub async fn get_all_minted_nfts_of(collection_name: &str,
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

    pub async fn find_by_id(col_id: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        Ok(
            UserCollectionData::default()
        )

    }

    pub async fn get_all_private_collections_for(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{
        
        let user_collections = users_collections
            .order(created_at.desc())
            .filter(owner_screen_cid.eq(screen_cid))
            .load::<UserCollection>(connection);
        
        
        let Ok(collections) = user_collections else{
            let resp = Response{
                data: Some(screen_cid),
                message: COLLECTION_NOT_FOUND,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            
            collections
                .into_iter()
                .map(|c|{

                    UserCollectionData{
                        id: c.id,
                        contract_address: c.contract_address,
                        nfts: {
                            /* return those none minted ones */
                            if c.nfts.is_some(){
                                let col_nfts = c.nfts;
                                let decoded_nfts = if col_nfts.is_some(){
                                    serde_json::from_value::<Vec<UserNftData>>(col_nfts.unwrap()).unwrap()
                                } else{
                                    vec![]
                                };
                                
                                let none_minted_nfts = decoded_nfts
                                    .into_iter()
                                    .map(|nft|{
                                        if nft.is_minted == false{
                                            Some(nft)
                                        } else{
                                            None
                                        }
                                    }).collect::<Vec<Option<UserNftData>>>();
                                
                                let encoded_nfts = serde_json::to_value(none_minted_nfts).unwrap();
                                Some(encoded_nfts)
        
                            } else{
                                c.nfts
                            }
                        },
                        col_name: c.col_name,
                        symbol: c.symbol,
                        owner_screen_cid: c.owner_screen_cid,
                        metadata_updatable: c.metadata_updatable,
                        base_uri: c.base_uri,
                        royalties_share: c.royalties_share,
                        royalties_address_screen_cid: c.royalties_address_screen_cid,
                        collection_background: c.collection_background,
                        metadata: c.metadata,
                        col_description: c.col_description,
                        created_at: c.created_at.to_string(),
                        updated_at: c.updated_at.to_string(),
                    }

                })
                .collect::<Vec<UserCollectionData>>()
        )

    }

    pub async fn get_all_public_collections_for(screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{


        let user_collections = users_collections
            .order(created_at.desc())
            .filter(owner_screen_cid.eq(screen_cid))
            .load::<UserCollection>(connection);
        
        let Ok(collections) = user_collections else{
            let resp = Response{
                data: Some(screen_cid),
                message: COLLECTION_NOT_FOUND,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };

        Ok(
            
            collections
                .into_iter()
                .map(|c|{

                    UserCollectionData{
                        id: c.id,
                        contract_address: c.contract_address,
                        nfts: {
                            /* return those none minted ones */
                            if c.nfts.is_some(){
                                let col_nfts = c.nfts;
                                let decoded_nfts = if col_nfts.is_some(){
                                    serde_json::from_value::<Vec<UserNftData>>(col_nfts.unwrap()).unwrap()
                                } else{
                                    vec![]
                                };
                                
                                let none_minted_nfts = decoded_nfts
                                    .into_iter()
                                    .map(|nft|{
                                        if nft.is_minted == true{
                                            Some(nft)
                                        } else{
                                            None
                                        }
                                    }).collect::<Vec<Option<UserNftData>>>();
                                
                                let encoded_nfts = serde_json::to_value(none_minted_nfts).unwrap();
                                Some(encoded_nfts)
        
                            } else{
                                c.nfts
                            }
                        },
                        col_name: c.col_name,
                        symbol: c.symbol,
                        owner_screen_cid: c.owner_screen_cid,
                        metadata_updatable: c.metadata_updatable,
                        base_uri: c.base_uri,
                        royalties_share: c.royalties_share,
                        royalties_address_screen_cid: c.royalties_address_screen_cid,
                        collection_background: c.collection_background,
                        metadata: c.metadata,
                        col_description: c.col_description,
                        created_at: c.created_at.to_string(),
                        updated_at: c.updated_at.to_string(),
                    }

                })
                .collect::<Vec<UserCollectionData>>()
        )

    }


}

impl UserCollection{

    pub async fn insert(new_col_info: NewUserCollectionRequest, mut img: Multipart,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        // call this api https://docs.nftport.xyz/reference/deploy-nft-product-contract
        // insert the collection id into the user private gallery related to the passed in id
        
        
        // UserPrivateGallery::update(new_col_info.owner_screen_cid, new_col_info, fetched_gallery_info, connection).await;
        // ...

        /*
            let new_col_data = new_collection_data.unwrap();
            let new_col_data_owners_screen_cid = new_col_data["owner_screen_cid"].as_str().unwrap();
            
            if new_col_data_owners_screen_cid != &gallery_data.owner_screen_cid{
                
                let resp = Response::<'_, &str>{
                    data: Some(&gallery_data.owner_screen_cid),
                    message: COLLECTION_NOT_OWNED_BY,
                    status: 404,
                };
    
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                )
            }


            if new_gallery_info.collections.is_some(){
                
                let new_col_data = new_gallery_info.collections.unwrap();
                let mut decoded_cols = serde_json::from_value::<Vec<UserCollectionData>>(
                    new_gallery_info.collections.unwrap()
                ).unwrap();

                decoded_cols.push(serde_json::from_value::<UserCollectionData>(new_col_data).unwrap());
                let encoded_cols = serde_json::to_value(decoded_cols).unwrap();
                new_gallery_info.collections = Some(encoded_cols);
            }
        */

        // update gal record 
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

        // update gal record

        Ok(
            UserCollectionData::default()
        )

    }

}