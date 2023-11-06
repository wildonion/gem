


use wallexerr::Wallet;
use crate::*;
use crate::adapters::nftport;
use crate::constants::{GALLERY_NOT_OWNED_BY, NFT_NOT_OWNED_BY, NFT_UPLOAD_PATH, INVALID_QUERY_LIMIT, STORAGE_IO_ERROR_CODE, NFT_ONCHAINID_NOT_FOUND, NFT_UPLOAD_ISSUE};
use crate::misc::{Response, Limit};
use crate::schema::users_nfts::dsl::*;
use crate::schema::users_nfts;
use super::users::User;
use super::users_collections::{UserCollection, UserCollectionData, UpdateUserCollection};
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest};
use crate::schema::users_collections::dsl::*;
use crate::schema::users_collections;

/* 

    diesel migration generate users_nfts ---> create users_nfts migration sql files
    diesel migration run                 ---> apply sql files to db 
    diesel migration redo                ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_nfts)]
pub struct UserNft{
    pub id: i32,
    pub contract_address: String, /* this can be used to fetch the collection info cause every collection is a contract on the chain */
    pub current_owner_screen_cid: String,
    pub metadata_uri: String, /* an ipfs link contains metadata json file */
    pub onchain_id: Option<String>,
    pub nft_name: String,
    pub nft_description: String,
    pub is_minted: Option<bool>,
    pub current_price: Option<i64>,
    pub is_listed: Option<bool>,
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
    pub published_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NftLike{
    pub nft_onchain_id: String,
    pub upvoter_screen_cids: Vec<String>,
    pub downvoter_screen_cids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserLikeStat{
    pub nft_onchain_id: String,
    pub is_upvote: bool,
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
    pub is_minted: Option<bool>,
    pub nft_description: String,
    pub current_price: Option<i64>,
    pub is_listed: Option<bool>,
    pub freeze_metadata: Option<bool>,
    pub comments: Option<serde_json::Value>,
    pub likes: Option<serde_json::Value>,
    pub tx_hash: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserNftRequest{
    pub caller_cid: String,
    pub buyer_screen_cid: Option<String>,
    pub amount: i64, // amount of gas fee for this call
    pub event_type: String,
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
    pub caller_cid: String,
    pub amount: i64,
    pub contract_address: String,
    pub current_owner_screen_cid: String,
    pub nft_name: String,
    pub nft_description: String,
    pub current_price: i64,
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
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserReactionData{
    pub nft_metadata_uri: String,
    pub nft_onchain_addres: Option<String>,
    pub comments: Vec<NftComment>,
    pub likes: Vec<UserLikeStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NftReactionData{
    pub nft_metadata_uri: String,
    pub nft_onchain_addres: Option<String>,
    pub nft_created_at: String,
    pub comments: Vec<NftComment>,
    pub likes: Vec<NftLike>,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserNft{

    pub async fn get_all_user_reactions(caller_screen_cid: &str, limit: web::Query<Limit>, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserReactionData>, PanelHttpResponse>{

        
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


        match users_nfts
            .order(users_nfts::created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserNft>(connection)
            {
                Ok(nfts_) => {

                    
                    let mut user_reactions = vec![];
                    for nft in nfts_{

                        let nft_comments = nft.comments;
                        let decoded_comments = if nft_comments.is_some(){
                            serde_json::from_value::<Vec<NftComment>>(nft_comments.clone().unwrap()).unwrap()
                        } else{
                            vec![]
                        };

                        let nft_likes = nft.likes;
                        let decoded_likes = if nft_likes.is_some(){
                            serde_json::from_value::<Vec<NftLike>>(nft_likes.clone().unwrap()).unwrap()
                        } else{
                            vec![]
                        };
                        
                        
                        let mut owner_comments = vec![];
                        for comment in decoded_comments{
                            if comment.owner_screen_cid == caller_screen_cid{
                                owner_comments.push(comment);
                            }
                        }

                        let mut owner_likes = vec![];
                        for like in decoded_likes{
                            
                            let like_stat_data = if like.upvoter_screen_cids.contains(&caller_screen_cid.to_string()){
                                    Some(
                                        UserLikeStat{
                                            nft_onchain_id: like.nft_onchain_id,
                                            is_upvote: true,
                                        }
                                    )
                                } else if like.downvoter_screen_cids.contains(&caller_screen_cid.to_string()){

                                    Some(
                                        UserLikeStat{
                                            nft_onchain_id: like.nft_onchain_id,
                                            is_upvote: false,
                                        }
                                    )

                                } else{
                                    None
                                };
                                
                                if like_stat_data.is_some(){
                                    owner_likes.push(like_stat_data.unwrap())
                                }
                                    
                            };

                        
                        user_reactions.push(
                            UserReactionData{
                                comments: owner_comments,
                                likes: owner_likes,
                                nft_metadata_uri: nft.metadata_uri,
                                nft_onchain_addres: nft.onchain_id,
                            }
                        )
                    
    
                    }
                     
                    Ok(user_reactions)

                },
                Err(e) => {
    
                    let resp_err = &e.to_string();
    
    
                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::get_all_user_reactions");
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

    pub async fn get_all_nft_reactions(nft_onchain_id: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<NftReactionData, PanelHttpResponse>{
        
        let get_nft = users_nfts
            .filter(users_nfts::onchain_id.eq(nft_onchain_id))
            .first::<UserNft>(connection);
        
        
        let Ok(nft) = get_nft else{
            let resp = Response::<String>{
                data: Some(nft_onchain_id.to_string()),
                message: NFT_ONCHAINID_NOT_FOUND,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };


        let nft_comments = nft.comments;
        let decoded_comments = if nft_comments.is_some(){
            serde_json::from_value::<Vec<NftComment>>(nft_comments.clone().unwrap()).unwrap()
        } else{
            vec![]
        };

        let nft_likes = nft.likes;
        let decoded_likes = if nft_likes.is_some(){
            serde_json::from_value::<Vec<NftLike>>(nft_likes.clone().unwrap()).unwrap()
        } else{
            vec![]
        };
        
        
        let mut this_nft_comments = vec![];
        for comment in decoded_comments{
            if comment.nft_onchain_id == nft_onchain_id{
                this_nft_comments.push(comment);
            }
        } 

        let mut this_nft_likes = vec![];
        for like in decoded_likes{
            if like.nft_onchain_id == nft_onchain_id{
                this_nft_likes.push(like);
            }
        } 
        

        Ok(
            NftReactionData{ 
                comments: this_nft_comments, 
                likes: this_nft_likes,
                nft_metadata_uri: nft.metadata_uri,
                nft_onchain_addres: nft.onchain_id,
                nft_created_at: nft.created_at.to_string(),
            }
        )

    }

    pub async fn find_by_current_owner(current_owner: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{

        let user_nft = users_nfts
            .filter(users_nfts::current_owner_screen_cid.eq(current_owner))
            .first::<UserNft>(connection);

        let Ok(nft) = user_nft else{

            let resp = Response{
                data: Some(current_owner),
                message: NFT_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )

        };

        Ok(
            UserNftData{ 
                id: nft.id, 
                contract_address: nft.contract_address, 
                current_owner_screen_cid: nft.current_owner_screen_cid, 
                metadata_uri: nft.metadata_uri, 
                extra: nft.extra, 
                onchain_id: nft.onchain_id, 
                nft_name: nft.nft_name, 
                is_minted: nft.is_minted, 
                nft_description: nft.nft_description, 
                current_price: nft.current_price, 
                is_listed: nft.is_listed, 
                freeze_metadata: nft.freeze_metadata, 
                comments: nft.comments, 
                likes: nft.likes, 
                tx_hash: nft.tx_hash, 
                created_at: nft.created_at.to_string(), 
                updated_at: nft.updated_at.to_string() 
            }
        )

    }
    

}

impl UserNft{

    pub async fn insert(asset_info: NewUserNftRequest, mut img: Multipart,
        redis_client: redis::Client,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{
            
        /* find a collection data with the passed in contract address */
        let get_collection = UserCollection::find_by_contract_address(&asset_info.contract_address, connection).await;
        let Ok(collection_data) = get_collection else{
            let err_resp = get_collection.unwrap_err();
            return Err(err_resp);
        };

        /* find a gallery data with the passed in owner screen address of the found collection */
        let get_gallery = UserPrivateGallery::find_by_owner_and_contract_address(&collection_data.owner_screen_cid, &collection_data.contract_address, connection).await;
        let Ok(gallery_data) = get_gallery else{
            let err_resp = get_gallery.unwrap_err();
            return Err(err_resp);
        };
        
        let caller_screen_cid = Wallet::generate_keccak256_from(asset_info.clone().caller_cid);
        if gallery_data.owner_screen_cid != caller_screen_cid{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            );
        }

        let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
        let Ok(user) = get_user else{

            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };

        /* uploading nft image on server */
        let get_nft_img_path = misc::store_file(
            NFT_UPLOAD_PATH, &format!("nft:{}-incontract:{}-by:{}", asset_info.nft_name, asset_info.contract_address, asset_info.current_owner_screen_cid), 
            "nft", 
            img).await;
        let Ok(nft_img_path) = get_nft_img_path else{

            let err_res = get_nft_img_path.unwrap_err();
            return Err(err_res);
        };


        /* 
            upload nft in the background inside tokio green threadpool, the metadata uri
            must be shared between outside of the threadpool and tokio spawn using 
            mpsc jobq channel
        */
        let mut nft_metadata_uri = String::from("");
        let (metadata_uri_sender, mut metadata_uri_receiver)
            = tokio::sync::mpsc::channel::<String>(1024);
        let asset_data = asset_info.clone();
        tokio::spawn(async move{
  
            let final_metadata_uri = nftport::upload_nft_to_ipfs(
                redis_client.clone(), 
                nft_img_path,
                asset_data
            ).await;

            if let Err(why) = metadata_uri_sender.clone().send(final_metadata_uri).await{
                error!("can't send `final_metadata_uri` to the mpsc channel because: {}", why.to_string());
            }

        });

        /* receiving asyncly from the channel in outside of the tokio spawn */
        while let Some(uri) = metadata_uri_receiver.recv().await{
            nft_metadata_uri = uri;
        }

        if nft_metadata_uri.is_empty(){

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: NFT_UPLOAD_ISSUE,
                status: 417,
            };
            return Err(
                Ok(HttpResponse::ExpectationFailed().json(resp))
            );

        }

        /* 
            update user balance frist, if anything goes wrong they can call us 
            to pay them back, actually this is the gas fee that they must be 
            charged for since we already have paid the fee when we did the 
            onchain process
        */
        let new_balance = user.balance.unwrap() - asset_info.amount;
        let update_user_balance = User::update_balance(user.id, new_balance, connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        /*  ---------------------------------
            default values will be stored as:
                - is_minted       :‌ false ----- by default nft goes to private gallery
                - is_listed       : true  ----- by default nft is listed and some who has invited to the gallery can mint and buy it
                - freeze_metadata : false ----- by default nft metadata must not be frozen onchain 
        */
        let new_insert_nft = InsertNewUserNftRequest{
            contract_address: collection_data.clone().contract_address,
            current_owner_screen_cid: caller_screen_cid,
            metadata_uri: nft_metadata_uri,
            nft_name: asset_info.nft_name,
            nft_description: asset_info.nft_description,
            current_price: asset_info.current_price,
            extra: asset_info.extra,
        };

        /* inserting new nft */
        match diesel::insert_into(users_nfts)
            .values(&new_insert_nft)
            .returning(UserNft::as_returning())
            .get_result::<UserNft>(connection)
            {
                Ok(fetched_nft_data) => {
                    
                    let user_nft_data = UserNftData{
                        id: fetched_nft_data.clone().id,
                        contract_address: fetched_nft_data.clone().contract_address,
                        current_owner_screen_cid: fetched_nft_data.clone().current_owner_screen_cid,
                        metadata_uri: fetched_nft_data.clone().metadata_uri,
                        extra: fetched_nft_data.clone().extra,
                        onchain_id: fetched_nft_data.clone().onchain_id,
                        nft_name: fetched_nft_data.clone().nft_name,
                        is_minted: fetched_nft_data.clone().is_minted,
                        nft_description: fetched_nft_data.clone().nft_description,
                        current_price: fetched_nft_data.clone().current_price,
                        is_listed: fetched_nft_data.clone().is_listed,
                        freeze_metadata: fetched_nft_data.clone().freeze_metadata,
                        comments: fetched_nft_data.clone().comments,
                        likes: fetched_nft_data.clone().likes,
                        tx_hash: fetched_nft_data.clone().tx_hash,
                        created_at: fetched_nft_data.clone().created_at.to_string(),
                        updated_at: fetched_nft_data.clone().updated_at.to_string(),
                    };

                    /* updating collection data with newly nft */
                    let new_collection_data = UpdateUserCollection{
                        nfts: {
                            let nfts_ = collection_data.clone().nfts;
                            let mut decoded_nfts = if nfts_.is_some(){
                                serde_json::from_value::<Vec<UserNftData>>(nfts_.clone().unwrap()).unwrap()
                            } else{
                                vec![]
                            };

                            /* since this is new nft we have to push */
                            decoded_nfts.push(user_nft_data.clone());

                            Some(
                                serde_json::to_value(decoded_nfts).unwrap()
                            )
                        },
                        freeze_metadata: collection_data.clone().freeze_metadata,
                        base_uri: collection_data.clone().base_uri,
                        royalties_share: collection_data.clone().royalties_share,
                        royalties_address_screen_cid: collection_data.clone().royalties_address_screen_cid,
                        collection_background: collection_data.clone().collection_background,
                        extra: collection_data.clone().extra,
                        contract_tx_hash: collection_data.clone().contract_tx_hash.unwrap(),
                        col_description: collection_data.clone().col_description,
                    };

                    match diesel::update(users_collections.filter(users_collections::id.eq(collection_data.id)))
                        .set(&new_collection_data)
                        .returning(UserCollection::as_returning())
                        .get_result::<UserCollection>(connection)
                        {
                            Ok(fetched_collection_data) => {
                                
                                let user_collection_data = UserCollectionData{
                                    id: fetched_collection_data.clone().id,
                                    contract_address: fetched_collection_data.clone().contract_address,
                                    nfts: fetched_collection_data.clone().nfts,
                                    col_name: fetched_collection_data.clone().col_name,
                                    symbol: fetched_collection_data.clone().symbol,
                                    owner_screen_cid: fetched_collection_data.clone().owner_screen_cid,
                                    metadata_updatable: fetched_collection_data.clone().metadata_updatable,
                                    freeze_metadata: fetched_collection_data.clone().freeze_metadata,
                                    base_uri: fetched_collection_data.clone().base_uri,
                                    royalties_share: fetched_collection_data.clone().royalties_share,
                                    royalties_address_screen_cid: fetched_collection_data.clone().royalties_address_screen_cid,
                                    collection_background: fetched_collection_data.clone().collection_background,
                                    extra: fetched_collection_data.clone().extra,
                                    col_description: fetched_collection_data.clone().col_description,
                                    contract_tx_hash: fetched_collection_data.clone().contract_tx_hash,
                                    created_at: fetched_collection_data.clone().created_at.to_string(),
                                    updated_at: fetched_collection_data.clone().updated_at.to_string(),
                                };

                                /* updating gallery data with the updated collection */
                                let new_gal_data = UpdateUserPrivateGalleryRequest{
                                    collections: {
                                        let cols = gallery_data.collections;
                                        let mut decoded_cols = if cols.is_some(){
                                            serde_json::from_value::<Vec<UserCollectionData>>(cols.clone().unwrap()).unwrap()
                                        } else{
                                            vec![]
                                        };

                                        /* since there is no new collection we should update the old one in vector */
                                        let collection_position = decoded_cols.iter().position(|c| c.contract_address == collection_data.clone().contract_address);
                                        if collection_position.is_some(){
                                            decoded_cols[collection_position.unwrap()] = user_collection_data;
                                        }

                                        Some(
                                            serde_json::to_value(decoded_cols).unwrap()
                                        )
                                    },
                                    gal_name: gallery_data.gal_name,
                                    gal_description: gallery_data.gal_description,
                                    invited_friends: gallery_data.invited_friends,
                                    extra: gallery_data.extra,
                                    owner_cid: asset_info.caller_cid,
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

                                    Ok(updated_gal) => Ok(user_nft_data),
                                    Err(resp) => Err(resp)
                                }
                            },
                            Err(e) => {

                                let resp_err = &e.to_string();
            
            
                                /* custom error handler */
                                use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                
                                let error_content = &e.to_string();
                                let error_content = error_content.as_bytes().to_vec();  
                                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::insert_update_collection");
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

                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::insert_insert_nft");
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
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= NFT OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn update(asset_info: UpdateUserNftRequest, mut img: Multipart,
        redis_client: redis::Client,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{

        let caller_screen_cid = Wallet::generate_keccak256_from(asset_info.caller_cid);
        
        /* find an nft data with the passed in owner address cause only owner can call this method */
        let get_nft = UserNft::find_by_current_owner(&caller_screen_cid, connection).await;
        let Ok(nft_data) = get_nft else{
            let err_resp = get_nft.unwrap_err();
            return Err(err_resp);
        };


        /* find a collection data with the passed in contract address */
        let get_collection = UserCollection::find_by_contract_address(&nft_data.contract_address, connection).await;
        let Ok(collection_data) = get_collection else{
            let err_resp = get_collection.unwrap_err();
            return Err(err_resp);
        };

        /* find a gallery data with the passed in owner screen address of the found collection */
        let get_gallery = UserPrivateGallery::find_by_owner_and_contract_address(&collection_data.owner_screen_cid, &nft_data.contract_address, connection).await;
        let Ok(gallery_data) = get_gallery else{
            let err_resp = get_gallery.unwrap_err();
            return Err(err_resp);
        };

        let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
        let Ok(user) = get_user else{

            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };


        /* 
            update user balance frist, if anything goes wrong they can call us 
            to pay them back, actually this is the gas fee that they must be 
            charged for since we already have paid the fee when we did the 
            onchain process
        */
        let new_balance = user.balance.unwrap() - asset_info.amount;
        let update_user_balance = User::update_balance(user.id, new_balance, connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        // remember to update nft in collections and update collection in gallery not push
        // update col 
        // update gal


        match asset_info.event_type.as_str(){
            "mint" => {

                /* ------- charge user balance for gas fee ------- */
                // https://docs.nftport.xyz/reference/customizable-minting
                // use asset_info.metadata_uri as the nft url 
                // asset_info.onchain_id will be fulfilled after minting
                // set asset_info.is_minted to true
                // call nftport::mint_nft()
                todo!()
            },
            "transfer" => {

                /* ------- charge user balance for gas fee ------- */
                // https://docs.nftport.xyz/reference/transfer-minted-nft
                // call nftport::transfer_nft()
                todo!()
            },
            "sell" => { // update listing
                
                // update is_listed field
                todo!()
            },
            "buy" => {
                
                /* ------- charge user balance for gas fee ------- */
                // update balance field of royalties_address_screen_cid in each nft sell
                // if the nft is_listed field was set to true then nft can be sold out to the asset_info.buyer_screen_cid
                // transfer nft ownership to the asset_info.buyer_screen_cid
                /* consider royalties process of the contract based on in-app token using royalties_share amounts */
                // call nftport::transfer_nft()
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

                /* uploading nft image on server */
                let get_nft_img_path = misc::store_file(
                    NFT_UPLOAD_PATH, &format!("nft:{}-incontract:{}-by:{}", asset_info.nft_name, asset_info.contract_address, asset_info.current_owner_screen_cid), 
                    "nft", 
                    img).await;
                let Ok(nft_img_path) = get_nft_img_path else{
        
                    let err_res = get_nft_img_path.unwrap_err();
                    return Err(err_res);
                };
                
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