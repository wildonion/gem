


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


pub mod exports{
    pub use super::get_top_nfts;
    pub use super::get_all_nfts;
    pub use super::get_nft_product_collections;
}