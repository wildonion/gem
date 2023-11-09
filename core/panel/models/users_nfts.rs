


use wallexerr::Wallet;
use crate::*;
use crate::adapters::nftport::{self, NftExt};
use crate::constants::{GALLERY_NOT_OWNED_BY, NFT_NOT_OWNED_BY, NFT_UPLOAD_PATH, INVALID_QUERY_LIMIT, STORAGE_IO_ERROR_CODE, NFT_ONCHAINID_NOT_FOUND, NFT_UPLOAD_ISSUE, CANT_MINT_CARD, CANT_MINT_NFT, CANT_TRANSFER_NFT, NFT_EVENT_TYPE_RECIPIENT_IS_NEEDED, NFT_EVENT_TYPE_METADATA_URI_IS_NEEDED, INVALID_NFT_EVENT_TYPE, NFT_IS_NOT_MINTED_YET, CANT_UPDATE_NFT, NFT_NOT_FOUND_OF, NFT_IS_ALREADY_MINTED, NFT_IS_NOT_LISTED_YET, NFT_PRICE_IS_EMPTY, NFT_EVENT_TYPE_BUYER_IS_NEEDED, CALLER_IS_NOT_BUYER, INVALID_NFT_ROYALTY, INVALID_NFT_PRICE, RECIPIENT_SCREEN_CID_NOT_FOUND};
use crate::misc::{Response, Limit};
use crate::schema::users_nfts::dsl::*;
use crate::schema::users_nfts;
use super::users::User;
use super::users_collections::{UserCollection, UserCollectionData, UpdateUserCollection};
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest, UserPrivateGalleryData};
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
    pub attributes: Option<serde_json::Value>, /* pg key, value based json binary object */
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
    pub attributes: Option<serde_json::Value>,
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
    pub transfer_to_screen_cid: Option<String>,
    pub amount: i64, // amount of gas fee for this call
    pub event_type: String,
    pub contract_address: String,
    pub current_owner_screen_cid: String,
    pub metadata_uri: String,
    pub extra: Option<serde_json::Value>,
    pub attributes: Option<serde_json::Value>,
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
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, AsChangeset)]
#[diesel(table_name=users_nfts)]
pub struct UpdateUserNft{
    pub current_owner_screen_cid: String,
    pub metadata_uri: String,
    pub extra: Option<serde_json::Value>,
    pub attributes: Option<serde_json::Value>,
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
    pub attributes: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub tx_signature: String,
    pub hash_data: String,
}


impl NftExt for NewUserNftRequest{
    type AssetInfo = NewUserNftRequest;

    fn get_nft_name(&self) -> String {
        self.nft_name.clone()
    }

    fn get_nft_description(&self) -> String {
        self.nft_description.clone()
    }

    fn get_nft_contract_address(&self) -> String {
        self.contract_address.clone()
    }

    fn get_nft_current_owner_address(&self) -> String {
        self.current_owner_screen_cid.clone()
    }

    fn get_nft_extra(&self) -> Option<serde_json::Value>{
        self.extra.clone()
    }

    fn get_self(self) -> Self::AssetInfo {
        self as NewUserNftRequest
    }

    fn get_recipient_screen_cid(&self) -> String {
        String::from("")
    }

    fn get_nft_attribute(&self) -> Option<serde_json::Value> {
        self.attributes.clone()
    }

}

impl NftExt for UpdateUserNftRequest{
    type AssetInfo = UpdateUserNftRequest;
    
    fn get_nft_description(&self) -> String{
        self.nft_description.clone()
    }

    fn get_nft_name(&self) -> String{
        self.nft_name.clone()
    }

    fn get_nft_contract_address(&self) -> String {
        self.contract_address.clone()
    }

    fn get_nft_current_owner_address(&self) -> String {
        self.current_owner_screen_cid.clone()
    }

    fn get_nft_extra(&self) -> Option<serde_json::Value> {
        self.extra.clone()
    }

    fn get_self(self) -> Self::AssetInfo {
        self as UpdateUserNftRequest
    }

    fn get_recipient_screen_cid(&self) -> String {
        
        /* 
            since unwrap() takes the ownership of type we shouldn't allow this
            to be happened for self cause self is behind a shared reference
            and can't be moved cause by moving the whole instance will be moved
            solution to this is cloning or using as_ref() method
        */
        if self.buyer_screen_cid.is_some() && self.event_type == "buy"{
            return self.buyer_screen_cid.clone().unwrap();
        }

        if self.transfer_to_screen_cid.is_some() && self.event_type == "transfer"{
            return self.transfer_to_screen_cid.clone().unwrap();
        }

        String::from("")
    }

    fn get_nft_attribute(&self) -> Option<serde_json::Value> {
        self.attributes.clone()
    }
    
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
    pub attributes: Option<serde_json::Value>, /* pg key, value based json binary object */
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AddReactionRequest{
    pub caller_cid: String,
    pub nft_onchain_id: String,
    pub reaction_type: String, // comment or like or dislike
    pub comment_content: Option<String>,
    pub is_like_upvote: Option<bool>,
    pub is_like_downvote: Option<bool>,
    pub tx_signature: String,
    pub hash_data: String
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
                updated_at: nft.updated_at.to_string(),
                attributes: nft.attributes, 
            }
        )

    }

    pub async fn find_by_onchain_id(onchain_id_: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{

        let user_nft = users_nfts
            .filter(users_nfts::onchain_id.eq(onchain_id_))
            .first::<UserNft>(connection);

        let Ok(nft) = user_nft else{

            let resp = Response{
                data: Some(onchain_id_),
                message: NFT_NOT_FOUND_OF,
                status: 404,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
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
                updated_at: nft.updated_at.to_string(),
                attributes: nft.attributes, 
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

        /* find a gallery data with the passed in owner screen address */
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


        /* start uploading nft onchain */
        let get_nft_metadata_uri = 
            nftport::get_nft_onchain_metadata_uri(
                img,
                redis_client.clone(),
                asset_info.clone()
            ).await;
        
        let Ok(nft_metadata_uri) = get_nft_metadata_uri else{

            let err_resp = get_nft_metadata_uri.unwrap_err();
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

        /*  ---------------------------------
            default values will be stored as:
                - is_minted       :‌ false ----- by default nft goes to private gallery
                - is_listed       : false ----- by default nft isn't listed
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
            attributes: asset_info.attributes,
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
                        attributes: fetched_nft_data.attributes,
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

    pub async fn update_nft_col_gal(
        collection_data: UserCollectionData, 
        gallery_data: UserPrivateGalleryData, 
        udpate_nft_request: UpdateUserNftRequest, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{

        
        let update_nft_data = UpdateUserNft{
            current_owner_screen_cid: udpate_nft_request.current_owner_screen_cid,
            metadata_uri: udpate_nft_request.metadata_uri,
            extra: udpate_nft_request.extra,
            onchain_id: udpate_nft_request.onchain_id,
            nft_name: udpate_nft_request.nft_name,
            is_minted: udpate_nft_request.is_minted,
            nft_description: udpate_nft_request.nft_description,
            current_price: udpate_nft_request.current_price,
            is_listed: udpate_nft_request.is_listed,
            freeze_metadata: udpate_nft_request.freeze_metadata,
            comments: udpate_nft_request.comments,
            likes: udpate_nft_request.likes,
            tx_hash: udpate_nft_request.tx_hash,
            attributes: udpate_nft_request.attributes,
        };
        
        /* inserting new nft */
        match diesel::update(users_nfts)
            .set(&update_nft_data)
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
                        attributes: fetched_nft_data.attributes,
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

                            /* since there is no new nft we should update the old one in vector */
                            let nft_position = decoded_nfts.iter().position(|nft| nft.current_owner_screen_cid == user_nft_data.clone().current_owner_screen_cid);
                            if nft_position.is_some(){
                                decoded_nfts[nft_position.unwrap()] = user_nft_data.clone();
                            }

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
                                    owner_cid: udpate_nft_request.caller_cid,
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
                                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::update_nft_col_gal");
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
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::update_nft_col_gal");
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

    pub async fn add_reaction_to_nft(add_reaction_request: AddReactionRequest, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{
        
        let caller_screen_cid = Wallet::generate_keccak256_from(add_reaction_request.clone().caller_cid);

        let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
        let Ok(user) = get_user else{

            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };

        let get_nft = UserNft::find_by_onchain_id(&add_reaction_request.nft_onchain_id, connection).await;
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

        /* find a gallery data with the passed in owner screen address */
        let get_gallery = UserPrivateGallery::find_by_contract_address(&nft_data.contract_address, connection).await;
        let Ok(gallery_data) = get_gallery else{
            let err_resp = get_gallery.unwrap_err();
            return Err(err_resp);
        };

        
        let updated_nft_data = UpdateUserNftRequest{
            caller_cid: add_reaction_request.clone().caller_cid,
            buyer_screen_cid: None,
            transfer_to_screen_cid: None,
            amount: 0,
            event_type: String::from(""),
            contract_address: nft_data.contract_address,
            current_owner_screen_cid: nft_data.current_owner_screen_cid,
            metadata_uri: nft_data.metadata_uri,
            extra: nft_data.extra,
            onchain_id: nft_data.onchain_id,
            nft_name: nft_data.nft_name,
            is_minted: nft_data.is_minted,
            nft_description: nft_data.nft_description,
            current_price: nft_data.current_price,
            is_listed: nft_data.is_listed,
            freeze_metadata: nft_data.freeze_metadata,
            comments: {
                let nft_comments = nft_data.comments;
                let mut decoded_comments = if nft_comments.is_some(){
                    serde_json::from_value::<Vec<NftComment>>(nft_comments.unwrap()).unwrap()
                } else{
                    vec![]
                };

                if add_reaction_request.comment_content.is_some(){

                    decoded_comments.push(
                        NftComment{ 
                            nft_onchain_id: add_reaction_request.clone().nft_onchain_id, 
                            content: add_reaction_request.clone().comment_content.unwrap(), 
                            owner_screen_cid: caller_screen_cid.clone(), 
                            published_at: chrono::Local::now().timestamp(),
                        }
                    );

                }

                Some(serde_json::to_value(decoded_comments).unwrap())
            },
            likes: {

                let nft_likes = nft_data.likes;
                let decoded_likes = if nft_likes.is_some(){
                    serde_json::from_value::<Vec<NftLike>>(nft_likes.unwrap()).unwrap()
                } else{
                    vec![]
                };

                let this_nft_position = decoded_likes
                    .iter()
                    .position(|nft| nft.nft_onchain_id == add_reaction_request.nft_onchain_id);

                if add_reaction_request.is_like_upvote.is_some(){

                    if this_nft_position.is_some(){
                        /* 
                            getting a mutable pointer to the found NftLike instance inside decoded_likes
                            means that if we mutate the instance in other scopes the actual type inside 
                            the vector will be mutated too
                        */
                        let this_nft_likes = &mut decoded_likes[this_nft_position.unwrap()].clone();
                        for upvote in this_nft_likes.clone().upvoter_screen_cids{
                            let caller = caller_screen_cid.clone();
                            if upvote != caller{
                                this_nft_likes.upvoter_screen_cids.push(caller);
                            }
                        }

                        /* remove the caller from downvoters if there was any since he liked the nft */
                        if this_nft_likes.clone().downvoter_screen_cids
                            .into_iter()
                            .any(|scid| scid == caller_screen_cid.clone()){

                                let downvoter_position_scid = this_nft_likes.downvoter_screen_cids.iter().position(|scid| scid == &caller_screen_cid.clone());
                                if downvoter_position_scid.is_some(){
                                    this_nft_likes.downvoter_screen_cids.remove(downvoter_position_scid.unwrap());
                                }
                            }
                    }
        
                }
        
                if add_reaction_request.is_like_downvote.is_some(){
        
                    if this_nft_position.is_some(){
                        /* 
                            getting a mutable pointer to the found NftLike instance inside decoded_likes
                            means that if we mutate the instance in other scopes the actual type inside 
                            the vector will be mutated too
                        */
                        let this_nft_likes = &mut decoded_likes[this_nft_position.unwrap()].clone();
                        for downvote in this_nft_likes.clone().downvoter_screen_cids{
                            let caller = caller_screen_cid.clone();
                            if downvote != caller{
                                this_nft_likes.downvoter_screen_cids.push(caller);
                            }
                        }

                        /* remove the caller from upvoters if there was any since he disliked the nft */
                        if this_nft_likes.clone().upvoter_screen_cids
                            .into_iter()
                            .any(|scid| scid == caller_screen_cid.clone()){

                                let upvoter_position_scid = this_nft_likes.upvoter_screen_cids.iter().position(|scid| scid == &caller_screen_cid.clone());
                                if upvoter_position_scid.is_some(){
                                    this_nft_likes.upvoter_screen_cids.remove(upvoter_position_scid.unwrap());
                                }
                            }
                    }
                }

                Some(serde_json::to_value(decoded_likes).unwrap())
            },
            tx_hash: nft_data.tx_hash,
            tx_signature: String::from(""),
            hash_data: String::from(""),
            attributes: nft_data.attributes,
        };


        Self::update_nft_col_gal(
            collection_data, 
            gallery_data, 
            updated_nft_data, 
            connection
        ).await

    }

    pub async fn buy_nft(mut buy_nft_request: UpdateUserNftRequest, redis_client: redis::Client,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{

        
        if buy_nft_request.event_type.as_str() == "buy"{

            let caller_screen_cid = Wallet::generate_keccak256_from(buy_nft_request.clone().caller_cid);

            let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
            let Ok(user) = get_user else{

                let err_resp = get_user.unwrap_err();
                return Err(err_resp);
            };

            let get_nft = UserNft::find_by_onchain_id(&buy_nft_request.clone().onchain_id.unwrap(), connection).await;
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

            /* find a gallery data with the passed in owner screen address */
            let get_gallery = UserPrivateGallery::find_by_contract_address(&nft_data.contract_address, connection).await;
            let Ok(gallery_data) = get_gallery else{
                let err_resp = get_gallery.unwrap_err();
                return Err(err_resp);
            };

            /* if the onchain id wasn't found we simply terminate the caller */
            if nft_data.is_minted.is_none() && nft_data.onchain_id.is_none(){

                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: NFT_IS_NOT_MINTED_YET,
                    status: 406,
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                ); 

            }

            if nft_data.is_listed.is_some() && nft_data.is_listed.unwrap() == true{

                if buy_nft_request.buyer_screen_cid.is_some(){

                    let buyer_screen_cid = buy_nft_request.buyer_screen_cid.clone().unwrap();

                    if caller_screen_cid != buyer_screen_cid{
                        
                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: CALLER_IS_NOT_BUYER,
                            status: 406,
                        };
                        return Err(
                            Ok(HttpResponse::NotAcceptable().json(resp))
                        );
                    }

                    let get_nft_price = buy_nft_request.current_price;
                    if get_nft_price.is_none(){

                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: NFT_PRICE_IS_EMPTY,
                            status: 406,
                        };
                        return Err(
                            Ok(HttpResponse::NotAcceptable().json(resp))
                        );
                        
                    }

                    let royalty = collection_data.royalties_share;
                    let royalty_owner = collection_data.clone().royalties_address_screen_cid;
                    let seller = buy_nft_request.clone().current_owner_screen_cid;

                    let get_royalty_owner_info = User::find_by_screen_cid(&royalty_owner, connection).await;
                    let Ok(royalty_owner_info) = get_royalty_owner_info else{

                        let err_resp = get_royalty_owner_info.unwrap_err();
                        return Err(err_resp);
                    };

                    let get_seller_info = User::find_by_screen_cid(&seller, connection).await;
                    let Ok(seller_info) = get_seller_info else{

                        let err_resp = get_seller_info.unwrap_err();
                        return Err(err_resp);
                    };
                    
                    /* ---------------------------------------------------------------------------- */
                    /*                  calculating royalty for collection
                        since royalties_address_screen_cid is heap data thus by getting this field 
                        the collection_data instance will be moved, we should clone it 
                    */
                    /* ---------------------------------------------------------------------------- */
                    let nft_price = get_nft_price.unwrap() as f64;
                    let royalty_bps = (royalty as f64 / 100.0) as f64;
                    let percent = percentage::Percentage::from_decimal(royalty_bps);
                    let royalty_amount = percent.apply_to(nft_price);
                    
                    if royalty_amount > nft_price{

                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: INVALID_NFT_ROYALTY,
                            status: 406,
                        };
                        return Err(
                            Ok(HttpResponse::NotAcceptable().json(resp))
                        );

                    }

                    let pay_to_seller = (nft_price - royalty_amount.round()) as i64;

                    /* --------------------------------------------- */
                    /* -------------- update balances -------------- */
                    /* --------------------------------------------- */
                    /* update buyer balance (nft price + onchain gas fee) */
                    let new_balance = user.balance.unwrap() - (nft_price as i64 + buy_nft_request.amount);
                    let update_user_balance = User::update_balance(user.id, new_balance, connection).await;
                    let Ok(updated_user_data) = update_user_balance else{

                        let err_resp = update_user_balance.unwrap_err();
                        return Err(err_resp);
                        
                    };

                    /* update seller balance */
                    let new_balance = user.balance.unwrap() + pay_to_seller as i64;
                    let update_user_balance = User::update_balance(seller_info.id, new_balance, connection).await;
                    let Ok(updated_user_data) = update_user_balance else{

                        let err_resp = update_user_balance.unwrap_err();
                        return Err(err_resp);
                        
                    };

                    /* update royalty owner balance */
                    let new_balance = user.balance.unwrap() + royalty_amount as i64;
                    let update_user_balance = User::update_balance(royalty_owner_info.id, new_balance, connection).await;
                    let Ok(updated_user_data) = update_user_balance else{

                        let err_resp = update_user_balance.unwrap_err();
                        return Err(err_resp);
                        
                    };

                    /* ----------------------------------------------------- */
                    /* ------- transferring the ownership of the nft ------- */
                    /* ----------------------------------------------------- */
                    let (new_tx_hash, status) = 
                        nftport::transfer_nft(
                            redis_client.clone(), 
                            buy_nft_request.clone()
                        ).await;

                    if status == 1{
                        
                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: CANT_TRANSFER_NFT,
                            status: 417,
                        };
                        return Err(
                            Ok(HttpResponse::ExpectationFailed().json(resp))
                        );
                    }

                    buy_nft_request.tx_hash = Some(new_tx_hash);
                    buy_nft_request.current_owner_screen_cid = buyer_screen_cid;
                    buy_nft_request.is_listed = Some(false);
                    buy_nft_request.current_price = Some(0);

                    Self::update_nft_col_gal(
                        collection_data, 
                        gallery_data, 
                        buy_nft_request.clone(), 
                        connection).await
                    
                } else{
                    
                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_EVENT_TYPE_BUYER_IS_NEEDED,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    );
                }


            } else{
                
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: NFT_IS_NOT_LISTED_YET,
                    status: 406,
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                );
            }
        
        } else{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_NFT_EVENT_TYPE,
                status: 406,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

    }

    pub async fn mint_nft(mut mint_nft_request: UpdateUserNftRequest, redis_client: redis::Client,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{

        
        if mint_nft_request.event_type.as_str() == "mint"{

            let caller_screen_cid = Wallet::generate_keccak256_from(mint_nft_request.clone().caller_cid);

            let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
            let Ok(user) = get_user else{

                let err_resp = get_user.unwrap_err();
                return Err(err_resp);
            };

            let get_nft = UserNft::find_by_current_owner(&mint_nft_request.clone().current_owner_screen_cid, connection).await;
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

            /* find a gallery data with the passed in owner screen address */
            let get_gallery = UserPrivateGallery::find_by_contract_address(&nft_data.contract_address, connection).await;
            let Ok(gallery_data) = get_gallery else{
                let err_resp = get_gallery.unwrap_err();
                return Err(err_resp);
            };

            if nft_data.is_minted.is_some() && 
                nft_data.is_minted.unwrap() == true &&
                nft_data.onchain_id.is_some() &&
                !nft_data.onchain_id.unwrap().is_empty(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_IS_ALREADY_MINTED,
                        status: 302,
                    };
                    return Err(
                        Ok(HttpResponse::Found().json(resp))
                    );

                }

                if mint_nft_request.metadata_uri.is_empty(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_EVENT_TYPE_METADATA_URI_IS_NEEDED,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 
                
                }

                if mint_nft_request.is_listed.is_none(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_IS_NOT_LISTED_YET,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 
                
                }

                let get_nft_price = mint_nft_request.current_price;
                if get_nft_price.is_none(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_PRICE_IS_EMPTY,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    );
                    
                }
                let nft_price = get_nft_price.unwrap();


                let (new_tx_hash, token_id, status) = 
                    nftport::mint_nft(redis_client.clone(), mint_nft_request.clone()).await;

                if status == 1{
                    
                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: CANT_MINT_NFT,
                        status: 417,
                    };
                    return Err(
                        Ok(HttpResponse::ExpectationFailed().json(resp))
                    );
                }

                mint_nft_request.is_minted = Some(true);
                mint_nft_request.tx_hash = Some(new_tx_hash); /* updating tx hash with the latest onchain update */
                mint_nft_request.onchain_id = Some(token_id);
                mint_nft_request.current_owner_screen_cid = caller_screen_cid;
                mint_nft_request.is_listed = Some(false);
                mint_nft_request.current_price = Some(0);


                /* update minter balance (nft price + onchain gas fee) */
                let new_balance = user.balance.unwrap() - (nft_price + mint_nft_request.amount);
                let update_user_balance = User::update_balance(user.id, new_balance, connection).await;
                let Ok(updated_user_data) = update_user_balance else{

                    let err_resp = update_user_balance.unwrap_err();
                    return Err(err_resp);
                    
                };
                
                Self::update_nft_col_gal(
                    collection_data, 
                    gallery_data, 
                    mint_nft_request.clone(), 
                    connection).await


        } else{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_NFT_EVENT_TYPE,
                status: 406,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= NFT OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn update(mut asset_info: UpdateUserNftRequest, mut img: Multipart,
        redis_client: redis::Client,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{

        let caller_screen_cid = Wallet::generate_keccak256_from(asset_info.clone().caller_cid);
        
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

        /* find a gallery data with the passed in owner screen address */
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

        if nft_data.current_owner_screen_cid != user.screen_cid.unwrap(){
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: NFT_NOT_OWNED_BY,
                status: 403,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            );
        }

        let res = match asset_info.event_type.as_str(){
            "transfer" => {                
                
                if nft_data.is_minted.is_none() && nft_data.onchain_id.is_none(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_IS_NOT_MINTED_YET,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }
                
                if asset_info.transfer_to_screen_cid.is_none(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_EVENT_TYPE_RECIPIENT_IS_NEEDED,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }
                    
                let recipient = asset_info.clone().transfer_to_screen_cid.unwrap();

                /* make sure that there is a user with this screen cid in the app */
                let get_recipient = User::find_by_screen_cid(&recipient, connection).await;
                let Ok(recipient_info) = get_recipient else{

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: RECIPIENT_SCREEN_CID_NOT_FOUND,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 
                    
                };


                let (new_tx_hash, status) = 
                    nftport::transfer_nft(
                        redis_client.clone(), 
                        asset_info.clone()
                    ).await;

                if status == 1{
                    
                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: CANT_TRANSFER_NFT,
                        status: 417,
                    };
                    return Err(
                        Ok(HttpResponse::ExpectationFailed().json(resp))
                    );
                }

                asset_info.tx_hash = Some(new_tx_hash); /* updating tx hash with the latest onchain update */
                asset_info.current_owner_screen_cid = recipient; 
                asset_info.is_listed = Some(false); /* set is_listed to false for new owner */
                asset_info.current_price = Some(0); /* set current_price to false for new owner */
                
                Self::update_nft_col_gal(
                    collection_data, 
                    gallery_data, 
                    asset_info.clone(), 
                    connection).await


            },
            "sell" => { 
                
                /* 
                    the only way to update price is by update listing 
                    of the nft and the nft must be minted already
                */ 
                
                if nft_data.is_minted.is_none(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_IS_NOT_MINTED_YET,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }

                if asset_info.current_price.is_none(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_PRICE_IS_EMPTY,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }

                if asset_info.current_price.is_some() && asset_info.current_price.unwrap() < 0{

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: INVALID_NFT_PRICE,
                        status: 406,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }

                asset_info.is_listed = Some(true);

                Self::update_nft_col_gal(
                    collection_data, 
                    gallery_data, 
                    asset_info.clone(), 
                    connection).await

            },
            "update" => { // only freeze_metadata and metadata_uri

                asset_info.metadata_uri = if nft_data.freeze_metadata.is_some() &&
                    nft_data.freeze_metadata.unwrap() == true{

                        /* just ignore updating metadata_uri and use the old one */
                        nft_data.metadata_uri 

                    } else{

                        /* start uploading new nft metdata onchain */
                        let get_nft_metadata_uri = 
                        nftport::get_nft_onchain_metadata_uri(
                            img,
                            redis_client.clone(),
                            asset_info.clone()
                        ).await;
                    
                        let Ok(nft_metadata_uri) = get_nft_metadata_uri else{

                            let err_resp = get_nft_metadata_uri.unwrap_err();
                            return Err(err_resp);
                        };

                    nft_metadata_uri
                };

                /* updating freeze_metadata field */
                if asset_info.freeze_metadata.is_some() &&
                    asset_info.freeze_metadata.unwrap() == false{

                        /* next call user can upload new metadata uri */
                        asset_info.freeze_metadata = Some(false);
                
                } else{

                    /* next call we'll use the old metadata uri but can update freeze_metadata and set it to flase */
                    asset_info.freeze_metadata = Some(true);

                }
                
                let (new_tx_hash, status) = 
                    nftport::update_nft(redis_client.clone(), asset_info.clone()).await;

                if status == 1{
                    
                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: CANT_UPDATE_NFT,
                        status: 417,
                    };
                    return Err(
                        Ok(HttpResponse::ExpectationFailed().json(resp))
                    );
                }

                asset_info.tx_hash = Some(new_tx_hash);

                Self::update_nft_col_gal(
                    collection_data, 
                    gallery_data, 
                    asset_info.clone(), 
                    connection).await

            },
            _ => {
                
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_NFT_EVENT_TYPE,
                    status: 406,
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                );
            }
        };

        if res.is_ok(){

            /* if any update goes well we charge the user for onchain gas fee */
            let new_balance = user.balance.unwrap() - asset_info.amount;
            let update_user_balance = User::update_balance(user.id, new_balance, connection).await;
            let Ok(updated_user_data) = update_user_balance else{

                let err_resp = update_user_balance.unwrap_err();
                return Err(err_resp);
                
            };

            res /* contains updated nft in ok part */

        } else{
            return Err(res.unwrap_err()); /* the error part is an http response */
        }
        

    }

}