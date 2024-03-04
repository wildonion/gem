


pub use super::*;


#[get("/search/")]
pub(self) async fn search(
        req: HttpRequest,   
        query: web::Query<Search>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

            let from = query.from.unwrap_or(0) as usize;
            let to = query.to.unwrap_or(10) as usize;

            if to < from {
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_QUERY_LIMIT,
                    status: 406,
                    is_error: true
                };
                return Ok(HttpResponse::NotAcceptable().json(resp));
                
            }

            /* search in users */
            let query_to_seatch = format!("%{}%", query.q);
            let get_users_info = users::table
                .filter(
                    users::username.ilike(query_to_seatch.as_str())
                        .or(
                            users::screen_cid.ilike(query_to_seatch.as_str())
                        )
                        .or(
                            users::mail.ilike(query_to_seatch.as_str())
                        )
                        .or(
                            users::phone_number.ilike(query_to_seatch.as_str())
                        )
                )
                .order(users::created_at.desc())
                .load::<User>(connection);

            let Ok(users_info) = get_users_info else{
                let err = get_users_info.unwrap_err();
                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: &err.to_string(),
                    status: 500,
                    is_error: true
                };
                return 
                    Ok(HttpResponse::InternalServerError().json(resp))
                
            };

            let users_info = 
                users_info
                    .into_iter()
                    .map(|u|{
                        UserData { 
                            id: u.id, 
                            region: u.region.clone(),
                            username: u.clone().username,
                            bio: u.bio.clone(),
                            avatar: u.avatar.clone(),
                            banner: u.banner.clone(), 
                            wallet_background: u.wallet_background.clone(), 
                            activity_code: u.clone().activity_code, 
                            twitter_username: u.clone().twitter_username, 
                            facebook_username: u.clone().facebook_username, 
                            discord_username: u.clone().discord_username, 
                            identifier: u.clone().identifier, 
                            user_role: {
                                match u.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: u.token_time,
                            balance: u.balance,
                            last_login: { 
                                if u.last_login.is_some(){
                                    Some(u.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: u.created_at.to_string(),
                            updated_at: u.updated_at.to_string(),
                            mail: u.clone().mail,
                            google_id: u.clone().google_id,
                            microsoft_id: u.clone().microsoft_id,
                            is_mail_verified: u.is_mail_verified,
                            is_phone_verified: u.is_phone_verified,
                            phone_number: u.clone().phone_number,
                            paypal_id: u.clone().paypal_id,
                            account_number: u.clone().account_number,
                            device_id: u.clone().device_id,
                            social_id: u.clone().social_id,
                            cid: u.clone().cid,
                            screen_cid: u.clone().screen_cid,
                            snowflake_id: u.snowflake_id,
                            stars: u.stars,
                            extra: u.clone().extra,
                        }
                    })
                    .collect::<Vec<UserData>>();
                         
            /* search in galleries, collections and nfts */
            let get_user_galleries_info = users_galleries
                .load::<UserPrivateGallery>(connection);

            let Ok(galleries_info) = get_user_galleries_info else{
                let err = get_user_galleries_info.unwrap_err();
                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: &err.to_string(),
                    status: 500,
                    is_error: true
                };
                return 
                    Ok(HttpResponse::InternalServerError().json(resp))
                
            };

            let mut galleries_info = 
                galleries_info
                    .into_iter()
                    .map(|g|{

                        UserPrivateGalleryData{ 
                            id: g.id, 
                            owner_screen_cid: g.owner_screen_cid, 
                            collections: g.collections, 
                            gal_name: g.gal_name, 
                            gal_description: g.gal_description, 
                            invited_friends: g.invited_friends, 
                            extra: g.extra, 
                            gallery_background: g.gallery_background,
                            created_at: g.created_at.to_string(), 
                            updated_at: g.updated_at.to_string() 
                        }

                    })
                    .collect::<Vec<UserPrivateGalleryData>>();

            /* order based on newest ones */
            galleries_info.sort_by(|g1, g2|{

                let g1_created_at = NaiveDateTime
                    ::parse_from_str(&g1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                let g2_created_at = NaiveDateTime
                    ::parse_from_str(&g2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                g2_created_at.cmp(&g1_created_at)

            });

            let mut found_collections = vec![];
            let mut found_nfts = vec![];
            for gallery in galleries_info{

                let cols = gallery.collections;
                let decoded_cols = if cols.is_some(){
                    serde_json::from_value::<Vec<UserCollectionData>>(cols.unwrap()).unwrap()
                } else{
                    vec![]
                };

                let match_collections = decoded_cols.clone()
                    .into_iter()
                    .map(|col| {

                        if col.col_name.contains(&query.q) ||
                            col.col_description.contains(&query.q) ||
                            col.owner_screen_cid.contains(&query.q) || 
                            col.contract_address.contains(&query.q) || 
                            col.contract_tx_hash.clone().unwrap_or(String::from("")).contains(&query.q)
                            {
                                /* -----------------------------------------------------------------
                                    > in those case that we don't want to create a separate struct 
                                    and allocate an instance of it to map a utf8 bytes data coming
                                    from a server or client into its feilds we can use serde_json::to_value()
                                    which maps an instance of a structure into a serde json value 
                                    or serde_json::json!({}) to create a json value from those fields 
                                    that we want to return them, but if we want to mutate data in rust we 
                                    have to convert the json value or received bytes into the structure, 
                                */
                                Some(
                                    serde_json::json!({
                                        "id": col.id,
                                        "contract_address": col.contract_address,
                                        "col_name": col.col_name,
                                        "symbol": col.symbol,
                                        "owner_screen_cid": col.owner_screen_cid,
                                        "metadata_updatable": col.metadata_updatable,
                                        "freeze_metadata": col.freeze_metadata,
                                        "base_uri": col.base_uri,
                                        "royalties_share": col.royalties_share,
                                        "royalties_address_screen_cid": col.royalties_address_screen_cid,
                                        "collection_background": col.collection_background,
                                        "extra": col.extra,
                                        "col_description": col.col_description,
                                        "contract_tx_hash": col.contract_tx_hash,
                                        "created_at": col.created_at,
                                        "updated_at": col.updated_at,
                                    })
                                )
                        } else{
                            None
                        }
                    })
                    .collect::<Vec<Option<serde_json::Value>>>();
                
                found_collections.extend(match_collections);
                found_collections.retain(|col| col.is_some());

                for collection in decoded_cols{
                    
                    let colnfts = collection.clone().nfts;
                    let decoded_nfts = if colnfts.is_some(){
                        serde_json::from_value::<Vec<UserNftData>>(colnfts.unwrap()).unwrap()
                    } else{
                        vec![]
                    };

                    let match_nfts = decoded_nfts
                        .into_iter()
                        .map(|nft| {
                            if nft.is_minted.is_some() && nft.is_minted.unwrap() == true && 
                            (
                                nft.nft_name.contains(&query.q) ||
                                nft.nft_description.contains(&query.q) ||
                                nft.current_owner_screen_cid.contains(&query.q) ||
                                nft.contract_address.contains(&query.q) ||
                                nft.onchain_id.clone().unwrap().contains(&query.q) ||
                                nft.tx_hash.clone().unwrap().contains(&query.q)
                            ){
                                Some(
                                    NftColInfo{ 
                                        col_data: UserCollectionDataGeneralInfo{
                                            id: collection.id,
                                            contract_address: collection.clone().contract_address,
                                            col_name: collection.clone().col_name,
                                            symbol: collection.clone().symbol,
                                            owner_screen_cid: collection.clone().owner_screen_cid,
                                            metadata_updatable: collection.clone().metadata_updatable,
                                            freeze_metadata: collection.clone().freeze_metadata,
                                            base_uri: collection.clone().base_uri,
                                            royalties_share: collection.clone().royalties_share,
                                            royalties_address_screen_cid: collection.clone().royalties_address_screen_cid,
                                            collection_background: collection.clone().collection_background,
                                            extra: collection.clone().extra,
                                            col_description: collection.clone().col_description,
                                            contract_tx_hash: collection.clone().contract_tx_hash,
                                            created_at: collection.created_at.to_string(),
                                            updated_at: collection.updated_at.to_string(),
                                        }, 
                                        nfts_data: nft 
                                    }
                                )
                            } else{
                                None
                            }
                        })
                        .collect::<Vec<Option<NftColInfo>>>();
                    

                    found_nfts.extend(match_nfts);
                    found_nfts.retain(|nft| nft.is_some());

                }

            }

            /* order based on newest ones */
            found_nfts.sort_by(|n1, n2|{

                /* 
                    cannot move out of `*n1` which is behind a shared reference
                    move occurs because `*n1` has type `std::option::Option<NftColInfo>`, 
                    which does not implement the `Copy` trait and unwrap() takes the 
                    ownership of the instance.
                    also we must create a longer lifetime for `NftColInfo::default()` by 
                    putting it inside a type so we can take a reference to it and pass the 
                    reference to the `unwrap_or()`, cause &NftColInfo::default() will be dropped 
                    at the end of the `unwrap_or()` statement while we're borrowing it.
                */
                let n1_default = NftColInfo::default();
                let n2_default = NftColInfo::default();
                let n1 = n1.as_ref().unwrap_or(&n1_default);
                let n2 = n2.as_ref().unwrap_or(&n2_default);

                let n1_created_at = NaiveDateTime
                    ::parse_from_str(&n1.nfts_data.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                let n2_created_at = NaiveDateTime
                    ::parse_from_str(&n2.nfts_data.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                n2_created_at.cmp(&n1_created_at)

            });

            /* order based on newest ones */
            found_collections.sort_by(|c1, c2|{

                /* 
                    cannot move out of `*c1` which is behind a shared reference
                    move occurs because `*c1` has type `std::option::Option<UserCollectionData>`, 
                    which does not implement the `Copy` trait and unwrap() takes the 
                    ownership of the instance.
                    also we must create a longer lifetime for `UserCollectionData::default()` by 
                    putting it inside a type so we can take a reference to it and pass the 
                    reference to the `unwrap_or()`, cause &UserCollectionData::default() will be dropped 
                    at the end of the `unwrap_or()` statement while we're borrowing it.
                */
                let c1_default = UserCollectionData::default();
                let c2_default = UserCollectionData::default();
                let c1 = serde_json::from_value::<UserCollectionData>(c1.clone().unwrap()).unwrap_or(c1_default);
                let c2 = serde_json::from_value::<UserCollectionData>(c2.clone().unwrap()).unwrap_or(c2_default);

                let c1_created_at = NaiveDateTime
                    ::parse_from_str(&c1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                let c2_created_at = NaiveDateTime
                    ::parse_from_str(&c2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();

                c2_created_at.cmp(&c1_created_at)

            });

            /*  
                first we need to slice the current vector convert that type into 
                another vector, the reason behind doing this is becasue we can't
                call to_vec() on the slice directly since the lifetime fo the slice
                will be dropped while is getting used we have to create a longer 
                lifetime then call to_vec() on that type
            */            
            let found_collections = if from < found_collections.len(){
                if found_collections.len() > to{
                    let data = &found_collections[from..to+1];
                    data.to_vec()
                } else{
                    let data = &found_collections[from..found_collections.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };

            let found_nfts = if from < found_nfts.len(){
                if found_nfts.len() > to{
                    let data = &found_nfts[from..to+1];
                    data.to_vec()
                } else{
                    let data = &found_nfts[from..found_nfts.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };

            let users_info = if from < users_info.len(){
                if users_info.len() > to{
                    let data = &users_info[from..to+1];
                    data.to_vec()
                } else{
                    let data = &users_info[from..users_info.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };

            let mut matched_data = HashMap::new();
            matched_data.insert("collection".to_string(), serde_json::to_value(&found_collections).unwrap());
            matched_data.insert("users".to_string(), serde_json::to_value(&users_info).unwrap());
            matched_data.insert("nfts".to_string(), serde_json::to_value(&found_nfts).unwrap());
            
            resp!{
                HashMap<String, serde_json::Value>, // the data type
                matched_data, // response data
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

#[get("/search-top-nfts/")]
pub(self) async fn search_in_top_nfts(
        req: HttpRequest,   
        query: web::Query<UnlimitedSearch>,
        app_state: web::Data<AppState>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = app_state.app_sotrage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {
        
            let connection = &mut pg_pool.get().unwrap();
            let mut redis_conn = redis_client.get_async_connection().await.unwrap();

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


            
            let search_query = &query.q;
            let mut match_top_nfts = vec![];
            for nft in top_nfts{

                if nft.nfts_data.nft_name.contains(search_query) ||
                    nft.nfts_data.nft_description.contains(search_query) ||
                    nft.nfts_data.current_owner_screen_cid.contains(search_query) ||
                    nft.nfts_data.contract_address.contains(search_query) ||
                    nft.nfts_data.onchain_id.clone().unwrap_or(String::from("")).contains(search_query) ||
                    nft.nfts_data.tx_hash.clone().unwrap_or(String::from("")).contains(search_query){

                        match_top_nfts.push(nft);
                    }
            }
            
            resp!{
                Vec<NftColInfo>, // the data type
                match_top_nfts, // response data
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
    pub use super::search_in_top_nfts;
    pub use super::search;
}