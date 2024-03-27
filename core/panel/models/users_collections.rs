
use rand::seq::SliceRandom;
use std::time::{SystemTime, UNIX_EPOCH};
use actix::Addr;
use chrono::NaiveDateTime;
use crate::events::publishers::action::{SingleUserNotif, NotifData, ActionType};
use crate::schema::users_galleries::dsl::users_galleries;
use crate::adapters::nftport;
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN, COLLECTION_NOT_FOUND_FOR_CONTRACT, COLLECTION_NOT_FOUND, COLLECTIONS, CALLER_CANT_VIEW_GALLERY, GALLERY_HAS_NO_INVITED_FRIENDS_YET, CANT_UPDATE_FROZEN_COLLECTION_ONCHAIN};
use crate::helpers::misc::{Response, Limit};
use crate::{*, constants::COLLECTION_NOT_FOUND_OF};
use self::constants::COLLECTION_ROYALTY_IS_EXCEEDED;

use super::users::{User, UserWalletInfoResponse, UserData};
use super::users_fans::{FriendData, UserFan};
use super::users_galleries::{UserPrivateGalleryData, UserPrivateGallery, UpdateUserPrivateGallery, UpdateUserPrivateGalleryRequest};
use super::users_nfts::{UserNft, UserNftData};
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
pub struct CollectionInfoResponse{
    pub id: i32,
    pub contract_address: String,
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
    pub extra: Option<serde_json::Value>,
    pub col_description: String,
    pub col_name: String,
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
    pub contract_tx_hash: String,
    pub col_name: String,
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
    pub extra: Option<serde_json::Value>,
    pub contract_tx_hash: String,
    pub col_description: String,
    pub collection_background: String
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct CollectionOwnerCount{
    pub owner_wallet_info: UserWalletInfoResponse,
    pub collections_count: usize,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserCollection{

    pub async fn get_all_by_owner(owner: &str, 
        connection: &mut DbPoolConnection) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{

        /* get all nfts owned by the passed in current_owner */
        let users_collections_ = users_collections
            .filter(users_collections::owner_screen_cid.eq(owner))
            .load::<UserCollection>(connection);

        let Ok(all_collections) = users_collections_ else{

            let resp = Response{
                data: Some(owner),
                message: COLLECTION_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )

        };

        Ok(
            all_collections
                .into_iter()
                .map(|collection|{
    
                    UserCollectionData{
                        id: collection.id,
                        contract_address: collection.clone().contract_address,
                        nfts: {
                            let get_nfts = UserNft::get_all_inside_contract_none_async(&collection.contract_address, connection);
                            let nfts_ = if get_nfts.is_ok(){
                                get_nfts.unwrap()
                            } else{
                                vec![]
                            };
                            Some(
                                serde_json::to_value(&nfts_).unwrap()
                            )
                        },
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
    
                })
                .collect::<Vec<UserCollectionData>>()
        )

    }

    pub async fn get_all(
        connection: &mut DbPoolConnection) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{

        /* get all nfts owned by the passed in current_owner */
        let users_collections_ = users_collections
            .load::<UserCollection>(connection);

        let Ok(all_collections) = users_collections_ else{

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: COLLECTION_NOT_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        Ok(
            all_collections
                .into_iter()
                .map(|collection|{
    
                    UserCollectionData{
                        id: collection.id,
                        contract_address: collection.clone().contract_address,
                        nfts: {
                            let get_nfts = UserNft::get_all_inside_contract_none_async(&collection.contract_address, connection);
                            let nfts_ = if get_nfts.is_ok(){
                                get_nfts.unwrap()
                            } else{
                                vec![]
                            };
                            Some(
                                serde_json::to_value(&nfts_).unwrap()
                            )
                        },
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
    
                })
                .collect::<Vec<UserCollectionData>>()
        )

    }

    pub fn get_all_nfts_of_collection(col_id: i32, connection: &mut DbPoolConnection)
        -> Result<UserCollectionData, PanelHttpResponse>{

        /* get all nfts owned by the passed in current_owner */
        let users_collection = users_collections
            .filter(users_collections::id.eq(col_id))
            .first::<UserCollection>(connection);

        let Ok(col_info) = users_collection else{

            let resp = Response{
                data: Some(col_id),
                message: COLLECTION_NOT_FOUND_OF,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )

        };

        Ok(
            UserCollectionData{
                id: col_info.id,
                contract_address: col_info.clone().contract_address,
                nfts: {
                    let get_nfts = UserNft::get_all_inside_contract_none_async(&col_info.contract_address, connection);
                    let nfts_ = if get_nfts.is_ok(){
                        get_nfts.unwrap()
                    } else{
                        vec![]
                    };
                    Some(
                        serde_json::to_value(&nfts_).unwrap()
                    )
                },
                col_name: col_info.col_name,
                symbol: col_info.symbol,
                owner_screen_cid: col_info.owner_screen_cid,
                metadata_updatable: col_info.metadata_updatable,
                freeze_metadata: col_info.freeze_metadata,
                base_uri: col_info.base_uri,
                royalties_share: col_info.royalties_share,
                royalties_address_screen_cid: col_info.royalties_address_screen_cid,
                collection_background: col_info.collection_background,
                extra: col_info.extra,
                col_description: col_info.col_description,
                contract_tx_hash: col_info.contract_tx_hash,
                created_at: col_info.created_at.to_string(),
                updated_at: col_info.updated_at.to_string(),
            }
        )

    } 

    pub async fn get_all_nft_product_collections_by_owner(owner: &str, 
        connection: &mut DbPoolConnection) 
        -> Result<Vec<CollectionInfoResponse>, PanelHttpResponse>{

        /* get all nfts owned by the passed in current_owner */
        let users_collections_ = users_collections
            .filter(users_collections::owner_screen_cid.eq(owner))
            .load::<UserCollection>(connection);

        let Ok(all_collections) = users_collections_ else{

            let resp = Response{
                data: Some(owner),
                message: COLLECTION_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )

        };

        Ok(
            all_collections
                .into_iter()
                .map(|collection|{
    
                    CollectionInfoResponse{
                        id: collection.id,
                        contract_address: collection.contract_address,
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
    
                })
                .collect::<Vec<CollectionInfoResponse>>()
        )

    }

    pub async fn get_owners_with_lots_of_collections(owners: Vec<UserData>, connection: &mut DbPoolConnection) 
        -> Result<Vec<CollectionOwnerCount>, PanelHttpResponse>{

            let mut collection_owner_map = vec![];
            for owner in owners{

                if owner.screen_cid.is_none() || owner.screen_cid.clone().unwrap().is_empty(){
                    continue;
                }
    
                let owner_screen_cid_ = owner.screen_cid.unwrap();
                let get_all_collections_owned_by = UserCollection::get_all_by_owner(&owner_screen_cid_, connection).await;
                let collections_owned_by = if get_all_collections_owned_by.is_ok(){
                    get_all_collections_owned_by.unwrap()
                } else{
                    vec![]
                };
    
                let user = User::find_by_screen_cid(&owner_screen_cid_, connection).await.unwrap();
                let user_wallet_info = UserWalletInfoResponse{
                    username: user.username,
                    avatar: user.avatar,
                    bio: user.bio,
                    banner: user.banner,
                    mail: user.mail,
                    screen_cid: user.screen_cid,
                    extra: user.extra,
                    stars: user.stars,
                    created_at: user.created_at.to_string(),
                };
    
                collection_owner_map.push(
                    CollectionOwnerCount{
                        owner_wallet_info: user_wallet_info,
                        collections_count: collections_owned_by.len()
                    }
                )
            }
    
            collection_owner_map.sort_by(|col1, col2|{
    
                let col1_count = col1.collections_count;
                let col2_count = col2.collections_count;
    
                col2_count.cmp(&col1_count)
    
            });
            
        Ok(collection_owner_map)
                
    }

    pub async fn get_all_minted_nfts_of_collection(col_id: i32, limit: web::Query<Limit>, 
        caller_screen_cid: &str, connection: &mut DbPoolConnection) 
        -> Result<Vec<Option<UserNftData>>, PanelHttpResponse>{


        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
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

        let get_nfts = UserNft::get_all_inside_contract_none_async(&collection.contract_address, connection);
        let nfts_ = if get_nfts.is_ok(){
            get_nfts.unwrap()
        } else{
            vec![]
        };

        let mut minted_ones = nfts_
            .into_iter()
            .map(|nft|{
                /* if we couldn't unwrap the is_minted means it's not minted yet and it's false */
                if 
                    nft.is_minted.unwrap_or(false) == true{
                    Some(nft)
                } else{
                    None
                }
            })
            .collect::<Vec<Option<UserNftData>>>();
        
        minted_ones.retain(|nft| nft.is_some());

        /* sorting nfts in desc order */
        minted_ones.sort_by(|nft1, nft2|{
            /* 
                cannot move out of `*nft1` which is behind a shared reference
                move occurs because `*nft1` has type `std::option::Option<UserNftData>`, 
                which does not implement the `Copy` trait and unwrap() takes the 
                ownership of the instance.
                also we must create a longer lifetime for `UserNftData::default()` by 
                putting it inside a type so we can take a reference to it and pass the 
                reference to the `unwrap_or()`, cause &UserNftData::default() will be dropped 
                at the end of the `unwrap_or()` statement while we're borrowing it.
            */
            let nft1_default = UserNftData::default();
            let nft2_default = UserNftData::default();
            let nft1 = nft1.as_ref().unwrap_or(&nft1_default);
            let nft2 = nft2.as_ref().unwrap_or(&nft2_default);

            let nft1_created_at = NaiveDateTime
                ::parse_from_str(&nft1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

            let nft2_created_at = NaiveDateTime
                ::parse_from_str(&nft2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

            nft2_created_at.cmp(&nft1_created_at)

        });
        
        /*  
            first we need to slice the current vector convert that type into 
            another vector, the reason behind doing this is becasue we can't
            call to_vec() on the slice directly since the lifetime fo the slice
            will be dropped while is getting used we have to create a longer 
            lifetime then call to_vec() on that type
        */
        let sliced = if from < minted_ones.len(){
            if minted_ones.len() > to{
                let data = &minted_ones[from..to+1];
                data.to_vec()
            } else{
                let data = &minted_ones[from..minted_ones.len()];
                data.to_vec()
            }
        } else{
            vec![]
        };

        Ok(
            if sliced.contains(&None){
                vec![]
            } else{
                sliced.to_owned()
            }
        )


    }

    pub fn get_all_pure_minted_nfts_of_collection_with_address(col_addr: &str,
        connection: &mut DbPoolConnection) 
        -> Result<Vec<Option<UserNftData>>, PanelHttpResponse>{


        let get_nfts = UserNft::get_all_inside_contract_none_async(col_addr, connection);
        let Ok(nfts_) = get_nfts else{
            let error_resp = get_nfts.unwrap_err();
            return Err(error_resp);
        };

        let mut minted_ones = nfts_
            .into_iter()
            .map(|nft|{
                /* if we couldn't unwrap the is_minted means it's not minted yet and it's false */
                if nft.is_minted.unwrap_or(false) == true{
                    Some(nft)
                } else{
                    None
                }
            })
            .collect::<Vec<Option<UserNftData>>>();
        
        minted_ones.retain(|nft| nft.is_some());

        Ok(
            minted_ones
        )


    }

    pub async fn find_by_contract_address(col_contract_address: &str, connection: &mut DbPoolConnection) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        let user_collection = users_collections
            .filter(users_collections::contract_address.eq(col_contract_address))
            .first::<UserCollection>(connection);

        let Ok(collection) = user_collection else{

            let resp = Response{
                data: Some(col_contract_address),
                message: COLLECTION_NOT_FOUND_FOR_CONTRACT,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };


        Ok(
            UserCollectionData{
                id: collection.id,
                contract_address: collection.clone().contract_address,
                nfts: {
                    let get_nfts = UserNft::get_all_inside_contract_none_async(&collection.contract_address, connection);
                    let nfts_ = if get_nfts.is_ok(){
                        get_nfts.unwrap()
                    } else{
                        vec![]
                    };
                    Some(
                        serde_json::to_value(&nfts_).unwrap()
                    )
                },
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

    pub fn find_by_contract_address_none_async(col_contract_address: &str, connection: &mut DbPoolConnection) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        let user_collection = users_collections
            .filter(users_collections::contract_address.eq(col_contract_address))
            .first::<UserCollection>(connection);

        let Ok(collection) = user_collection else{

            let resp = Response{
                data: Some(col_contract_address),
                message: COLLECTION_NOT_FOUND_FOR_CONTRACT,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };


        Ok(
            UserCollectionData{
                id: collection.id,
                contract_address: collection.clone().contract_address,
                nfts: {
                    let get_nfts = UserNft::get_all_inside_contract_none_async(&collection.contract_address, connection);
                    let nfts_ = if get_nfts.is_ok(){
                        get_nfts.unwrap()
                    } else{
                        vec![]
                    };
                    Some(
                        serde_json::to_value(&nfts_).unwrap()
                    )
                },
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

    pub async fn find_by_id(col_id: i32, connection: &mut DbPoolConnection) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        let user_collection = users_collections
            .filter(users_collections::id.eq(col_id))
            .first::<UserCollection>(connection);

        let Ok(collection) = user_collection else{

            let resp = Response{
                data: Some(col_id),
                message: COLLECTION_NOT_FOUND_OF,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };


        Ok(
            UserCollectionData{
                id: collection.id,
                contract_address: collection.clone().contract_address,
                nfts: {
                    let get_nfts = UserNft::get_all_inside_contract_none_async(&collection.contract_address, connection);
                    let nfts_ = if get_nfts.is_ok(){
                        get_nfts.unwrap()
                    } else{
                        vec![]
                    };
                    Some(
                        serde_json::to_value(&nfts_).unwrap()
                    )
                },
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

    pub fn find_by_id_none_async(col_id: i32, connection: &mut DbPoolConnection) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        let user_collection = users_collections
            .filter(users_collections::id.eq(col_id))
            .first::<UserCollection>(connection);

        let Ok(collection) = user_collection else{

            let resp = Response{
                data: Some(col_id),
                message: COLLECTION_NOT_FOUND_OF,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };


        Ok(
            UserCollectionData{
                id: collection.id,
                contract_address: collection.clone().contract_address,
                nfts: {
                    let get_nfts = UserNft::get_all_inside_contract_none_async(&collection.contract_address, connection);
                    let nfts_ = if get_nfts.is_ok(){
                        get_nfts.unwrap()
                    } else{
                        vec![]
                    };
                    Some(
                        serde_json::to_value(&nfts_).unwrap()
                    )
                },
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
        limit: web::Query<Limit>, redis_client: RedisClient, connection: &mut DbPoolConnection) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{
            
        let get_gallery_data = UserPrivateGallery::find_by_id(gal_id, redis_client.clone(), connection).await;
        let Ok(gallery_data) = get_gallery_data else{
            let error_resp = get_gallery_data.unwrap_err();
            return Err(error_resp);
        };

        if gallery_data.owner_screen_cid != caller_screen_cid{
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let user_collections = users_collections
            .order(created_at.desc())
            .filter(owner_screen_cid.eq(caller_screen_cid))
            .load::<UserCollection>(connection);
        
        
        let Ok(collections) = user_collections else{
            let resp = Response::<String>{
                data: Some(caller_screen_cid.to_string()),
                message: COLLECTION_NOT_FOUND_FOR,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };
        
        // get all collections related to the passed in gallery using redis
        // there is a mapping between gallery id and collections ids has inside
        let emppty_vec_of_ids: Vec<i32> = vec![];
        let mut redis_conn = redis_client.get_async_connection().await.unwrap();
        let get_all_collections_of_gal: String = redis_conn.get(gallery_data.id).await.unwrap_or(
            serde_json::to_string_pretty(&emppty_vec_of_ids).unwrap()
        );
        let collection_ids = serde_json::from_str::<Vec<i32>>(&get_all_collections_of_gal).unwrap();
        let mut gal_collections = vec![];
        if !collection_ids.is_empty(){
            for col_id in collection_ids{
                gal_collections.push(
                    UserCollection::find_by_id(col_id, connection).await.unwrap_or_default()
                )
            }
        }

        let mut cols = vec![];
        for c in gal_collections{
            if c.id == 0{
                continue;
            } else{
                cols.push(
                    UserCollectionData{
                        id: c.id,
                        contract_address: c.clone().contract_address,
                        nfts: {
                            /* return those none minted ones */
                            if c.nfts.is_some(){
                                
                                let get_nfts = UserNft::get_all_inside_contract_none_async(&c.contract_address, connection);
                                let nfts_ = if get_nfts.is_ok(){
                                    get_nfts.unwrap()
                                } else{
                                    vec![]
                                };

                                let mut none_minted_nfts = nfts_
                                    .into_iter()
                                    .map(|nft|{
                                        /* if we couldn't unwrap the is_minted means it's not minted yet and it's false */
                                        if nft.is_minted.unwrap_or(false) == false{
                                            Some(nft)
                                        } else{
                                            None
                                        }
                                    }).collect::<Vec<Option<UserNftData>>>();

                                none_minted_nfts.retain(|nft| nft.is_some());

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
                )
            }
        }
            

        let sliced = if from < cols.len(){
            if cols.len() > to{
                let data = &cols[from..to+1];
                data.to_vec()
            } else{
                let data = &cols[from..cols.len()];
                data.to_vec()
            }
        } else{
            vec![]
        };


        Ok(
            sliced
        )

    }

    pub async fn get_all_private_collections_for_invited_friends(caller_screen_cid: &str, owner_screen_cid_: &str, 
        gal_id: i32, limit: web::Query<Limit>, redis_client: RedisClient, connection: &mut DbPoolConnection) 
        -> Result<Vec<UserCollectionData>, PanelHttpResponse>{
            
        let get_gallery_data = UserPrivateGallery::find_by_id(gal_id, redis_client.clone(), connection).await;
        let Ok(gallery_data) = get_gallery_data else{
            let error_resp = get_gallery_data.unwrap_err();
            return Err(error_resp);
        };

        if gallery_data.owner_screen_cid != owner_screen_cid_{
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        let inv_frds = gallery_data.invited_friends;
        if inv_frds.is_none(){
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_HAS_NO_INVITED_FRIENDS_YET,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let friends_ = inv_frds.unwrap();
        if !friends_.contains(&Some(caller_screen_cid.to_string())){
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: CALLER_CANT_VIEW_GALLERY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let user_collections = users_collections
            .order(created_at.desc())
            .filter(owner_screen_cid.eq(owner_screen_cid_))
            .load::<UserCollection>(connection);
        
        
        let Ok(collections) = user_collections else{
            let resp = Response::<String>{
                data: Some(caller_screen_cid.to_string()),
                message: COLLECTION_NOT_FOUND_FOR,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };
        
        let mut cols =
            
            collections
                .into_iter()
                .map(|c|{

                    UserCollectionData{
                        id: c.id,
                        contract_address: c.clone().contract_address,
                        nfts: {
                            /* return those none minted ones */
                            if c.nfts.is_some(){
                                
                                let get_nfts = UserNft::get_all_inside_contract_none_async(&c.contract_address, connection);
                                let nfts_ = if get_nfts.is_ok(){
                                    get_nfts.unwrap()
                                } else{
                                    vec![]
                                };
                                
                                let mut none_minted_nfts = nfts_
                                    .into_iter()
                                    .map(|nft|{
                                        /* if we couldn't unwrap the is_minted means it's not minted yet and it's false */
                                        if nft.is_minted.unwrap_or(false) == false{
                                            Some(nft)
                                        } else{
                                            None
                                        }
                                    }).collect::<Vec<Option<UserNftData>>>();
                                
                                none_minted_nfts.retain(|nft| nft.is_some());
                                
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
                .collect::<Vec<UserCollectionData>>();
        

        cols.sort_by(|col1, col2|{

            let col1_created_at = NaiveDateTime
                ::parse_from_str(&col1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

            let col2_created_at = NaiveDateTime
                ::parse_from_str(&col2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

            col2_created_at.cmp(&col1_created_at)

        });
            

        let sliced = if from < cols.len(){
            if cols.len() > to{
                let data = &cols[from..to+1];
                data.to_vec()
            } else{
                let data = &cols[from..cols.len()];
                data.to_vec()
            }
        } else{
            vec![]
        };


        Ok(
            sliced
        )

    }

    pub async fn upload_collection_img(
        col_id: i32,
        caller_screen_cid: &str,
        mut img: Multipart, 
        redis_actor: Addr<RedisActor>,
        redis_client: RedisClient,
        connection: &mut DbPoolConnection) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        let get_collection = Self::find_by_id(col_id, connection).await;
        let Ok(collection_data) = get_collection else{
            
            let err_resp = get_collection.unwrap_err();
            return Err(err_resp);
        };


        let get_user = User::find_by_screen_cid(caller_screen_cid, connection).await;
        let Ok(user) = get_user else{
            
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };


        /* caller must be the collection and gallery owner */
        if caller_screen_cid.to_string() != collection_data.owner_screen_cid{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: COLLECTION_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
                
        }

        /* getting gallery data */        
        let get_gallery_data = UserPrivateGallery::find_by_owner_and_contract_address(&caller_screen_cid, &collection_data.contract_address, redis_client.clone(), connection).await;
        let Ok(gallery_data) = get_gallery_data else{

            let err_resp = get_gallery_data.unwrap_err();
            return Err(err_resp);
        };

        /* caller must be the gallery owner */
        if gallery_data.owner_screen_cid != collection_data.owner_screen_cid{
    
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        /* uploading collection image */
        let img = std::sync::Arc::new(tokio::sync::Mutex::new(img));
        let get_collection_img_path = multipartreq::store_file(
            COLLECTION_UPLOAD_PATH, &collection_data.owner_screen_cid, 
            "collection", 
            img).await;
        let Ok(collection_img_path) = get_collection_img_path else{

            let err_res = get_collection_img_path.unwrap_err();
            return Err(err_res);
        };

        /* if the onchain data was ok we simply update the record based on the data updated onchain */
        let new_col_data = UpdateUserCollection{
            nfts: collection_data.clone().nfts,
            base_uri: collection_data.clone().base_uri,
            royalties_share: collection_data.clone().royalties_share,
            royalties_address_screen_cid: collection_data.clone().royalties_address_screen_cid,
            collection_background: if collection_img_path.is_empty(){
                collection_data.clone().collection_background
            } else{
                collection_img_path
            },
            extra: collection_data.clone().extra,
            col_description: collection_data.clone().col_description,
            freeze_metadata: collection_data.clone().freeze_metadata,
            contract_tx_hash: collection_data.clone().contract_tx_hash.unwrap_or(String::from("")),
            col_name: collection_data.clone().col_name,
        };
    
        match diesel::update(users_collections.filter(users_collections::id.eq(collection_data.id)))
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

                    Ok(user_collection_data)

                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserCollection::upload_collection_img");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }
        

    }

    pub async fn get_all_public_collections_for(screen_cid: &str, limit: web::Query<Limit>, caller_screen_cid: &str,
        connection: &mut DbPoolConnection) 
        -> Result<Vec<Option<UserCollectionData>>, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }


            let user_collections = users_collections
                .order(created_at.desc())
                .filter(owner_screen_cid.eq(screen_cid))
                .load::<UserCollection>(connection);
        
            let Ok(collections_) = user_collections else{
                let resp = Response{
                    data: Some(screen_cid),
                    message: COLLECTION_NOT_FOUND_FOR,
                    status: 404,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                )
            };
                

            let mut minted_cols = vec![];
            for col in collections_{

                let nfts_ = {
                    let get_minted_nfts_of_this_collection = Self::get_all_pure_minted_nfts_of_collection_with_address(&col.contract_address, connection);
                    if get_minted_nfts_of_this_collection.is_ok(){
                        let mut minted_nfts_of_this_collection = get_minted_nfts_of_this_collection.unwrap();
                        if minted_nfts_of_this_collection.len() != 0{
                            minted_nfts_of_this_collection.retain(|nft| nft.is_some());
                            Some(
                                serde_json::to_value(minted_nfts_of_this_collection).unwrap()
                            )
                        } else{
                            // ignore pushing the collection into the vector, regardless of everything 
                            // cause if the user has no minted nfts yet means that none of his nfts are
                            // public and still his collection is inside the private gallery so no one  
                            // must be able to see his collection data and info 
                            continue;
                        }
                    } else{
                        Some(
                            serde_json::to_value::<Vec<UserNftData>>(vec![]).unwrap()
                        )
                    }
                };

                minted_cols.push(
                    Some(
                        UserCollectionData{
                            id: col.id,
                            contract_address: col.contract_address,
                            /* get all minted nfts for this collection */
                            nfts: nfts_,
                            col_name: col.col_name,
                            symbol: col.symbol,
                            owner_screen_cid: col.owner_screen_cid,
                            metadata_updatable: col.metadata_updatable,
                            base_uri: col.base_uri,
                            royalties_share: col.royalties_share,
                            royalties_address_screen_cid: col.royalties_address_screen_cid,
                            collection_background: col.collection_background,
                            extra: col.extra,
                            col_description: col.col_description,
                            created_at: col.created_at.to_string(),
                            updated_at: col.updated_at.to_string(),
                            freeze_metadata: col.freeze_metadata,
                            contract_tx_hash: col.contract_tx_hash,
                        }
                    )
                )
            }

        minted_cols.retain(|c| c.is_some());            
        
        minted_cols.sort_by(|col1, col2|{

            let col1_created_at = NaiveDateTime
                ::parse_from_str(col1.clone().unwrap().created_at.as_str(), "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

            let col2_created_at = NaiveDateTime
                ::parse_from_str(col2.clone().unwrap().created_at.as_str(), "%Y-%m-%d %H:%M:%S%.f")
                .unwrap();

            col2_created_at.cmp(&col1_created_at)

        });      
        
        let sliced = if from < minted_cols.len(){
            if minted_cols.len() > to{
                let data = &minted_cols[from..to+1];
                data.to_vec()
            } else{
                let data = &minted_cols[from..minted_cols.len()];
                data.to_vec()
            }
        } else{
            vec![]
        };


        Ok(
            sliced
        )

    }

}

impl UserCollection{

    /*  ------------- nftport request body -------------
    create new nft product contract collection
    https://docs.nftport.xyz/docs/contract-comparison
        onchian updates:
            - name
            - symbol
            - owner_address
            - metadata_updatable
            - royalties_share
            - royalties_address
            - base_uri
    */
    pub async fn insert(new_col_info: NewUserCollectionRequest, redis_actor: Addr<RedisActor>,
        redis_client: redis::Client, connection: &mut DbPoolConnection) 
        -> Result<UserCollectionData, PanelHttpResponse>{

        if new_col_info.royalties_share > 50000{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: COLLECTION_ROYALTY_IS_EXCEEDED,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

        let caller_screen_cid = walletreq::evm::get_keccak256_from(new_col_info.clone().owner_cid);
        /* caller must be in db */
        let Ok(user) = User::find_by_screen_cid(
            &caller_screen_cid, connection).await 
            else{
                let resp = Response{
                    data: Some(new_col_info.owner_cid),
                    message: USER_SCREEN_CID_NOT_FOUND,
                    status: 404,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
        };

        /* caller must be the gallery owner */
        let get_gallery_data = UserPrivateGallery::find_by_id(new_col_info.clone().gallery_id, redis_client.clone(), connection).await;
        let Ok(gallery_data) = get_gallery_data else{

            let err_resp = get_gallery_data.unwrap_err();
            return Err(err_resp);
        };

        if gallery_data.owner_screen_cid != caller_screen_cid{
    
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        /* 
            update user balance frist, if anything goes wrong they can call us 
            to pay them back, actually this is the gas fee that they must be 
            charged for since we already have paid the fee when we created 
            the contract collection
        */
        let new_balance = user.balance.unwrap() - new_col_info.amount;
        let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actor.clone(), connection).await;
        let Ok(updated_user_balance_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };
        
        /* getting onchain contract information */
        let (contract_onchain_address, contract_create_tx_hash, status) = nftport::create_collection(redis_client.clone(), new_col_info.clone()).await;
        
        if status == 1 && contract_onchain_address == String::from("") && 
            contract_create_tx_hash == String::from(""){
            
            // if anything goes wrong payback the user
            let new_balance = updated_user_balance_data.balance.unwrap() + new_col_info.amount;
            let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actor, connection).await;
            let Ok(updated_user_data) = update_user_balance else{
    
                let err_resp = update_user_balance.unwrap_err();
                return Err(err_resp);
                
            };

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: CANT_CREATE_COLLECTION_ONCHAIN,
                status: 417,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        }

        if !contract_create_tx_hash.starts_with("0x"){

            // if anything goes wrong payback the user
            let new_balance = updated_user_balance_data.balance.unwrap() + new_col_info.amount;
            let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actor, connection).await;
            let Ok(updated_user_data) = update_user_balance else{
    
                let err_resp = update_user_balance.unwrap_err();
                return Err(err_resp);
                
            };

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: INVALID_CONTRACT_TX_HASH,
                status: 417,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        } 

        if status == 1 && contract_onchain_address == String::from("") && 
            contract_create_tx_hash.starts_with("0x"){

            // if anything goes wrong payback the user
            let new_balance = updated_user_balance_data.balance.unwrap() + new_col_info.amount;
            let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actor, connection).await;
            let Ok(updated_user_data) = update_user_balance else{
    
                let err_resp = update_user_balance.unwrap_err();
                return Err(err_resp);
                
            };

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: CANT_GET_CONTRACT_ADDRESS,
                status: 417,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        }  
        
        // use random collections
        let random_collection = COLLECTIONS.choose(&mut rand::thread_rng());
    
        let new_col_data = InsertNewUserCollectionRequest{
            col_name: new_col_info.clone().col_name,
            symbol: new_col_info.clone().symbol,
            // contract_address: random_collection.unwrap_or(&"").to_string(), /* NEW */
            contract_address: contract_onchain_address, /* NEW */
            owner_screen_cid: walletreq::evm::get_keccak256_from(new_col_info.clone().owner_cid),
            metadata_updatable: new_col_info.clone().metadata_updatable,
            /* 
                remember to use a valid url if u want to fill this field since nftport 
                wille append the base_uri to eah nft metadata_uri address like so:
                https://onions.ioipfs://bafkreifvsdjrvezjfg67fcy6bwhjbjoxh6uxnm6ob4nm6z22tci6fe6sge
                so it's better to pass this empty
            */
            base_uri: new_col_info.clone().base_uri,
            royalties_share: new_col_info.clone().royalties_share,
            royalties_address_screen_cid: new_col_info.clone().royalties_address_screen_cid,
            extra: new_col_info.clone().extra,
            col_description: new_col_info.clone().col_description,
            contract_tx_hash: contract_create_tx_hash,
            collection_background: String::from(""), /* will be updated later */
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

                    // ------------------------------------------------
                    // get collection ids from redis for this gallery 
                    // ------------------------------------------------
                    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
                    let empty_vec_of_ids: Vec<i32> = vec![];
                    let get_gal_collections: String = redis_conn.get(new_col_info.gallery_id).await.unwrap_or(
                        serde_json::to_string_pretty(&empty_vec_of_ids).unwrap()
                    );
                    let mut gal_collections = serde_json::from_str::<Vec<i32>>(&get_gal_collections).unwrap();
                    // ----------------------------------------------------
                    // store this collection id into redis for this gallery 
                    // ----------------------------------------------------
                    gal_collections.push(user_collection_data.id);
                    let ـ : RedisResult<String> = redis_conn.set(new_col_info.gallery_id, 
                        serde_json::to_string(&gal_collections).unwrap()
                    ).await;


                    /** -------------------------------------------------------------------- */
                    /** ----------------- publish new event data to `on_user_action` channel */
                    /** -------------------------------------------------------------------- */
                    // if the actioner is the user himself we'll notify user with something like:
                    // u've just done that action!
                    let actioner_wallet_info = UserWalletInfoResponse{
                        username: user.clone().username,
                        avatar: user.clone().avatar,
                        bio: user.clone().bio,
                        banner: user.clone().banner,
                        mail: user.clone().mail,
                        screen_cid: user.clone().screen_cid,
                        extra: user.clone().extra,
                        stars: user.clone().stars,
                        created_at: user.clone().created_at.to_string(),
                    };
                    let user_wallet_info = UserWalletInfoResponse{
                        username: user.clone().username,
                        avatar: user.clone().avatar,
                        bio: user.clone().bio,
                        banner: user.clone().banner,
                        mail: user.clone().mail,
                        screen_cid: user.clone().screen_cid,
                        extra: user.clone().extra,
                        stars: user.clone().stars,
                        created_at: user.clone().created_at.to_string(),
                    };
                    let user_notif_info = SingleUserNotif{
                        wallet_info: user_wallet_info,
                        notif: NotifData{
                            actioner_wallet_info,
                            fired_at: Some(chrono::Local::now().timestamp()),
                            action_type: ActionType::CreateCollection,
                            action_data: serde_json::to_value(user_collection_data.clone()).unwrap()
                        }
                    };
                    let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                    events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

                    Ok(user_collection_data)
                    
                },
                Err(e) => {

                    let new_balance = updated_user_balance_data.balance.unwrap() + new_col_info.amount;
                    let update_user_balance = User::update_balance(user.id, new_balance, redis_client.to_owned(), redis_actor, connection).await;
                    let Ok(updated_user_data) = update_user_balance else{
            
                        let err_resp = update_user_balance.unwrap_err();
                        return Err(err_resp);
                        
                    };

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserCollection::insert");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= COLLECTION OWNER -=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /*  ------------- nftport request body -------------
        onchian updates:
            - freeze_metadata
            - owner_address
            - royalties_share
            - royalties_address
            - base_uri
    */
    pub async fn update(mut col_info: UpdateUserCollectionRequest, redis_actor: Addr<RedisActor>,
        redis_client: redis::Client, connection: &mut DbPoolConnection) 
        -> Result<UserCollectionData, PanelHttpResponse>{
        
        if col_info.royalties_share > 50000{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: COLLECTION_ROYALTY_IS_EXCEEDED,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

        let collection_owner_screen_cid = walletreq::evm::get_keccak256_from(col_info.clone().owner_cid);
        
        /* caller must be in db */
        let Ok(user) = User::find_by_screen_cid(
            &collection_owner_screen_cid, connection).await 
            else{
                let resp = Response{
                    data: Some(col_info.owner_cid),
                    message: USER_SCREEN_CID_NOT_FOUND,
                    status: 404,
                    is_error: true
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

        /* caller must be the collection owner, we've got the caller screen_cid from the JWT id */
        if collection_owner_screen_cid != collection_data.owner_screen_cid{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: COLLECTION_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
               
        }

        /* getting gallery data */        
        let get_gallery_data = UserPrivateGallery::find_by_id(col_info.gallery_id, redis_client.clone(), connection).await;
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
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )
        }

        // base_uri depends on freeze_metadata and metadata_updatable fields
        col_info.base_uri = if collection_data.freeze_metadata.is_some() && 
            collection_data.freeze_metadata.unwrap() == true &&
            collection_data.metadata_updatable.is_some() && 
            collection_data.metadata_updatable.unwrap() != true{
            
            /* 
                just ignore the base_uri, can't update base_uri since contract is frozen 
                and metadata_updatable is false
            */
            String::from("") 

        } else{
            /* 
                contract is not frozen and metadata_updatable is true 
                hence we can update base_uri 
            */
            col_info.base_uri 
        };

        /* updating onchain contract information */
        let (contract_update_tx_hash, status) = nftport::update_collection(
            redis_client.clone(), 
            col_info.clone(), 
            collection_data.contract_address.clone()).await;
        
        if status == 1 && contract_update_tx_hash == String::from(""){

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: CANT_UPDATE_COLLECTION_ONCHAIN,
                status: 417,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        }  

        if status == 2 && contract_update_tx_hash == String::from(""){

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: CANT_UPDATE_FROZEN_COLLECTION_ONCHAIN,
                status: 417,
                is_error: true
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
                is_error: true
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            )
        }


        /* if the onchain data was ok we simply update the record based on the data updated onchain */
        let new_col_data = UpdateUserCollection{
            nfts: col_info.clone().nfts,
            base_uri: col_info.clone().base_uri,
            royalties_share: col_info.clone().royalties_share,
            royalties_address_screen_cid: col_info.clone().royalties_address_screen_cid,
            collection_background: collection_data.clone().collection_background,
            extra: col_info.clone().extra,
            col_description: col_info.clone().col_description,
            freeze_metadata: Some(col_info.clone().freeze_metadata),
            contract_tx_hash: collection_data.clone().contract_tx_hash.unwrap_or(String::from("")),
            col_name: col_info.clone().col_name,
        };
    
        match diesel::update(users_collections.filter(users_collections::id.eq(collection_data.id)))
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
           
                    /** -------------------------------------------------------------------- */
                    /** ----------------- publish new event data to `on_user_action` channel */
                    /** -------------------------------------------------------------------- */
                    // if the actioner is the user himself we'll notify user with something like:
                    // u've just done that action!
                    let actioner_wallet_info = UserWalletInfoResponse{
                        username: user.clone().username,
                        avatar: user.clone().avatar,
                        bio: user.clone().bio,
                        banner: user.clone().banner,
                        mail: user.clone().mail,
                        screen_cid: user.clone().screen_cid,
                        extra: user.clone().extra,
                        stars: user.clone().stars,
                        created_at: user.clone().created_at.to_string(),
                    };
                    let user_wallet_info = UserWalletInfoResponse{
                        username: user.clone().username,
                        avatar: user.clone().avatar,
                        bio: user.clone().bio,
                        banner: user.clone().banner,
                        mail: user.clone().mail,
                        screen_cid: user.clone().screen_cid,
                        extra: user.clone().extra,
                        stars: user.clone().stars,
                        created_at: user.clone().created_at.to_string(),
                    };
                    let user_notif_info = SingleUserNotif{
                        wallet_info: user_wallet_info,
                        notif: NotifData{
                            actioner_wallet_info,
                            fired_at: Some(chrono::Local::now().timestamp()),
                            action_type: ActionType::UpdateCollection,
                            action_data: serde_json::to_value(user_collection_data.clone()).unwrap()
                        }
                    };
                    let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                    events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;
                    
                    Ok(user_collection_data)


                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserCollection::update");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

}