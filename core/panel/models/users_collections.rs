

use std::time::{SystemTime, UNIX_EPOCH};
use wallexerr::Wallet;
use crate::adapters::nftport;
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_IMAGE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN};
use crate::misc::{Response, Limit};
use crate::{*, constants::COLLECTION_NOT_FOUND_OF};
use super::users::User;
use super::users_galleries::{UserPrivateGalleryData, UserPrivateGallery, UpdateUserPrivateGallery, UpdateUserPrivateGalleryRequest};
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
    pub freeze_metadata: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub col_description: String,
    pub contract_tx_hash: Option<String>,
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
    pub freeze_metadata: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub extra: Option<serde_json::Value>,
    pub col_description: String,
    pub contract_tx_hash: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserCollectionRequest{
    pub collection_id: i32,
    pub gallery_id: i32,
    pub amount: i64,
    pub freeze_metadata: bool,
    pub nfts: Option<serde_json::Value>,
    pub owner_cid: String,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub extra: Option<serde_json::Value>,
    pub col_description: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, AsChangeset)]
#[diesel(table_name=users_collections)]
pub struct UpdateUserCollection{
    pub nfts: Option<serde_json::Value>,
    pub freeze_metadata: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub extra: Option<serde_json::Value>,
    pub col_description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NewUserCollectionRequest{
    pub gallery_id: i32,
    pub amount: i64,
    pub col_name: String,
    pub symbol: String,
    pub owner_cid: String,
    pub metadata_updatable: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32, // in-app token amount
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub extra: Option<serde_json::Value>,
    pub col_description: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Insertable)]
#[diesel(table_name=users_collections)]
pub struct InsertNewUserCollectionRequest{
    pub col_name: String,
    pub symbol: String,
    pub contract_address: String,
    pub owner_screen_cid: String,
    pub metadata_updatable: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub collection_background: String,
    pub extra: Option<serde_json::Value>,
    pub contract_tx_hash: String,
    pub col_description: String,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserCollection{


    pub async fn get_all_minted_nfts_of_collection(col_id: i32, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<Option<UserNftData>>, PanelHttpResponse>{


        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let get_collection = Self::find_by_id(col_id, connection).await;
        let Ok(collection) = get_collection else{
            let error_resp = get_collection.unwrap_err();
            return Err(error_resp);
        };

        let nfts_ = collection.clone().nfts;
        let decoded_nfts = if nfts_.is_some(){
            serde_json::from_value::<Vec<UserNftData>>(nfts_.clone().unwrap()).unwrap()
        } else{
            vec![]
        };


        let minted_ones = decoded_nfts
            .into_iter()
            .map(|nft|{
                if nft.is_minted == true{
                    Some(nft)
                } else{
                    None
                }
            })
            .collect::<Vec<Option<UserNftData>>>();
        
        let sliced = &minted_ones[from..to+1].to_vec();
        
        Ok(
            sliced.to_owned()
        )

    }

    pub async fn find_by_id(col_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        let user_collection = users_collections
            .filter(users_collections::id.eq(col_id))
            .first::<UserCollection>(connection);

        let Ok(collection) = user_collection else{

            let resp = Response{
                data: Some(col_id),
                message: COLLECTION_NOT_FOUND_OF,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };


        Ok(
            UserCollectionData{
                id: collection.id,
                contract_address: collection.contract_address,
                nfts: collection.nfts,
                col_name: collection.col_name,
                symbol: collection.symbol,
                owner_screen_cid: collection.owner_screen_cid,
                metadata_updatable: collection.metadata_updatable,
                base_uri: collection.base_uri,
                royalties_share: collection.royalties_share,
                royalties_address_screen_cid: collection.royalties_address_screen_cid,
                collection_background: collection.collection_background,
                extra: collection.extra,
                col_description: collection.col_description,
                created_at: collection.created_at.to_string(),
                updated_at: collection.updated_at.to_string(),
                freeze_metadata: collection.freeze_metadata,
                contract_tx_hash: collection.contract_tx_hash,
            }
        )

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn get_all_private_collections_for(caller_screen_cid: &str, gal_id: i32,
        limit: web::Query<Limit>, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{
            
        let get_gallery_data = UserPrivateGallery::find_by_id(gal_id, connection).await;
        let Ok(gallery_data) = get_gallery_data else{
            let error_resp = get_gallery_data.unwrap_err();
            return Err(error_resp);
        };

        if gallery_data.owner_screen_cid != caller_screen_cid{
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let user_collections = users_collections
            .order(created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .filter(owner_screen_cid.eq(caller_screen_cid))
            .load::<UserCollection>(connection);
        
        
        let Ok(collections) = user_collections else{
            let resp = Response::<String>{
                data: Some(caller_screen_cid.to_string()),
                message: COLLECTION_NOT_FOUND_FOR,
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
                        extra: c.extra,
                        col_description: c.col_description,
                        created_at: c.created_at.to_string(),
                        updated_at: c.updated_at.to_string(),
                        freeze_metadata: c.freeze_metadata,
                        contract_tx_hash: c.contract_tx_hash,
                    }

                })
                .collect::<Vec<UserCollectionData>>()
        )

    }

    pub async fn get_all_public_collections_for(screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let user_collections = users_collections
            .order(created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .filter(owner_screen_cid.eq(screen_cid))
            .load::<UserCollection>(connection);
        
        let Ok(collections) = user_collections else{
            let resp = Response{
                data: Some(screen_cid),
                message: COLLECTION_NOT_FOUND_FOR,
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
                        extra: c.extra,
                        col_description: c.col_description,
                        created_at: c.created_at.to_string(),
                        updated_at: c.updated_at.to_string(),
                        freeze_metadata: c.freeze_metadata,
                        contract_tx_hash: c.contract_tx_hash,
                    }

                })
                .collect::<Vec<UserCollectionData>>()
        )

    }


}

impl UserCollection{

    pub async fn insert(new_col_info: NewUserCollectionRequest, mut img: Multipart,
        redis_client: redis::Client, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        let caller_screen_cid = Wallet::generate_keccak256_from(new_col_info.clone().owner_cid);
        /* caller must be in db */
        let Ok(user) = User::find_by_screen_cid(
            &caller_screen_cid, connection).await 
            else{
                let resp = Response{
                    data: Some(new_col_info.owner_cid),
                    message: USER_SCREEN_CID_NOT_FOUND,
                    status: 404
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
        };

        /* caller must be the gallery owner */
        let get_gallery_data = UserPrivateGallery::find_by_id(new_col_info.clone().gallery_id, connection).await;
        let Ok(gallery_data) = get_gallery_data else{

            let err_resp = get_gallery_data.unwrap_err();
            return Err(err_resp);
        };

        if gallery_data.owner_screen_cid != caller_screen_cid{
    
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        /* getting onchain contract information */
        let (contract_onchain_address, contract_create_tx_hash, status) = nftport::create_collection(redis_client, new_col_info.clone()).await;
        
        if status == 1 && contract_onchain_address == String::from("") && 
            contract_create_tx_hash == String::from(""){

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: CANT_CREATE_COLLECTION_ONCHAIN,
                status: 417,
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        }

        if !contract_create_tx_hash.starts_with("0x"){

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: INVALID_CONTRACT_TX_HASH,
                status: 417,
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        } 

        if status == 1 && contract_onchain_address == String::from("") && 
            contract_create_tx_hash.starts_with("0x"){

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: CANT_GET_CONTRACT_ADDRESS,
                status: 417,
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        }   
    
        /* uploading collection image */
        let get_collection_img_path = misc::upload_img(
            COLLECTION_UPLOAD_PATH, &contract_onchain_address, 
            "collection", 
            img).await;
        let Ok(collection_img_path) = get_collection_img_path else{

            let err_res = get_collection_img_path.unwrap_err();
            return Err(err_res);
        };
        
        /* 
            update user balance frist, if anything goes wrong they can call us 
            to pay them back, actually this is the gas fee that they must be 
            charged for since we already have paid the fee when we created 
            the contract collection
        */
        let new_balance = user.balance.unwrap() - new_col_info.amount;
        let update_user_balance = User::update_balance(user.id, new_balance, connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        let new_col_data = InsertNewUserCollectionRequest{
            col_name: new_col_info.clone().col_name,
            symbol: new_col_info.clone().symbol,
            contract_address: contract_onchain_address, /* NEW */
            owner_screen_cid: Wallet::generate_keccak256_from(new_col_info.clone().owner_cid),
            metadata_updatable: new_col_info.clone().metadata_updatable,
            base_uri: new_col_info.clone().base_uri,
            royalties_share: new_col_info.clone().royalties_share,
            royalties_address_screen_cid: new_col_info.clone().royalties_address_screen_cid,
            collection_background: collection_img_path, /* NEW */
            extra: new_col_info.clone().extra,
            col_description: new_col_info.clone().col_description,
            contract_tx_hash: contract_create_tx_hash
        };
    
        match diesel::insert_into(users_collections)
            .values(&new_col_data)
            .returning(UserCollection::as_returning())
            .get_result::<UserCollection>(connection)
            {
                Ok(fetched_collection_data) => {
                    
                    let user_collection_data = UserCollectionData{

                        extra: fetched_collection_data.clone().extra,
                        id: fetched_collection_data.clone().id,
                        contract_address: fetched_collection_data.clone().contract_address,
                        nfts: fetched_collection_data.clone().nfts,
                        col_name: fetched_collection_data.clone().col_name,
                        symbol: fetched_collection_data.clone().symbol,
                        owner_screen_cid: fetched_collection_data.clone().owner_screen_cid,
                        metadata_updatable: fetched_collection_data.clone().metadata_updatable,
                        base_uri: fetched_collection_data.clone().base_uri,
                        royalties_share: fetched_collection_data.clone().royalties_share,
                        royalties_address_screen_cid: fetched_collection_data.clone().royalties_address_screen_cid,
                        collection_background: fetched_collection_data.clone().collection_background,
                        col_description: fetched_collection_data.clone().col_description,
                        created_at: fetched_collection_data.clone().created_at.to_string(),
                        updated_at: fetched_collection_data.clone().updated_at.to_string(),
                        freeze_metadata: fetched_collection_data.clone().freeze_metadata,
                        contract_tx_hash: fetched_collection_data.clone().contract_tx_hash,
                    };

                    /* updating gallery data */
                    let new_gal_data = UpdateUserPrivateGalleryRequest{
                        collections: {
                            let cols = gallery_data.collections;
                            let mut decoded_cols = if cols.is_some(){
                                serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
                            } else{
                                vec![]
                            };

                            decoded_cols.push(user_collection_data.clone());

                            Some(
                                serde_json::to_value(decoded_cols).unwrap()
                            )
                        },
                        gal_name: gallery_data.gal_name,
                        gal_description: gallery_data.gal_description,
                        invited_friends: gallery_data.invited_friends,
                        extra: gallery_data.extra,
                        owner_cid: new_col_info.clone().owner_cid,
                        tx_signature: String::from(""),
                        hash_data: String::from(""),
                    };

                    /* update gallery with new collection */
                    match UserPrivateGallery::update(
                        &fetched_collection_data.owner_screen_cid, 
                        new_gal_data, 
                        gallery_data.id, 
                        connection
                    ).await{

                        Ok(updated_gal) => Ok(user_collection_data),
                        Err(resp) => Err(resp)
                    }


                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserCollection::insert");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn update(col_info: UpdateUserCollectionRequest, mut img: Option<Multipart>, 
        redis_client: redis::Client, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserCollectionData, PanelHttpResponse>{
        
        let collection_owner_screen_cid = Wallet::generate_keccak256_from(col_info.clone().owner_cid);
        
        /* caller must be in db */
        let Ok(user) = User::find_by_screen_cid(
            &collection_owner_screen_cid, connection).await 
            else{
                let resp = Response{
                    data: Some(col_info.owner_cid),
                    message: USER_SCREEN_CID_NOT_FOUND,
                    status: 404
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
        };

        /* getting collection data */
        let get_collection_data = Self::find_by_id(col_info.collection_id, connection).await;
        let Ok(collection_data) = get_collection_data else{

            /* collection not found response */
            let err_resp = get_collection_data.unwrap_err();
            return Err(err_resp);
        };

        /* caller must be the collection owner */
        if collection_owner_screen_cid != collection_data.owner_screen_cid{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: COLLECTION_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
               
        }

        /* getting gallery data */        
        let get_gallery_data = UserPrivateGallery::find_by_id(col_info.gallery_id, connection).await;
        let Ok(gallery_data) = get_gallery_data else{

            let err_resp = get_gallery_data.unwrap_err();
            return Err(err_resp);
        };

        /* caller must be the gallery owner */
        if gallery_data.owner_screen_cid != collection_owner_screen_cid{
    
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }


        /* updating onchain contract information */
        let (contract_update_tx_hash, status) = nftport::update_collection(
            redis_client, 
            col_info.clone(), 
            collection_data.contract_address.clone()).await;
        
        if status == 1 && contract_update_tx_hash == String::from(""){

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: CANT_UPDATE_COLLECTION_ONCHAIN,
                status: 417,
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        }  

        if !contract_update_tx_hash.starts_with("0x"){

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: INVALID_CONTRACT_TX_HASH,
                status: 417,
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        }

    
        /* uploading collection image */
        let fill_me_with_col_img_path = if img.is_some(){
            let get_collection_img_path = misc::upload_img(
                COLLECTION_UPLOAD_PATH, &collection_data.contract_address, 
                "collection", 
                img.unwrap()).await;
            let Ok(collection_img_path) = get_collection_img_path else{
                
                let err_res = get_collection_img_path.unwrap_err();
                return Err(err_res);
            };

            collection_img_path

        } else{
            String::from("")
        };


        /* 
            updating user balance frist, if anything goes wrong they can call us 
            to pay them back, actually this is the gas fee that they must be 
            charged for since we already have paid the fee when we updated  
            the contract collection
        */
        let new_balance = user.balance.unwrap() - col_info.amount;
        let update_user_balance = User::update_balance(user.id, new_balance, connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        /* if the onchain data was ok we simply update the record based on the data updated onchain */
        let new_col_data = UpdateUserCollection{
            nfts: col_info.clone().nfts,
            base_uri: col_info.clone().base_uri,
            royalties_share: col_info.clone().royalties_share,
            royalties_address_screen_cid: col_info.clone().royalties_address_screen_cid,
            collection_background: if fill_me_with_col_img_path.is_empty(){
                collection_data.collection_background
            } else{
                fill_me_with_col_img_path
            },
            extra: col_info.clone().extra,
            col_description: col_info.clone().col_description,
            freeze_metadata: Some(col_info.clone().freeze_metadata),
        };
    
        match diesel::update(users_collections)
            .set(&new_col_data)
            .returning(UserCollection::as_returning())
            .get_result::<UserCollection>(connection)
            {
                Ok(fetched_collection_data) => {
                    
                    let user_collection_data = UserCollectionData{

                        extra: fetched_collection_data.clone().extra,
                        id: fetched_collection_data.clone().id,
                        contract_address: fetched_collection_data.clone().contract_address,
                        nfts: fetched_collection_data.clone().nfts,
                        col_name: fetched_collection_data.clone().col_name,
                        symbol: fetched_collection_data.clone().symbol,
                        owner_screen_cid: fetched_collection_data.clone().owner_screen_cid,
                        metadata_updatable: fetched_collection_data.clone().metadata_updatable,
                        base_uri: fetched_collection_data.clone().base_uri,
                        royalties_share: fetched_collection_data.clone().royalties_share,
                        royalties_address_screen_cid: fetched_collection_data.clone().royalties_address_screen_cid,
                        collection_background: fetched_collection_data.clone().collection_background,
                        col_description: fetched_collection_data.clone().col_description,
                        created_at: fetched_collection_data.clone().created_at.to_string(),
                        updated_at: fetched_collection_data.clone().updated_at.to_string(),
                        freeze_metadata: fetched_collection_data.clone().freeze_metadata,
                        contract_tx_hash: fetched_collection_data.clone().contract_tx_hash,
                    };

                    /* updating gallery data */
                    let new_gal_data = UpdateUserPrivateGalleryRequest{
                        collections: {
                            let cols = gallery_data.collections;
                            let mut decoded_cols = if cols.is_some(){
                                serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
                            } else{
                                vec![]
                            };

                            decoded_cols.push(user_collection_data.clone());

                            Some(
                                serde_json::to_value(decoded_cols).unwrap()
                            )
                        },
                        gal_name: gallery_data.gal_name,
                        gal_description: gallery_data.gal_description,
                        invited_friends: gallery_data.invited_friends,
                        extra: gallery_data.extra,
                        owner_cid: col_info.clone().owner_cid,
                        tx_signature: String::from(""),
                        hash_data: String::from(""),
                    };

                    /* update gallery with new collection */
                    match UserPrivateGallery::update(
                        &fetched_collection_data.owner_screen_cid, 
                        new_gal_data, 
                        gallery_data.id, 
                        connection
                    ).await{

                        Ok(updated_gal) => Ok(user_collection_data),
                        Err(resp) => Err(resp)
                    }


                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserCollection::update");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

}