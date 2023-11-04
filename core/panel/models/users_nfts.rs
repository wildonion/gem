


use diesel::sql_types::Text;
use wallexerr::Wallet;

use crate::*;
use crate::constants::{GALLERY_NOT_OWNED_BY, NFT_NOT_OWNED_BY, NFT_UPLOAD_PATH, INVALID_QUERY_LIMIT, STORAGE_IO_ERROR_CODE, NFT_ONCHAINID_NOT_FOUND};
use crate::misc::{Response, Limit};
use crate::schema::users_nfts::dsl::*;
use crate::schema::users_nfts;
use super::users_collections::UserCollection;
use super::users_galleries::UserPrivateGallery;

/* 

    diesel migration generate users_nfts ---> create users_nfts migration sql files
    diesel migration run                 ---> apply sql files to db 
    diesel migration redo                ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_nfts)]
pub struct UserNft{
    pub id: i32,
    pub contract_address: String,
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
    pub contract_address: String,
    pub event_type: String,
    pub amount: i64,
    pub buyer_screen_cid: Option<String>,
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
    pub metadata_uri: String,
    pub current_owner_screen_cid: String,
    pub nft_name: String,
    pub nft_description: String,
    pub current_price: i64,
    pub freeze_metadata: Option<bool>,
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
    pub freeze_metadata: Option<bool>,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserReactionData{
    pub comments: Vec<NftComment>,
    pub likes: Vec<UserLikeStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NftReactionData{
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
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserNft>(connection)
            {
                Ok(nfts) => {

                    
                    let mut user_reactions = vec![];
                    for nft in nfts{

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

    pub async fn get_all_nft_reactions(nft_onchain_id: &str, limit: web::Query<Limit>, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<NftReactionData, PanelHttpResponse>{
        
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
                likes: this_nft_likes 
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
        let get_gallery = UserPrivateGallery::find_by_owner(&collection_data.owner_screen_cid, connection).await;
        let Ok(gallery_data) = get_gallery else{
            let err_resp = get_gallery.unwrap_err();
            return Err(err_resp);
        };
        
        let caller_screen_cid = Wallet::generate_keccak256_from(asset_info.caller_cid);
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

        /* uploading nft image */
        let get_nft_img_path = misc::store_file(
            NFT_UPLOAD_PATH, &asset_info.contract_address, 
            "nft", 
            img).await;
        let Ok(nft_img_path) = get_nft_img_path else{

            let err_res = get_nft_img_path.unwrap_err();
            return Err(err_res);
        };

        // - set asset_info.is_minted to false which means that is not stored on contract yet
        // - by default is_listed will be set to true since an nft goes to private collection by default 
        // - set current price to what it has been passed to the call
        // upload img on pastel using sense and cascade apis: paste::sense::detect(), paste::cascade::upload()
        // spend token for gas fee and update listings
        // which must be listed to be sold to friends have been invited by the gallery owner

        // update col 
        // update gal

        Ok(
            UserNftData::default()
        )

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= GALLERY OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-= */
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
        let get_gallery = UserPrivateGallery::find_by_owner(&collection_data.owner_screen_cid, connection).await;
        let Ok(gallery_data) = get_gallery else{
            let err_resp = get_gallery.unwrap_err();
            return Err(err_resp);
        };


        // update col 
        // update gal 


        match asset_info.event_type.as_str(){
            "mint" => {

                /* ------- charge user balance for gas fee ------- */
                // https://docs.nftport.xyz/reference/customizable-minting
                // asset_info.onchain_id will be fulfilled after minting
                // call nftport::mint_nft()
                todo!()
            },
            "transfer" => {

                /* ------- charge user balance for gas fee ------- */
                // https://docs.nftport.xyz/reference/transfer-minted-nft
                // call nftport::transfer_nft()
                todo!()
            },
            "sell" => {
                
                // update is_listed field
                todo!()
            },
            "buy" => {
                
                /* ------- charge user balance for gas fee ------- */
                // update balance field of royalties_address_screen_cid in each nft sell
                // if the nft is_listed field was set to true then nft can be sold out to the asset_info.buyer_screen_cid
                // transfer nft ownership to the asset_info.buyer_screen_cid
                /* consider royalties process of the contract based on in-app token */
                // call nftport::mint_nft()
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

                /* uploading nft image */
                let get_nft_img_path = misc::store_file(
                    NFT_UPLOAD_PATH, &asset_info.contract_address, 
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