


use crate::models::users_nfts::{UserNftDataWithWalletInfo, UserNftDataWithWalletInfoAndCollectionData};

pub use super::*;


#[get("/nft/get/collections/for/{col_owner}")]
pub(self) async fn get_nft_product_collections(
        req: HttpRequest,
        col_owner: web::Path<String>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            match UserCollection::get_all_nft_product_collections_by_owner(&col_owner.to_owned(), connection).await{

                Ok(collection_info) => {

                    resp!{
                        Vec<CollectionInfoResponse>, // the data type
                        collection_info, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
                    }

                },
                Err(resp) => {
                    resp
                }
            }
            
        
        }, 
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }         


}



#[get("/get-top-nfts/")]
pub(self) async fn get_top_nfts(
        req: HttpRequest,   
        limit: web::Query<Limit>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            let from = limit.from.unwrap_or(0) as usize;
            let to = limit.to.unwrap_or(10) as usize;

            if to < from {
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_QUERY_LIMIT,
                    status: 406,
                    is_error: true
                };
                return Ok(HttpResponse::NotAcceptable().json(resp));
                
            }

            let get_nfts = UserNft::get_all(connection).await;
            let Ok(nfts) = get_nfts else{
                let err_resp = get_nfts.unwrap_err();
                return err_resp;
            };

            let mut nft_like_map = vec![];
            for nft in nfts{

                if nft.is_minted.unwrap_or(false) == false{
                    continue;
                }
                
                let nft_likes = nft.likes;
                let mut decoded_likes = if nft_likes.is_some(){
                    serde_json::from_value::<Vec<NftLike>>(nft_likes.unwrap()).unwrap()
                } else{
                    vec![]
                };  
                
                for like in decoded_likes{
                    nft_like_map.push(
                        NftUpvoterLikes{
                            id: nft.id,
                            upvoter_screen_cids: like.upvoter_screen_cids.len() as u64
                        }
                    );
                }

            }
            
            // sort by the most likes to less ones
            nft_like_map.sort_by(|nl1, nl2|{

                let nl1_likes = nl1.upvoter_screen_cids;
                let nl2_likes = nl2.upvoter_screen_cids;

                nl2_likes.cmp(&nl1_likes)

            });
            
            let top_nfts = nft_like_map
                .into_iter()
                .map(|nlinfo|{

                    let nft = UserNft::find_by_id_none_async(nlinfo.id, connection).unwrap();
                    NftColInfo{
                        col_data: {
                            let col_info = UserCollection::find_by_contract_address_none_async(&nft.contract_address, connection).unwrap();
                            UserCollectionDataGeneralInfo{
                                id: col_info.id,
                                contract_address: col_info.contract_address,
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
                        },
                        nfts_data: nft,
                    }

                })
                .collect::<Vec<NftColInfo>>();


            let sliced = if from < top_nfts.len(){
                if top_nfts.len() > to{
                    let data = &top_nfts[from..to+1];
                    data.to_vec()
                } else{
                    let data = &top_nfts[from..top_nfts.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };
            
            resp!{
                Vec<NftColInfo>, // the data type
                sliced, // response data
                FETCHED, // response message
                StatusCode::OK, // status code
                None::<Cookie<'_>>, // cookie
            }
        
        }, 
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }         


}

#[get("/get-all-minted-nfts/")]
pub(self) async fn get_all_nfts(
        req: HttpRequest,   
        limit: web::Query<Limit>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            let from = limit.from.unwrap_or(0) as usize;
            let to = limit.to.unwrap_or(10) as usize;

            if to < from {
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_QUERY_LIMIT,
                    status: 406,
                    is_error: true
                };
                return Ok(HttpResponse::NotAcceptable().json(resp));
                
            }

            let get_nfts = UserNft::get_all(connection).await;
            let Ok(nfts) = get_nfts else{
                let err_resp = get_nfts.unwrap_err();
                return err_resp;
            };

            let mut minted_ones = vec![];
            for nft in nfts{
                if nft.is_minted.is_some() && nft.is_minted.clone().unwrap(){
                    minted_ones.push(
                        NftColInfo{
                            col_data: {
                                let col_info = UserCollection::find_by_contract_address(&nft.contract_address, connection).await.unwrap();
                                UserCollectionDataGeneralInfo{
                                    id: col_info.id,
                                    contract_address: col_info.contract_address,
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
                            },
                            nfts_data: nft,
                        }
                    )
                }

            }
            
            // let mut rng = rand::thread_rng();
            // minted_ones.shuffle(&mut rng);

            minted_ones.sort_by(|nftcol1, nftcol2|{

                let nftcol1_created_at = NaiveDateTime
                    ::parse_from_str(&nftcol1.nfts_data.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                let nftcol2_created_at = NaiveDateTime
                    ::parse_from_str(&nftcol2.nfts_data.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                nftcol2_created_at.cmp(&nftcol1_created_at)

            });

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

            resp!{
                Vec<NftColInfo>, // the data type
                sliced, // response data
                FETCHED, // response message
                StatusCode::OK, // status code
                None::<Cookie<'_>>, // cookie
            }
        
        }, 
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }         


}

#[get("/nft/get/{asset_id}")]
pub(self) async fn get_single_nft(
    req: HttpRequest,  
    asset_id: web::Path<i32>,    
    app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse {


    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let async_redis_client = storage.as_ref().clone().unwrap().get_async_redis_pubsub_conn().await.unwrap();

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();

            match UserNft::find_by_id(asset_id.to_owned(), connection).await{
                        
                Ok(user_nft_data) => {
                    resp!{
                        UserNftDataWithWalletInfoAndCollectionData, // the data type
                        UserNftDataWithWalletInfoAndCollectionData{ 
                            id: user_nft_data.id, 
                            contract_address: user_nft_data.clone().contract_address, 
                            collection_data: {
                                let collection_data = UserCollection::find_by_contract_address_none_async(&user_nft_data.contract_address, connection).unwrap();
                                serde_json::json!({
                                    "id": collection_data.id,
                                    "contract_address": collection_data.contract_address,
                                    "col_name": collection_data.col_name,
                                    "symbol": collection_data.symbol,
                                    "owner_screen_cid": collection_data.owner_screen_cid,
                                    "metadata_updatable": collection_data.metadata_updatable,
                                    "freeze_metadata": collection_data.freeze_metadata,
                                    "base_uri": collection_data.base_uri,
                                    "royalties_share": collection_data.royalties_share,
                                    "royalties_address_screen_cid": collection_data.royalties_address_screen_cid,
                                    "collection_background": collection_data.collection_background,
                                    "extra": collection_data.extra, /* pg key, value based json binary object */
                                    "col_description": collection_data.col_description,
                                    "contract_tx_hash": collection_data.contract_tx_hash,
                                    "created_at": collection_data.created_at.to_string(),
                                    "updated_at": collection_data.updated_at.to_string(),
                                })
                            },
                            current_owner_wallet_info: {
                                let user = User::find_by_screen_cid(&user_nft_data.current_owner_screen_cid, connection).await.unwrap();
                                UserWalletInfoResponse{
                                    username: user.username,
                                    avatar: user.avatar,
                                    bio: user.bio,
                                    banner: user.banner,
                                    mail: user.mail,
                                    screen_cid: user.screen_cid,
                                    extra: user.extra,
                                    stars: user.stars,
                                    created_at: user.created_at.to_string(),
                                }
                            }, 
                            metadata_uri: user_nft_data.metadata_uri, 
                            extra: user_nft_data.extra, 
                            onchain_id: user_nft_data.onchain_id, 
                            nft_name: user_nft_data.nft_name, 
                            is_minted: user_nft_data.is_minted, 
                            nft_description: user_nft_data.nft_description, 
                            current_price: user_nft_data.current_price, 
                            is_listed: user_nft_data.is_listed, 
                            freeze_metadata: user_nft_data.freeze_metadata, 
                            comments: user_nft_data.comments, 
                            likes: user_nft_data.likes, 
                            tx_hash: user_nft_data.tx_hash, 
                            created_at: user_nft_data.created_at.to_string(), 
                            updated_at: user_nft_data.updated_at.to_string(),
                            attributes: user_nft_data.attributes, 
                        }, // response data
                        FETCHED, // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
                    }

                },
                Err(resp) => {
                    resp
                }
            }

        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }


}

#[get("/collection/get/{col_id}")]
pub(self) async fn get_public_collection(
        req: HttpRequest,   
        col_id: web::Path<i32>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            let get_col_info = UserCollection::find_by_id(col_id.to_owned(), connection).await;
            let Ok(mut collection) = get_col_info else{
                let err_resp = get_col_info.unwrap_err();
                return err_resp;
            };

            let get_minted_nfts_of_collection = UserCollection
                ::get_all_pure_minted_nfts_of_collection_with_address(&collection.contract_address, connection);
            
            let Ok(mut minted_nfts) = get_minted_nfts_of_collection else{
                let err_resp = get_minted_nfts_of_collection.unwrap_err();
                return err_resp;
            };

            collection.nfts = Some(
                serde_json::to_value(&minted_nfts).unwrap()
            );

            resp!{
                UserCollectionData, // the data type
                collection, // response data
                FETCHED, // response message
                StatusCode::OK, // status code
                None::<Cookie<'_>>, // cookie
            }
        
        }, 
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }         


}


pub mod exports{
    pub use super::get_top_nfts;
    pub use super::get_all_nfts;
    pub use super::get_public_collection;
    pub use super::get_single_nft;
    pub use super::get_nft_product_collections;
}