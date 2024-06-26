


 
use actix::Addr;
use crate::adapters::nftport::OnchainContracts;
use crate::*;
use crate::adapters::nftport::{self, NftExt, OnchainNfts};
use crate::constants::{GALLERY_NOT_OWNED_BY, NFT_NOT_OWNED_BY, NFT_UPLOAD_PATH, INVALID_QUERY_LIMIT, STORAGE_IO_ERROR_CODE, NFT_ONCHAINID_NOT_FOUND, NFT_UPLOAD_ISSUE, CANT_MINT_CARD, CANT_MINT_NFT, CANT_TRANSFER_NFT, NFT_EVENT_TYPE_RECIPIENT_IS_NEEDED, NFT_EVENT_TYPE_METADATA_URI_IS_NEEDED, INVALID_NFT_EVENT_TYPE, NFT_IS_NOT_MINTED_YET, CANT_UPDATE_NFT, NFT_NOT_FOUND_OF, NFT_IS_ALREADY_MINTED, NFT_IS_NOT_LISTED_YET, NFT_PRICE_IS_EMPTY, NFT_EVENT_TYPE_BUYER_IS_NEEDED, CALLER_IS_NOT_BUYER, INVALID_NFT_ROYALTY, INVALID_NFT_PRICE, RECIPIENT_SCREEN_CID_NOT_FOUND, EMPTY_NFT_IMG, NFT_NOT_FOUND_OF_ID, USER_SCREEN_CID_NOT_FOUND, NFT_METADATA_URI_IS_EMPTY, NFT_IS_NOT_LISTED, NOT_FOUND_NFT, NFT_IS_NOT_OWNED_BY_THE_PASSED_IN_OWNER};
use crate::events::publishers::action::{SingleUserNotif, NotifData, ActionType};
use crate::helpers::misc::{Response, Limit};
use crate::schema::users_nfts::dsl::*;
use crate::schema::users_nfts;
use self::constants::{APP_NAME, NO_NFT_FOUND_IN_COLLECTION, NFT_MINT_LOCK};
use self::schema::nfts_comments;

use super::nfts_comments::NewNftCommentRequest;
use super::nfts_likes::NewNftLikeRequest;
use super::users::{User, UserData, UserWalletInfoResponse};
use super::users_collections::{UserCollection, UserCollectionData, UpdateUserCollection};
use super::users_fans::{UserFan, FriendData};
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
    pub nft_id: i32,
    pub content: String,
    pub owner_screen_cid: String,
    pub owner_username: String,
    pub owner_avatar: Option<String>,
    pub published_at: i64,
} 

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NftLike{
    pub nft_id: i32,
    pub upvoter_screen_cids: Vec<LikeUserInfo>,
    pub downvoter_screen_cids: Vec<LikeUserInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NftUpvoterLikes{
    pub id: i32,
    pub upvoter_screen_cids: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LikeUserInfo{
    pub screen_cid: String,
    pub username: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftColInfo{
    pub col_data: UserCollectionDataGeneralInfo,
    pub nfts_data: UserNftData
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct OnchainNftColInfo{
    pub col_data: UserCollectionDataGeneralInfo,
    pub nfts_data: serde_json::Value
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserCollectionDataGeneralInfo{
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

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct UserNftDataWithWalletInfoAndCollectionData{
    pub id: i32,
    pub contract_address: String,
    pub collection_data: serde_json::Value,
    pub current_owner_wallet_info: UserWalletInfoResponse,
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
pub struct UserLikeStat{
    pub nft_id: i32,
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

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct UserNftDataWithWalletInfo{
    pub id: i32,
    pub contract_address: String,
    pub current_owner_wallet_info: UserWalletInfoResponse,
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
    pub col_id: i32,
    pub buyer_screen_cid: Option<String>,
    pub transfer_to_screen_cid: Option<String>,
    pub amount: i64, // amount of gas fee for this call
    pub nft_id: i32,
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
    pub col_id: i32,
    pub contract_address: String,
    pub nft_name: String,
    pub nft_description: String,
    pub current_price: i64,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub attributes: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NewUserGiftCardNftRequest{
    pub caller_cid: String,
    pub amount: i64,
    pub col_id: i32,
    pub onchain_id: String,
    pub owner_screen_cid: String,
    pub contract_address: String,
    pub nft_name: String,
    pub nft_description: String,
    pub current_price: i64,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub metadata_uri: String,
    pub attributes: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub mint_tx_hash: String,
    pub tx_signature: String,
    pub hash_data: String,
}

/* 
    all fields must be String in order to create the json value 
    then we'll map some fields into their own type  
*/
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateNftMetadataUriRequest{
    pub caller_cid: String,
    pub amount: String,
    pub col_id: String,
    pub nft_id: String,
    pub nft_new_attributes: String,
    pub nft_new_extra: String,
    pub nft_new_name: String,
    pub nft_new_description: String,
    pub tx_signature: String,
    pub hash_data: String,
}


#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NewUserNftRequestString{
    pub caller_cid: String,
    pub amount: String,
    pub col_id: String,
    pub contract_address: String,
    pub nft_name: String,
    pub nft_description: String,
    pub current_price: String,
    pub extra: String, /* pg key, value based json binary object */
    pub attributes: String, /* pg key, value based json binary object */
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
        walletreq::evm::get_keccak256_from(self.caller_cid.clone())
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

    /* the transfer process can be either a buy or raw transfer */
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
    pub nft_name: String,
    pub nft_description: String,
    pub metadata_uri: String,
    pub current_price: i64,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub attributes: Option<serde_json::Value>, /* pg key, value based json binary object */
}

#[derive(Insertable)]
#[diesel(table_name=users_nfts)]
pub struct InsertNewUserGiftCardNftRequest{
    pub contract_address: String,
    pub onchain_id: String,
    pub current_owner_screen_cid: String,
    pub nft_name: String,
    pub nft_description: String,
    pub metadata_uri: String,
    pub current_price: i64,
    pub tx_hash: String,
    pub is_minted: Option<bool>,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub attributes: Option<serde_json::Value>, /* pg key, value based json binary object */
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserReactionData{
    pub nft_metadata_uri: String,
    pub nft_id: Option<i32>,
    pub comments: Vec<NftComment>,
    pub likes: Vec<UserLikeStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NftReactionData{
    pub nft_metadata_uri: String,
    pub nft_id: Option<i32>,
    pub nft_created_at: String,
    pub comments: Vec<NftComment>,
    pub likes: Vec<NftLike>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AddReactionRequest{
    pub caller_cid: String,
    pub col_id: i32,
    pub nft_id: i32,
    pub reaction_type: String, // comment or like or dislike
    pub comment_content: Option<String>,
    pub is_like_upvote: Option<bool>,
    pub is_like_downvote: Option<bool>,
    pub tx_signature: String,
    pub hash_data: String
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct NftOwnerCount{
    pub owner_wallet_info: UserWalletInfoResponse,
    pub minted_ones: usize,
    pub none_minted_ones: usize
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserNft{

    pub async fn update_nft_reactions_with_this_user(latest_user_info: UserData,
        connection: &mut DbPoolConnection) 
        -> Result<Vec<UserNft>, String>{

        match users_nfts
            .order(users_nfts::created_at.desc())
            .load::<UserNft>(connection)
            {
                Ok(mut nfts_) => {

                    let mut updated_nfts = vec![];
                    for nft in &mut nfts_{

                        let nft_comments = nft.construct_comments(connection);
                        let mut decoded_comments = if nft_comments.is_some(){
                            serde_json::from_value::<Vec<NftComment>>(nft_comments.clone().unwrap()).unwrap()
                        } else{
                            vec![]
                        };

                        let nft_likes = nft.construct_likes(connection);
                        let mut decoded_likes = if nft_likes.is_some(){
                            serde_json::from_value::<Vec<NftLike>>(nft_likes.clone().unwrap()).unwrap()
                        } else{
                            vec![]
                        };
                        
                        /* 
                            since we're taking a mutable pointer to decoded_comments
                            so by mutating an element of &mut decoded_comments the
                            decoded_comments itself will be mutated too
                        */
                        for comment in &mut decoded_comments{

                            if comment.owner_screen_cid == latest_user_info.clone().screen_cid.unwrap_or(String::from("")){

                                comment.owner_avatar = latest_user_info.clone().avatar;
                                comment.owner_username = latest_user_info.clone().username;
                            }
                        }

                        /* 
                            since we're taking a mutable pointer to decoded_likes
                            so by mutating an element of &mut decoded_likes the
                            decoded_likes itself will be mutated too
                        */
                        for like in &mut decoded_likes{

                            let mut downvoters = like.clone().downvoter_screen_cids;
                            for voter in &mut downvoters{
                                if voter.screen_cid == latest_user_info.clone().screen_cid.unwrap_or(String::from("")){
                                    voter.username = latest_user_info.clone().username;
                                    voter.avatar = latest_user_info.clone().avatar;
                                }
                            }
                            like.downvoter_screen_cids = downvoters;

                            let mut upvoters = like.clone().upvoter_screen_cids;
                            for voter in &mut upvoters{
                                if voter.screen_cid == latest_user_info.clone().screen_cid.unwrap_or(String::from("")){
                                    voter.username = latest_user_info.clone().username;
                                    voter.avatar = latest_user_info.clone().avatar;
                                }
                            }
                            like.upvoter_screen_cids = upvoters;
                        }

                        // update nft 
                        let _ = match diesel::update(users_nfts.find(nft.id))
                            .set((
                                comments.eq(
                                    serde_json::to_value(decoded_comments).unwrap()
                                ), 
                                likes.eq(
                                    serde_json::to_value(decoded_likes).unwrap()
                                )
                            ))
                            .returning(UserNft::as_returning())
                            .get_result::<UserNft>(connection)
                            {
                                Ok(fetched_nft_data) => {
                                    updated_nfts.push(fetched_nft_data);
                                },
                                Err(e) => {

                                    let resp_err = &e.to_string();

                                    /* custom error handler */
                                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                    
                                    let error_content = &e.to_string();
                                    let error_content = error_content.as_bytes().to_vec();  
                                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::update_nft_reactions_with_this_user");
                                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                                }
                            };


                    }

                    Ok(
                        updated_nfts
                    )

                },
                Err(e) => {
    
                    let resp_err = &e.to_string();
    
    
                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::update_nft_reactions_with_this_user");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    Err(resp_err.to_owned())
    
                }
            }

    }

    pub async fn get_all_nfts_owned_by(caller_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) -> Result<OnchainNfts, PanelHttpResponse>{

        
        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

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

        if to > 50 {
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: "Exceded 50",
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        Ok(
            nftport::get_nfts_owned_by(caller_screen_cid, from, to, connection).await
        )
        

    }

    pub async fn get_all_collections_owned_by(caller_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) -> Result<OnchainContracts, PanelHttpResponse>{

        
        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

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

        if to > 50 {
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: "Exceded 50",
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        Ok(
            nftport::get_contracts_owned_by(caller_screen_cid, from, to, connection).await
        )
        

    }

    pub async fn get_all_nft_reactions(nft_id: i32, 
        connection: &mut DbPoolConnection) 
        -> Result<NftReactionData, PanelHttpResponse>{
        
        let get_nft = users_nfts
            .filter(users_nfts::id.eq(nft_id))
            .first::<UserNft>(connection);
        
        
        let Ok(mut nft) = get_nft else{
            let resp = Response::<i32>{
                data: Some(nft_id),
                message: NFT_ONCHAINID_NOT_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };


        let nft_comments = nft.construct_comments(connection);
        let decoded_comments = if nft_comments.is_some(){
            serde_json::from_value::<Vec<NftComment>>(nft_comments.clone().unwrap()).unwrap()
        } else{
            vec![]
        };

        let nft_likes = nft.construct_likes(connection);
        let decoded_likes = if nft_likes.is_some(){
            serde_json::from_value::<Vec<NftLike>>(nft_likes.clone().unwrap()).unwrap()
        } else{
            vec![]
        };
        
        
        let mut this_nft_comments = vec![];
        for comment in decoded_comments{
            if comment.nft_id == nft_id{
                this_nft_comments.push(comment);
            }
        } 

        let mut this_nft_likes = vec![];
        for like in decoded_likes{
            if like.nft_id == nft_id{
                this_nft_likes.push(like);
            }
        } 
        

        Ok(
            NftReactionData{ 
                comments: this_nft_comments, 
                likes: this_nft_likes,
                nft_metadata_uri: nft.metadata_uri,
                nft_id: Some(nft.id),
                nft_created_at: nft.created_at.to_string(),
            }
        )

    }

    pub async fn get_all_by_current_owner(current_owner: &str, 
        connection: &mut DbPoolConnection) 
        -> Result<Vec<UserNftData>, PanelHttpResponse>{

        /* get all nfts owned by the passed in current_owner */
        let user_nfts = users_nfts
            .filter(users_nfts::current_owner_screen_cid.eq(current_owner))
            .load::<UserNft>(connection);

        let Ok(all_nfts) = user_nfts else{

            let resp = Response{
                data: Some(current_owner),
                message: NFT_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            )

        };

        Ok(
            all_nfts
                .into_iter()
                .map(|mut nft|{
    
                    UserNftData{ 
                        id: nft.id, 
                        contract_address: nft.clone().contract_address, 
                        current_owner_screen_cid: nft.clone().current_owner_screen_cid, 
                        metadata_uri: nft.clone().metadata_uri, 
                        extra: nft.clone().extra, 
                        onchain_id: nft.clone().onchain_id, 
                        nft_name: nft.clone().nft_name, 
                        is_minted: nft.clone().is_minted, 
                        nft_description: nft.clone().nft_description, 
                        current_price: nft.clone().current_price, 
                        is_listed: nft.clone().is_listed, 
                        freeze_metadata: nft.clone().freeze_metadata, 
                        comments: nft.clone().construct_comments(connection), 
                        likes: nft.clone().construct_likes(connection),  
                        tx_hash: nft.clone().tx_hash, 
                        created_at: nft.clone().created_at.to_string(), 
                        updated_at: nft.clone().updated_at.to_string(),
                        attributes: nft.clone().attributes, 
                    }
    
                })
                .collect::<Vec<UserNftData>>()
        )

    }

    pub fn get_all_inside_contract_none_async(col_addr: &str, 
        connection: &mut DbPoolConnection) 
        -> Result<Vec<UserNftData>, PanelHttpResponse>{

        /* get all nfts inside the passed in contract address */
        let user_nfts = users_nfts
            .filter(users_nfts::contract_address.eq(col_addr))
            .load::<UserNft>(connection);

        let Ok(all_nfts) = user_nfts else{

            let resp = Response{
                data: Some(col_addr),
                message: NO_NFT_FOUND_IN_COLLECTION,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        Ok(
            all_nfts
                .into_iter()
                .map(|mut nft|{
    
                    UserNftData{ 
                        id: nft.id, 
                        contract_address: nft.clone().contract_address, 
                        current_owner_screen_cid: nft.clone().current_owner_screen_cid, 
                        metadata_uri: nft.clone().metadata_uri, 
                        extra: nft.clone().extra, 
                        onchain_id: nft.clone().onchain_id, 
                        nft_name: nft.clone().nft_name, 
                        is_minted: nft.clone().is_minted, 
                        nft_description: nft.clone().nft_description, 
                        current_price: nft.clone().current_price, 
                        is_listed: nft.clone().is_listed, 
                        freeze_metadata: nft.clone().freeze_metadata, 
                        comments: nft.clone().construct_comments(connection), 
                        likes: nft.clone().construct_likes(connection),  
                        tx_hash: nft.clone().tx_hash, 
                        created_at: nft.clone().created_at.to_string(), 
                        updated_at: nft.clone().updated_at.to_string(),
                        attributes: nft.clone().attributes, 
                    }
    
                })
                .collect::<Vec<UserNftData>>()
        )

    }

    pub async fn get_owners_with_lots_of_nfts(owners: Vec<UserData>,
        connection: &mut DbPoolConnection) 
        -> Result<Vec<NftOwnerCount>, PanelHttpResponse>{
        
        let mut nft_owner_map = vec![];
        for owner in owners{

            if owner.screen_cid.is_none() || owner.screen_cid.clone().unwrap().is_empty(){
                continue;
            }
            
            let owner_screen_cid_ = owner.screen_cid.unwrap_or_default();
            let get_all_nfts_owned_by = UserNft::get_all_by_current_owner(&owner_screen_cid_, connection).await;
            let nfts_owned_by = if get_all_nfts_owned_by.is_ok(){
                get_all_nfts_owned_by.unwrap()
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

            let mut minted_ones = nfts_owned_by.clone()
                .into_iter()
                .map(|nft|{
                    if nft.is_minted.is_some() && nft.is_minted.unwrap(){
                        Some(nft)
                    } else{
                        None
                    }
                })
                .collect::<Vec<Option<UserNftData>>>();
            
            minted_ones.retain(|nft| nft.is_some());

            nft_owner_map.push(
                NftOwnerCount{
                    owner_wallet_info: user_wallet_info,
                    minted_ones: minted_ones.len(),
                    none_minted_ones: nfts_owned_by.len() - minted_ones.len(),
                }
            )
        }

        nft_owner_map.sort_by(|no1, no2|{

            let no1_count = no1.minted_ones;
            let no2_count = no2.minted_ones;

            no2_count.cmp(&no1_count)

        });
        
        Ok(nft_owner_map)

    } 

    pub async fn get_all(connection: &mut DbPoolConnection) 
        -> Result<Vec<UserNftData>, PanelHttpResponse>{


        /* get all nfts owned by the passed in current_owner */
        let user_nfts = users_nfts
            .order(users_nfts::created_at.desc())
            .load::<UserNft>(connection);

        let Ok(all_nfts) = user_nfts else{

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: NOT_FOUND_NFT,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        Ok(
            all_nfts
                .into_iter()
                .map(|mut nft|{
    
                    UserNftData{ 
                        id: nft.id, 
                        contract_address: nft.clone().contract_address, 
                        current_owner_screen_cid: nft.clone().current_owner_screen_cid, 
                        metadata_uri: nft.clone().metadata_uri, 
                        extra: nft.clone().extra, 
                        onchain_id: nft.clone().onchain_id, 
                        nft_name: nft.clone().nft_name, 
                        is_minted: nft.clone().is_minted, 
                        nft_description: nft.clone().nft_description, 
                        current_price: nft.clone().current_price, 
                        is_listed: nft.clone().is_listed, 
                        freeze_metadata: nft.clone().freeze_metadata, 
                        comments: nft.clone().construct_comments(connection), 
                        likes: nft.clone().construct_likes(connection), 
                        tx_hash: nft.clone().tx_hash, 
                        created_at: nft.clone().created_at.to_string(), 
                        updated_at: nft.clone().updated_at.to_string(),
                        attributes: nft.clone().attributes, 
                    }
    
                })
                .collect::<Vec<UserNftData>>()
        )

    }

    pub async fn find_by_onchain_id(onchain_id_: &str, 
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{

        let user_nft = users_nfts
            .filter(users_nfts::onchain_id.eq(onchain_id_))
            .first::<UserNft>(connection);

        let Ok(mut nft) = user_nft else{

            let resp = Response{
                data: Some(onchain_id_),
                message: NFT_NOT_FOUND_OF,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        Ok(
            UserNftData{ 
                id: nft.id, 
                contract_address: nft.clone().contract_address, 
                current_owner_screen_cid: nft.clone().current_owner_screen_cid, 
                metadata_uri: nft.clone().metadata_uri, 
                extra: nft.clone().extra, 
                onchain_id: nft.clone().onchain_id, 
                nft_name: nft.clone().nft_name, 
                is_minted: nft.clone().is_minted, 
                nft_description: nft.clone().nft_description, 
                current_price: nft.clone().current_price, 
                is_listed: nft.clone().is_listed, 
                freeze_metadata: nft.clone().freeze_metadata, 
                comments: nft.clone().construct_comments(connection), 
                likes: nft.clone().construct_likes(connection), 
                tx_hash: nft.clone().tx_hash, 
                created_at: nft.clone().created_at.to_string(), 
                updated_at: nft.clone().updated_at.to_string(),
                attributes: nft.clone().attributes, 
            }
        )

    }

    pub fn find_by_onchain_id_none_async(onchain_id_: &str, 
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{

        let user_nft = users_nfts
            .filter(users_nfts::onchain_id.eq(onchain_id_))
            .first::<UserNft>(connection);

        let Ok(mut nft) = user_nft else{

            let resp = Response{
                data: Some(onchain_id_),
                message: NFT_NOT_FOUND_OF,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        Ok(
            UserNftData{ 
                id: nft.id, 
                contract_address: nft.clone().contract_address, 
                current_owner_screen_cid: nft.clone().current_owner_screen_cid, 
                metadata_uri: nft.clone().metadata_uri, 
                extra: nft.clone().extra, 
                onchain_id: nft.clone().onchain_id, 
                nft_name: nft.clone().nft_name, 
                is_minted: nft.clone().is_minted, 
                nft_description: nft.clone().nft_description, 
                current_price: nft.clone().current_price, 
                is_listed: nft.clone().is_listed, 
                freeze_metadata: nft.clone().freeze_metadata, 
                comments: nft.clone().construct_comments(connection), 
                likes: nft.clone().construct_likes(connection), 
                tx_hash: nft.clone().tx_hash, 
                created_at: nft.clone().created_at.to_string(), 
                updated_at: nft.clone().updated_at.to_string(),
                attributes: nft.clone().attributes, 
            }
        )

    }

    pub async fn create_nft_metadata_uri(
        asset_info: CreateNftMetadataUriRequest,
        files: HashMap<String, Vec<u8>>, // a map between filenames and their utf8 bytes
        redis_client: redis::Client,
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{

        let CreateNftMetadataUriRequest{ caller_cid, amount, nft_id, nft_new_attributes, nft_new_extra, nft_new_name, nft_new_description, tx_signature, hash_data, col_id }
            = asset_info;

        /* parse the string fields to their own type */
        let amount = amount.parse::<i64>().unwrap();
        let nft_id = nft_id.parse::<i32>().unwrap(); 
        let col_id = col_id.parse::<i32>().unwrap(); 

        /* 
            nft_new_attributes and nft_new_extra are already in json string form
            but their type must be Value or json, solution to this this is decoding
            the json string directly into json or Value not creating the Value from
            json string, in that case we have still the json string but as a Value!!
        */
        let nft_new_attributes = Some(serde_json::from_str::<serde_json::Value>(&nft_new_attributes).unwrap());
        let nft_new_extra = Some(serde_json::from_str::<serde_json::Value>(&nft_new_extra).unwrap());


        let caller_screen_cid = walletreq::evm::get_keccak256_from(caller_cid);
        let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
        let Ok(user) = get_user else{
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };

        let get_nft = UserNft::find_by_id(nft_id, connection).await;
        let Ok(nft_info) = get_nft else{
            let err_resp = get_nft.unwrap_err();
            return Err(err_resp);
        };

        if caller_screen_cid.clone() != nft_info.current_owner_screen_cid{
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: NFT_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            );
        }

        /* find a collection data with the passed in contract address */
        let get_collection = UserCollection::find_by_id(col_id, connection).await;
        let Ok(collection_data) = get_collection else{
            let err_resp = get_collection.unwrap_err();
            return Err(err_resp);
        };

        /* find a gallery data with the passed in owner screen address */
        let get_gallery = UserPrivateGallery::find_by_owner_and_collection_id(&collection_data.owner_screen_cid, col_id, redis_client.clone(), connection).await;
        let Ok(gallery_data) = get_gallery else{
            let err_resp = get_gallery.unwrap_err();
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
            );
        }


        let mut udpate_nft_request = UpdateUserNftRequest{ 
            caller_cid: user.cid.unwrap(), 
            buyer_screen_cid: None, 
            transfer_to_screen_cid: None, 
            amount, 
            event_type: String::from(""), 
            contract_address: nft_info.contract_address, 
            current_owner_screen_cid: nft_info.current_owner_screen_cid, 
            metadata_uri: nft_info.metadata_uri, 
            extra: nft_new_extra, 
            attributes: nft_new_attributes, 
            onchain_id: nft_info.onchain_id, 
            nft_name: nft_new_name, 
            is_minted: nft_info.is_minted, 
            nft_description: nft_new_description, 
            current_price: nft_info.current_price, 
            is_listed: nft_info.is_listed, 
            freeze_metadata: nft_info.freeze_metadata, 
            comments: nft_info.comments, 
            likes: nft_info.likes, 
            tx_hash: nft_info.tx_hash, 
            tx_signature, 
            hash_data,
            nft_id,
            col_id, 
        };

        /* start uploading nft onchain */
        let first_map = files.iter().next().unwrap(); /* we'll use the first one */
        let filename = first_map.0.to_owned();
        let img_bytes = first_map.1.to_owned();
        let get_nft_metadata_uri = 
            nftport::get_nft_onchain_metadata_uri(
                (filename, img_bytes),
                redis_client.clone(),
                udpate_nft_request.clone()
            ).await;
        
        let Ok((nft_metadata_uri, nft_img_path)) = get_nft_metadata_uri else{

            let err_resp = get_nft_metadata_uri.unwrap_err();
            return Err(err_resp);
        };

        let nft_img = format!("{}::{}", nft_metadata_uri, nft_img_path);
        udpate_nft_request.metadata_uri = nft_img;

        Self::update_nft(
            udpate_nft_request, 
            redis_client.clone(),
            redis_actor.clone(),
            connection
        ).await
        

    }

    pub async fn find_by_id(asset_id: i32, 
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{

        let user_nft = users_nfts
            .filter(users_nfts::id.eq(asset_id))
            .first::<UserNft>(connection);

        let Ok(mut nft) = user_nft else{

            let resp = Response{
                data: Some(asset_id),
                message: NFT_NOT_FOUND_OF_ID,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        Ok(
            UserNftData{ 
                id: nft.id, 
                contract_address: nft.clone().contract_address, 
                current_owner_screen_cid: nft.clone().current_owner_screen_cid, 
                metadata_uri: nft.clone().metadata_uri, 
                extra: nft.clone().extra, 
                onchain_id: nft.clone().onchain_id, 
                nft_name: nft.clone().nft_name, 
                is_minted: nft.clone().is_minted, 
                nft_description: nft.clone().nft_description, 
                current_price: nft.clone().current_price, 
                is_listed: nft.clone().is_listed, 
                freeze_metadata: nft.clone().freeze_metadata, 
                comments: nft.clone().construct_comments(connection), 
                likes: nft.clone().construct_likes(connection),  
                tx_hash: nft.clone().tx_hash, 
                created_at: nft.clone().created_at.to_string(), 
                updated_at: nft.clone().updated_at.to_string(),
                attributes: nft.clone().attributes, 
            }
        )

    }

    pub fn find_by_id_none_async(asset_id: i32, 
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{

        let user_nft = users_nfts
            .filter(users_nfts::id.eq(asset_id))
            .first::<UserNft>(connection);

        let Ok(mut nft) = user_nft else{

            let resp = Response{
                data: Some(asset_id),
                message: NFT_NOT_FOUND_OF_ID,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        Ok(
            UserNftData{ 
                id: nft.id, 
                contract_address: nft.clone().contract_address, 
                current_owner_screen_cid: nft.clone().current_owner_screen_cid, 
                metadata_uri: nft.clone().metadata_uri, 
                extra: nft.clone().extra, 
                onchain_id: nft.clone().onchain_id, 
                nft_name: nft.clone().nft_name, 
                is_minted: nft.clone().is_minted, 
                nft_description: nft.clone().nft_description, 
                current_price: nft.clone().current_price, 
                is_listed: nft.clone().is_listed, 
                freeze_metadata: nft.clone().freeze_metadata, 
                comments: nft.clone().construct_comments(connection), 
                likes: nft.clone().construct_likes(connection), 
                tx_hash: nft.clone().tx_hash, 
                created_at: nft.clone().created_at.to_string(), 
                updated_at: nft.clone().updated_at.to_string(),
                attributes: nft.clone().attributes, 
            }
        )

    }
    

}

impl UserNft{

    pub async fn store_gift_card(asset_info: NewUserGiftCardNftRequest,
        redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{
            

        let minter_screen_cid = asset_info.clone().owner_screen_cid;
        
        /* find a collection data with the passed in contract address */
        let get_collection = UserCollection::find_by_id(asset_info.col_id, connection).await;
        let Ok(collection_data) = get_collection else{
            let err_resp = get_collection.unwrap_err();
            return Err(err_resp);
        };

        let depositor_screen_cid = walletreq::evm::get_keccak256_from(asset_info.caller_cid);
        let get_user = User::find_by_screen_cid(&depositor_screen_cid, connection).await;
        let Ok(user) = get_user else{

            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };

        let new_insert_nft = InsertNewUserGiftCardNftRequest{
            contract_address: collection_data.clone().contract_address,
            onchain_id: asset_info.onchain_id,
            current_owner_screen_cid: asset_info.owner_screen_cid,
            nft_name: asset_info.nft_name,
            nft_description: asset_info.nft_description,
            current_price: asset_info.current_price,
            metadata_uri: asset_info.metadata_uri,
            extra: asset_info.extra,
            is_minted: Some(true),
            tx_hash: asset_info.mint_tx_hash,
            attributes: asset_info.attributes,
        };

        /* inserting new nft */
        match diesel::insert_into(users_nfts)
            .values(&new_insert_nft)
            .returning(UserNft::as_returning())
            .get_result::<UserNft>(connection)
            {
                Ok(mut fetched_nft_data) => {
                    
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
                        comments: fetched_nft_data.construct_comments(connection),
                        likes: fetched_nft_data.construct_likes(connection),
                        tx_hash: fetched_nft_data.clone().tx_hash,
                        created_at: fetched_nft_data.clone().created_at.to_string(),
                        updated_at: fetched_nft_data.clone().updated_at.to_string(),
                        attributes: fetched_nft_data.attributes,
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
                        username: String::from(APP_NAME),
                        avatar: None,
                        bio: None,
                        banner: None,
                        mail: None,
                        screen_cid: Some(minter_screen_cid),
                        extra: None,
                        stars: None,
                        created_at: String::from(""),
                    };
                    let user_notif_info = SingleUserNotif{
                        wallet_info: user_wallet_info,
                        notif: NotifData{
                            actioner_wallet_info,
                            fired_at: Some(chrono::Local::now().timestamp()),
                            action_type: ActionType::MintNft,
                            action_data: serde_json::to_value(user_nft_data.clone()).unwrap()
                        }
                    };
                    let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                    events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

                    Ok(
                        user_nft_data
                    )                  

                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::store_gift_card");
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

    pub async fn insert(asset_info: NewUserNftRequest,
        redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{
            
        /* find a collection data with the passed in contract address */
        let get_collection = UserCollection::find_by_id(asset_info.col_id, connection).await;
        let Ok(collection_data) = get_collection else{
            let err_resp = get_collection.unwrap_err();
            return Err(err_resp);
        };

        /* find a gallery data with the passed in owner screen address */
        let get_gallery = UserPrivateGallery::find_by_owner_and_collection_id(&collection_data.owner_screen_cid, asset_info.col_id, redis_client.clone(), connection).await;
        let Ok(gallery_data) = get_gallery else{
            let err_resp = get_gallery.unwrap_err();
            return Err(err_resp);
        };
        
        let caller_screen_cid = walletreq::evm::get_keccak256_from(asset_info.clone().caller_cid);
        if gallery_data.owner_screen_cid != caller_screen_cid{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: GALLERY_NOT_OWNED_BY,
                status: 403,
                is_error: true
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

        /* 
            update user balance frist, if anything goes wrong they can call us 
            to pay them back, actually this is the gas fee that they must be 
            charged for since we already have paid the fee when we did the 
            onchain process

            also we're charging the user in here instead of charging him in create metadata
            api since if anything goes wrong in the second api the nft metadata won't be created
            on the chain but it has been created in db
        */
        let new_balance = user.balance.unwrap() - asset_info.amount;
        let update_user_balance = User::update_balance(user.id, "CreateArtWork", "Debit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
        let Ok(updated_user_balance_data) = update_user_balance else{

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
            nft_name: asset_info.nft_name,
            nft_description: asset_info.nft_description,
            current_price: asset_info.current_price,
            metadata_uri: String::from(""), /* will update in create nft metadata uri api */
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

                    Ok(user_nft_data)

                },
                Err(e) => {

                    let new_balance = updated_user_balance_data.balance.unwrap() + asset_info.amount;
                    let update_user_balance = User::update_balance(user.id, "ErrorCreateArtWork", "Credit", new_balance, redis_client.clone(), redis_actor, connection).await;
                    let Ok(updated_user_balance_data) = update_user_balance else{

                        let err_resp = update_user_balance.unwrap_err();
                        return Err(err_resp);
                        
                    };

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::insert");
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

    pub async fn update_nft(
        udpate_nft_request: UpdateUserNftRequest, 
        redis_client: redis::Client,
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
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
        
        /* update nft */
        match diesel::update(users_nfts.find(udpate_nft_request.nft_id))
            .set(&update_nft_data)
            .returning(UserNft::as_returning())
            .get_result::<UserNft>(connection)
            {
                Ok(mut fetched_nft_data) => {
                    
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
                        comments: fetched_nft_data.construct_comments(connection),
                        likes: fetched_nft_data.construct_likes(connection),
                        tx_hash: fetched_nft_data.clone().tx_hash,
                        created_at: fetched_nft_data.clone().created_at.to_string(),
                        updated_at: fetched_nft_data.clone().updated_at.to_string(),
                        attributes: fetched_nft_data.attributes,
                    };

                    Ok(user_nft_data)

                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserNft::update_nft");
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

    pub fn construct_comments(&mut self, connection: &mut DbPoolConnection) -> Option<serde_json::Value>{

        let get_all_nft_comments = models::nfts_comments::NftComment::get_all_for_nft(self.id, connection);
        let all_comments = if get_all_nft_comments.is_ok(){
            get_all_nft_comments.unwrap()
        } else{
            vec![]
        };
            
        let mut this_nft_comments = vec![];   
        for comment in all_comments{
            this_nft_comments.push(
                NftComment{ 
                    nft_id: comment.nft_id, 
                    content: comment.content.clone(), 
                    owner_screen_cid: {
                        User::find_by_id_none_async(comment.user_id, connection).unwrap().screen_cid.unwrap_or_default()
                    }, 
                    published_at: comment.published_at.timestamp(),
                    owner_username: {
                        User::find_by_id_none_async(comment.user_id, connection).unwrap().username
                    },
                    owner_avatar: {
                        User::find_by_id_none_async(comment.user_id, connection).unwrap().avatar
                    },
                }
            )
        }

        Some(
            serde_json::to_value(&this_nft_comments).unwrap()
        )
        
    }

    pub fn construct_likes(&mut self, connection: &mut DbPoolConnection) -> Option<serde_json::Value>{

        let mut this_nft_likes = vec![];

        // get all upvote likes related to this nft
        let get_all_upvote_nft_likes = models::nfts_likes::NftLike::get_all_upvotes_for_nft(self.id, connection);
        let upvote_likes = if get_all_upvote_nft_likes.is_ok(){
            get_all_upvote_nft_likes.unwrap()
        } else{
            vec![]
        };

        // get all downvote likes related to this nft
        let get_all_downvote_nft_likes = models::nfts_likes::NftLike::get_all_downvotes_for_nft(self.id, connection);
        let downvote_likes = if get_all_downvote_nft_likes.is_ok(){
            get_all_downvote_nft_likes.unwrap()
        } else{
            vec![]
        };

        let mut upvote_users = vec![];
        for upvote_like in upvote_likes{
            upvote_users.push(
                LikeUserInfo{ 
                    screen_cid: {
                        User::find_by_id_none_async(upvote_like.user_id, connection).unwrap().screen_cid.unwrap_or_default()
                    }, 
                    username: {
                        User::find_by_id_none_async(upvote_like.user_id, connection).unwrap().username
                    }, 
                    avatar: {
                        User::find_by_id_none_async(upvote_like.user_id, connection).unwrap().avatar
                    }
                }
            )
        }

        let mut downvote_users = vec![];
        for downvote_like in downvote_likes{
            downvote_users.push(
                LikeUserInfo{ 
                    screen_cid: {
                        User::find_by_id_none_async(downvote_like.user_id, connection).unwrap().screen_cid.unwrap_or_default()
                    }, 
                    username: {
                        User::find_by_id_none_async(downvote_like.user_id, connection).unwrap().username
                    }, 
                    avatar: {
                        User::find_by_id_none_async(downvote_like.user_id, connection).unwrap().avatar
                    }
                }
            )
        }

        // single element
        this_nft_likes.push(
            NftLike{
                nft_id: self.id,
                upvoter_screen_cids: upvote_users,
                downvoter_screen_cids: downvote_users,
            }
        );

        Some(
            serde_json::to_value(&this_nft_likes).unwrap()
        )

    }

    pub async fn add_reaction_to_nft(add_reaction_request: AddReactionRequest, redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{
        
        let caller_screen_cid = walletreq::evm::get_keccak256_from(add_reaction_request.clone().caller_cid);

        let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
        let Ok(user) = get_user else{

            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };

        let get_nft = UserNft::find_by_id(add_reaction_request.nft_id, connection).await;
        let Ok(nft_data) = get_nft else{
            let err_resp = get_nft.unwrap_err();
            return Err(err_resp);
        };

        let get_nft_owner = User::find_by_screen_cid(&nft_data.current_owner_screen_cid, connection).await;
        let Ok(nft_owner) = get_nft_owner else{
            let err_resp = get_nft_owner.unwrap_err();
            return Err(err_resp);
        };

        /* find a collection data with the passed in contract address */
        let get_collection = UserCollection::find_by_id(add_reaction_request.col_id, connection).await;
        let Ok(collection_data) = get_collection else{
            let err_resp = get_collection.unwrap_err();
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
                let empty_comments: Vec<NftComment> = vec![];
                Some(
                    serde_json::to_value(&empty_comments).unwrap()
                )
            },
            likes:{
                let empty_likes: Vec<NftLike> = vec![];
                Some(
                    serde_json::to_value(&empty_likes).unwrap()
                )
            },
            tx_hash: nft_data.tx_hash,
            tx_signature: String::from(""),
            hash_data: String::from(""),
            attributes: nft_data.attributes,
            nft_id: nft_data.id,
            col_id: add_reaction_request.col_id,
        };

        // finally update the nft
        match Self::update_nft(
            updated_nft_data, 
            redis_client.clone(),
            redis_actor.clone(),
            connection
        ).await{
            Ok(updated_user_nft_data) => {

                let nft_data = UserNftData{
                    id: updated_user_nft_data.id,
                    contract_address: updated_user_nft_data.contract_address,
                    current_owner_screen_cid: updated_user_nft_data.current_owner_screen_cid,
                    metadata_uri: updated_user_nft_data.metadata_uri,
                    extra: updated_user_nft_data.extra,
                    attributes: updated_user_nft_data.attributes,
                    onchain_id: updated_user_nft_data.onchain_id,
                    nft_name: updated_user_nft_data.nft_name,
                    is_minted: updated_user_nft_data.is_minted,
                    nft_description: updated_user_nft_data.nft_description,
                    current_price: updated_user_nft_data.current_price,
                    is_listed: updated_user_nft_data.is_listed,
                    freeze_metadata: updated_user_nft_data.freeze_metadata,
                    comments: {
                         
                        let mut this_nft_comments = vec![];
                        if add_reaction_request.comment_content.is_some() && add_reaction_request.reaction_type == "comment"{

                            // first insert a new comment 
                            let get_inserted_comment = models::nfts_comments::NftComment::insert(
                                NewNftCommentRequest{
                                    user_id: user.id,
                                    nft_id: nft_data.id,
                                    content: add_reaction_request.clone().comment_content.unwrap(),
                                }, connection).await; 
                                
                            let Ok(inserted_comment) = get_inserted_comment else{
                                let err_resp = get_inserted_comment.unwrap_err();
                                return Err(err_resp);
                            };
                            
                            // then get all comments related to this nft
                            let get_all_nft_comments = models::nfts_comments::NftComment::get_all_for_nft(updated_user_nft_data.id, connection);
                            let all_comments = if get_all_nft_comments.is_ok(){
                                get_all_nft_comments.unwrap()
                            } else{
                                vec![]
                            };

                            for comment in all_comments{
                                this_nft_comments.push(
                                    NftComment{ 
                                        nft_id: comment.nft_id, 
                                        content: comment.content.clone(), 
                                        owner_screen_cid: {
                                            User::find_by_id(comment.user_id, connection).await.unwrap().screen_cid.unwrap_or_default()
                                        }, 
                                        published_at: comment.published_at.timestamp(),
                                        owner_username: {
                                            User::find_by_id(comment.user_id, connection).await.unwrap().username
                                        },
                                        owner_avatar: {
                                            User::find_by_id(comment.user_id, connection).await.unwrap().avatar
                                        },
                                    }
                                )
                            }
        
                        }
        
                        Some(serde_json::to_value(this_nft_comments).unwrap())
                    },
                    likes: {
                        
                        let mut this_nft_likes = vec![];

                        // get all upvote likes related to this nft
                        let get_all_upvote_nft_likes = models::nfts_likes::NftLike::get_all_upvotes_for_nft(updated_user_nft_data.id, connection);
                        let upvote_likes = if get_all_upvote_nft_likes.is_ok(){
                            get_all_upvote_nft_likes.unwrap()
                        } else{
                            vec![]
                        };

                        // get all downvote likes related to this nft
                        let get_all_downvote_nft_likes = models::nfts_likes::NftLike::get_all_downvotes_for_nft(updated_user_nft_data.id, connection);
                        let downvote_likes = if get_all_downvote_nft_likes.is_ok(){
                            get_all_downvote_nft_likes.unwrap()
                        } else{
                            vec![]
                        };

                        let mut upvote_users = vec![];
                        for upvote_like in upvote_likes{
                            upvote_users.push(
                                LikeUserInfo{ 
                                    screen_cid: {
                                        User::find_by_id(upvote_like.user_id, connection).await.unwrap().screen_cid.unwrap_or_default()
                                    }, 
                                    username: {
                                        User::find_by_id(upvote_like.user_id, connection).await.unwrap().username
                                    }, 
                                    avatar: {
                                        User::find_by_id(upvote_like.user_id, connection).await.unwrap().avatar
                                    }
                                }
                            )
                        }

                        let mut downvote_users = vec![];
                        for downvote_like in downvote_likes{
                            downvote_users.push(
                                LikeUserInfo{ 
                                    screen_cid: {
                                        User::find_by_id(downvote_like.user_id, connection).await.unwrap().screen_cid.unwrap_or_default()
                                    }, 
                                    username: {
                                        User::find_by_id(downvote_like.user_id, connection).await.unwrap().username
                                    }, 
                                    avatar: {
                                        User::find_by_id(downvote_like.user_id, connection).await.unwrap().avatar
                                    }
                                }
                            )
                        }

                        // single element
                        this_nft_likes.push(
                            NftLike{
                                nft_id: updated_user_nft_data.id,
                                upvoter_screen_cids: upvote_users,
                                downvoter_screen_cids: downvote_users,
                            }
                        );

                        // getting a mutable pointer to all likes fetched from db
                        let mutable_likes = &mut this_nft_likes;
                        let this_nft_position = mutable_likes
                            .iter()
                            .position(|nft| nft.nft_id == add_reaction_request.nft_id);
        
                        if add_reaction_request.is_like_upvote.is_some() && 
                            add_reaction_request.is_like_upvote.unwrap() == true &&
                            add_reaction_request.reaction_type == "like"{
        
                            if this_nft_position.is_some(){
                                /* 
                                    getting a mutable pointer to the found NftLike instance inside mutable_likes
                                    means that if we mutate the instance in other scopes the actual type inside 
                                    the vector will be mutated too
                                */
                                
                                let mut this_nft_likes = mutable_likes[this_nft_position.unwrap()].clone();
                                
                                let caller = caller_screen_cid.clone();
                                if this_nft_likes.clone().upvoter_screen_cids.is_empty(){
                                    this_nft_likes.upvoter_screen_cids.push(
                                        LikeUserInfo{ 
                                            screen_cid: caller.clone(), 
                                            username: user.clone().username, 
                                            avatar: user.clone().avatar 
                                        }
                                    );
                                }
                                
                                for upvote in this_nft_likes.clone().upvoter_screen_cids{
                                    if upvote.screen_cid != caller{
                                        this_nft_likes.upvoter_screen_cids.push(
                                            LikeUserInfo{ 
                                                screen_cid: caller.clone(), 
                                                username: user.clone().username, 
                                                avatar: user.clone().avatar 
                                            }
                                        );
                                    }
                                }
        
                                /* remove the caller from downvoters if there was any since he liked the nft */
                                if this_nft_likes.clone().downvoter_screen_cids
                                    .into_iter()
                                    .any(|u| u.screen_cid == caller_screen_cid.clone()){
        
                                        let downvoter_position_scid = this_nft_likes.downvoter_screen_cids.iter().position(|u| &u.screen_cid == &caller_screen_cid);
                                        if downvoter_position_scid.is_some(){
                                            this_nft_likes.downvoter_screen_cids.remove(downvoter_position_scid.unwrap());
                                        }
                                    }
        
                                mutable_likes[this_nft_position.unwrap()] = this_nft_likes;
        
                            } else{
        
                                mutable_likes.push(
                                    NftLike{ 
                                        nft_id: add_reaction_request.clone().nft_id, 
                                        upvoter_screen_cids: vec![
                                            LikeUserInfo{ 
                                                screen_cid: caller_screen_cid.clone(), 
                                                username: user.clone().username, 
                                                avatar: user.clone().avatar 
                                            }
                                        ],
                                        downvoter_screen_cids: vec![]
                                    }
                                )
                            }


                            // upsert like
                            let get_upserted_like_info = models::nfts_likes::NftLike::upsert(
                                NewNftLikeRequest{
                                    user_id: user.id,
                                    nft_id: nft_data.id,
                                    is_upvote: true,
                                }, connection).await;
                            
                            let Ok(upserted_like_info) = get_upserted_like_info else{
                                let err_resp = get_upserted_like_info.unwrap_err();
                                return Err(err_resp);
                            };
                
                        }
                
                        if add_reaction_request.is_like_downvote.is_some() && 
                            add_reaction_request.is_like_downvote.unwrap() == true &&
                            add_reaction_request.reaction_type == "dislike"{
                
                            if this_nft_position.is_some(){
                                /* 
                                    getting a mutable pointer to the found NftLike instance inside mutable_likes
                                    means that if we mutate the instance in other scopes the actual type inside 
                                    the vector will be mutated too
                                */
                                let mut this_nft_likes = mutable_likes[this_nft_position.unwrap()].clone();
        
                                let caller = caller_screen_cid.clone();
                                if this_nft_likes.clone().downvoter_screen_cids.is_empty(){
                                    this_nft_likes.downvoter_screen_cids.push(
                                        LikeUserInfo{ 
                                            screen_cid: caller.clone(), 
                                            username: user.clone().username, 
                                            avatar: user.clone().avatar 
                                        }
                                    );
                                }
        
                                for downvote in this_nft_likes.clone().downvoter_screen_cids{
                                    if downvote.screen_cid != caller{
                                        this_nft_likes.downvoter_screen_cids.push(
                                            LikeUserInfo{ 
                                                screen_cid: caller.clone(), 
                                                username: user.clone().username, 
                                                avatar: user.clone().avatar 
                                            }
                                        );
                                    }
                                }
        
                                /* remove the caller from upvoters if there was any since he disliked the nft */
                                if this_nft_likes.clone().upvoter_screen_cids
                                    .into_iter()
                                    .any(|u| u.screen_cid == caller_screen_cid.clone()){
        
                                        let upvoter_position_scid = this_nft_likes.upvoter_screen_cids.iter().position(|u| &u.screen_cid == &caller_screen_cid);
                                        if upvoter_position_scid.is_some(){
                                            this_nft_likes.upvoter_screen_cids.remove(upvoter_position_scid.unwrap());
                                        }
                                    }
        
                                
                                mutable_likes[this_nft_position.unwrap()] = this_nft_likes;
        
                            } else{
                                mutable_likes.push(
                                    NftLike{ 
                                        nft_id: add_reaction_request.nft_id, 
                                        upvoter_screen_cids: vec![],
                                        downvoter_screen_cids: vec![
                                            LikeUserInfo{ 
                                                screen_cid: caller_screen_cid.clone(), 
                                                username: user.clone().username, 
                                                avatar: user.clone().avatar 
                                            }
                                        ]
                                    }
                                )
                            }

                            // upsert like
                            let get_upserted_like_info = models::nfts_likes::NftLike::upsert(
                                NewNftLikeRequest{
                                    user_id: user.id,
                                    nft_id: nft_data.id,
                                    is_upvote: false,
                                }, connection).await;
                            
                            let Ok(upserted_like_info) = get_upserted_like_info else{
                                let err_resp = get_upserted_like_info.unwrap_err();
                                return Err(err_resp);
                            };
                        }
        
                        Some(serde_json::to_value(mutable_likes).unwrap())
                    },
                    tx_hash: updated_user_nft_data.tx_hash,
                    created_at: updated_user_nft_data.created_at,
                    updated_at: updated_user_nft_data.updated_at,
                };

                /** -------------------------------------------------------------------- */
                /** ----------------- publish new event data to `on_user_action` channel */
                /** -------------------------------------------------------------------- */
                let actioner_wallet_info = UserWalletInfoResponse{
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
                let user_wallet_info = UserWalletInfoResponse{
                    username: nft_owner.username,
                    avatar: nft_owner.avatar,
                    bio: nft_owner.bio,
                    banner: nft_owner.banner,
                    mail: nft_owner.mail,
                    screen_cid: nft_owner.screen_cid,
                    extra: nft_owner.extra,
                    stars: nft_owner.stars,
                    created_at: nft_owner.created_at.to_string(),
                };
                let user_notif_info = SingleUserNotif{
                    wallet_info: user_wallet_info,
                    notif: NotifData{
                        actioner_wallet_info,
                        fired_at: Some(chrono::Local::now().timestamp()),
                        action_type: match add_reaction_request.clone().reaction_type.as_str(){
                            "comment" => {
                                ActionType::CommentNft
                            },
                            "like" => {
                                ActionType::LikeNft
                            },
                            _ => {
                                ActionType::DislikeNft
                            }
                        },
                        action_data: serde_json::to_value(nft_data.clone()).unwrap()
                    }
                };
                let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

                Ok(nft_data)
            },
            Err(err) => Err(err)
        }

    }

    /*  --------------------------------------------------------------------------------
        Note: Transferring is possible only if the token is owned by the contract owner 
        and the token has not been transferred/sold yet, in other words nft can be transferred 
        once and it can be transferred if the token has been minted to to the contract owner so 
        ✅ following scenario is correct
        only owner: create nft, mint nft then list
        only other: buy
        ❌ but the following is not:
        owner: create nft
        other: mint then list 
        owner or other: buy
        means that only owner can transfer or list nft for sell and only once!
        --------------------------------------------------------------------------------
    */
    pub async fn buy_nft(mut buy_nft_request: UpdateUserNftRequest, redis_client: redis::Client,
        redis_actor: Addr<RedisActor>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserNftData, PanelHttpResponse>{

        
        if buy_nft_request.event_type.as_str() == "buy"{

            let caller_screen_cid = walletreq::evm::get_keccak256_from(buy_nft_request.clone().caller_cid);

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
            let get_collection = UserCollection::find_by_id(buy_nft_request.col_id, connection).await;
            let Ok(collection_data) = get_collection else{
                let err_resp = get_collection.unwrap_err();
                return Err(err_resp);
            };

            /* find a gallery data with the passed in owner screen address */
            let get_gallery = UserPrivateGallery::find_by_owner_and_collection_id(&collection_data.owner_screen_cid, buy_nft_request.col_id, redis_client.clone(), connection).await;
            let Ok(gallery_data) = get_gallery else{
                let err_resp = get_gallery.unwrap_err();
                return Err(err_resp);
            };

            /* if the onchain id wasn't found we simply terminate the caller */
            if nft_data.clone().is_minted.is_none() || 
                (nft_data.clone().is_minted.is_some() && nft_data.clone().is_minted.unwrap() == false) &&  
                nft_data.clone().onchain_id.is_none() || 
                (nft_data.clone().onchain_id.is_some() && nft_data.clone().onchain_id.unwrap().is_empty()){

                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: NFT_IS_NOT_MINTED_YET,
                    status: 406,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                ); 

            }

            if nft_data.is_listed.is_some() && nft_data.is_listed.unwrap() == true{

                
                if buy_nft_request.buyer_screen_cid.is_some(){
                    
                    let buyer_screen_cid = buy_nft_request.buyer_screen_cid.clone().unwrap();
                    let current_nft_owner_screen_cid = buy_nft_request.current_owner_screen_cid.clone();

                    if caller_screen_cid != buyer_screen_cid{
                        
                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: CALLER_IS_NOT_BUYER,
                            status: 406,
                            is_error: true
                        };
                        return Err(
                            Ok(HttpResponse::NotAcceptable().json(resp))
                        );
                    }

                    if nft_data.current_owner_screen_cid != current_nft_owner_screen_cid{
                        
                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: NFT_IS_NOT_OWNED_BY_THE_PASSED_IN_OWNER,
                            status: 406,
                            is_error: true
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
                            is_error: true
                        };
                        return Err(
                            Ok(HttpResponse::NotAcceptable().json(resp))
                        );
                        
                    }

                    if get_nft_price.is_some() && get_nft_price.unwrap() < 0 || get_nft_price.unwrap() == 0 ||
                        (nft_data.current_price.is_some() && nft_data.current_price.unwrap() != get_nft_price.unwrap()){

                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: INVALID_NFT_PRICE,
                            status: 406,
                            is_error: true
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
                        the collection_data instance will be moved, we should either clone it or 
                        borrow it using ref, & or as_ref()
                    */
                    /* ---------------------------------------------------------------------------- */
                    let nft_price = get_nft_price.unwrap() as f64;
                    let royalty_bps = (royalty as f64 / 10000.0) as f64; // 100 basis points equal 1%
                    let percent = percentage::Percentage::from_decimal(royalty_bps);
                    let royalty_amount = percent.apply_to(nft_price);

                    if royalty_amount > nft_price{

                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: INVALID_NFT_ROYALTY,
                            status: 406,
                            is_error: true
                        };
                        return Err(
                            Ok(HttpResponse::NotAcceptable().json(resp))
                        );

                    }

                    let pay_to_seller = (nft_price - royalty_amount.round()) as i64;

                    let lock_ids = NFT_BUY_LOCK.clone();
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<bool>(1024);
                    // first spawn acquire the lock to push the product id
                    tokio::spawn(
                        {
                            let nft_data = nft_data.clone();
                            let tx = tx.clone();
                            let lock_ids = lock_ids.clone();
                            async move{
                                let mut write_lock = lock_ids.lock().await;
                                if (*write_lock).contains(&nft_data.id){
                                    // reject the request
                                    tx.send(true).await;
                                } else{
                                    (*write_lock).push(nft_data.id); // save the id for later readers to reject their request during the minting process
                                }

                            }
                        }
                    );

                    /* distribute shares */
                    let get_updated_user = Self::distributed_shares(
                        user.clone(),
                        seller_info.clone(),
                        royalty_owner_info.clone(),
                        buy_nft_request.amount,
                        nft_price as i64,
                        pay_to_seller as i64,
                        royalty_amount as i64,
                        redis_actor.clone(),
                        redis_client.clone(),
                        connection
                    ).await;
                    let Ok(updated_user) = get_updated_user else{

                        let lock_ids = lock_ids.clone();
                        tokio::spawn(async move{
                            let mut write_lock = lock_ids.lock().await;
                            (*write_lock).retain(|&nft_id| nft_id != nft_data.id);
                        });

                        let err_resp = get_updated_user.unwrap_err();
                        return Err(err_resp);
                    };


                    /* ----------------------------------------------------- */
                    /* ------- transferring the ownership of the nft ------- */
                    /* ----------------------------------------------------- */
                    // note that this can only be done once after nft minting

                    let cloned_redis_client = redis_client.clone();
                    let cloned_buy_nft_request = buy_nft_request.clone();
                    let (resptran_sender, mut resptran_receiver) = tokio::sync::mpsc::channel::<(String, u8)>(1024);
                    let tran_task = tokio::spawn(async move{
                        
                        let (new_tx_hash, status) = 
                            nftport::transfer_nft(
                                cloned_redis_client.clone(), 
                                cloned_buy_nft_request.clone()
                            ).await;
                        
                        resptran_sender.send((new_tx_hash, status)).await;

                    });


                    tokio::select!{
                        Some(flag) = rx.recv() => {
                            let resp = Response::<'_, &[u8]>{
                                data: Some(&[]),
                                message: &format!("NFT Is Being Bought By Another User"),
                                status: 406,
                                is_error: true
                            };
                            return Err(
                                Ok(HttpResponse::NotAcceptable().json(resp))
                            ); 
                        },
                        _ = tran_task => {

                            let mut get_tx_hash = String::from("");
                            let mut get_status = 0;
                            while let Some((new_tx_hash, status)) = resptran_receiver.recv().await{
                                get_tx_hash = new_tx_hash;
                                get_status = status;
                            }

                            // if anything went wrong we simpley revert the shares
                            if get_status == 1{
        
                                let lock_ids = lock_ids.clone();
                                tokio::spawn(async move{
                                    let mut write_lock = lock_ids.lock().await;
                                    (*write_lock).retain(|&nft_id| nft_id != nft_data.id);
                                });
        
                                /* revert shares */
                                let get_updated_user = Self::revert_shares(
                                    User::find_by_screen_cid(&caller_screen_cid, connection).await.unwrap(),
                                    User::find_by_screen_cid(&seller, connection).await.unwrap(),
                                    User::find_by_screen_cid(&royalty_owner, connection).await.unwrap(),
                                    buy_nft_request.amount,
                                    nft_price as i64,
                                    pay_to_seller as i64,
                                    royalty_amount as i64,
                                    redis_actor,
                                    redis_client.clone(),
                                    connection
                                ).await;
                                let Ok(updated_user) = get_updated_user else{
        
                                    let err_resp = get_updated_user.unwrap_err();
                                    return Err(err_resp);
                                };
        
                                let resp = Response::<'_, &[u8]>{
                                    data: Some(&[]),
                                    message: CANT_TRANSFER_NFT, /* maybe: can't transfer nft more than once */
                                    status: 417,
                                    is_error: true
                                };
                                return Err(
                                    Ok(HttpResponse::ExpectationFailed().json(resp))
                                );
        
                            }
        
        
                            // updating nft fields with new onchain data
                            buy_nft_request.tx_hash = Some(get_tx_hash);
                            buy_nft_request.current_owner_screen_cid = buyer_screen_cid;
                            buy_nft_request.is_listed = Some(false);
                            buy_nft_request.current_price = Some(0);
        
                            // release the lock after transferring
                            // (*lock_ids).retain(|&nft_id| nft_id != nft_data.id);
        
                            match Self::update_nft(
                                buy_nft_request.clone(), 
                                redis_client.clone(),
                                redis_actor.clone(),
                                connection).await{
        
                                    Ok(updated_user_nft_data) => {
        
                                        let get_nft_owner = User::find_by_screen_cid(&nft_data.current_owner_screen_cid, connection).await;
                                        let Ok(nft_owner) = get_nft_owner else{
                                            let err_resp = get_nft_owner.unwrap_err();
                                            return Err(err_resp);
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
                                            username: nft_owner.username,
                                            avatar: nft_owner.avatar,
                                            bio: nft_owner.bio,
                                            banner: nft_owner.banner,
                                            mail: nft_owner.mail,
                                            screen_cid: nft_owner.screen_cid,
                                            extra: nft_owner.extra,
                                            stars: nft_owner.stars,
                                            created_at: nft_owner.created_at.to_string(),
                                        };
                                        let user_notif_info = SingleUserNotif{
                                            wallet_info: user_wallet_info,
                                            notif: NotifData{
                                                actioner_wallet_info,
                                                fired_at: Some(chrono::Local::now().timestamp()),
                                                action_type: ActionType::BuyNft,
                                                action_data: serde_json::to_value(updated_user_nft_data.clone()).unwrap()
                                            }
                                        };
                                        let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                                        events::publishers::user::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;
        
                                        Ok(updated_user_nft_data)
                                    },
                                    Err(err) => Err(err)
                                }


                        }
                    }
                    
                } else{
                    
                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_EVENT_TYPE_BUYER_IS_NEEDED,
                        status: 406,
                        is_error: true
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
                    is_error: true
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
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

    }

    pub async fn distributed_shares(
        user: User, 
        seller_info: User, 
        royalty_owner_info: User,
        gas_fee: i64,
        nft_price: i64,
        pay_to_seller: i64,
        royalty_amount: i64,
        redis_actor: Addr<RedisActor>,
        redis_client: redis::Client,
        connection: &mut DbPoolConnection) 
        -> Result<(), PanelHttpResponse>{

        /* --------------------------------------------- */
        /* -------------- update balances -------------- */
        /* --------------------------------------------- */
        /* update buyer balance (nft price + onchain gas fee) */
        let new_balance = user.balance.unwrap() - (nft_price as i64 + gas_fee);
        let update_user_balance = User::update_balance(user.id, "Royalty", "Credit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        /* update seller balance */
        let new_balance = seller_info.balance.unwrap() + pay_to_seller as i64;
        let update_user_balance = User::update_balance(seller_info.id, "Royalty", "Credit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        /* update royalty owner balance */
        let new_balance = royalty_owner_info.balance.unwrap() + royalty_amount as i64;
        let update_user_balance = User::update_balance(royalty_owner_info.id, "Royalty", "Credit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        Ok(())

    }

    pub async fn revert_shares(
        user: User, 
        seller_info: User, 
        royalty_owner_info: User,
        gas_fee: i64,
        nft_price: i64,
        pay_to_seller: i64,
        royalty_amount: i64,
        redis_actor: Addr<RedisActor>,
        redis_client: redis::Client,
        connection: &mut DbPoolConnection) 
        -> Result<(), PanelHttpResponse>{

        /* --------------------------------------------- */
        /* -------------- update balances -------------- */
        /* --------------------------------------------- */
        /* update buyer balance (nft price + onchain gas fee) */
        let new_balance = user.balance.unwrap() + (nft_price as i64 + gas_fee);
        let update_user_balance = User::update_balance(user.id, "ErrorRoyalty", "Debit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        /* update seller balance */
        let new_balance = seller_info.balance.unwrap() - pay_to_seller as i64;
        let update_user_balance = User::update_balance(seller_info.id, "ErrorRoyalty", "Debit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        /* update royalty owner balance */
        let new_balance = royalty_owner_info.balance.unwrap() - royalty_amount as i64;
        let update_user_balance = User::update_balance(royalty_owner_info.id, "ErrorRoyalty", "Debit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
        let Ok(updated_user_data) = update_user_balance else{

            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
            
        };

        Ok(())

    }

    pub async fn mint_nft(mut mint_nft_request: UpdateUserNftRequest, redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{
        
        if mint_nft_request.event_type.as_str() == "mint"{

            let get_nft = UserNft::find_by_id(mint_nft_request.nft_id, connection).await;
            let Ok(nft_data) = get_nft else{
                let err_resp = get_nft.unwrap_err();
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
                        is_error: false
                    };
                    return Err(
                        Ok(HttpResponse::Found().json(resp))
                    );

                }

            let caller_screen_cid = walletreq::evm::get_keccak256_from(mint_nft_request.clone().caller_cid);

            let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
            let Ok(user) = get_user else{

                let err_resp = get_user.unwrap_err();
                return Err(err_resp);
            };

            let get_nft = UserNft::find_by_id(mint_nft_request.nft_id, connection).await;
            let Ok(nft_data) = get_nft else{
                let err_resp = get_nft.unwrap_err();
                return Err(err_resp);
            };

            let get_nft_owner = User::find_by_screen_cid(&nft_data.current_owner_screen_cid, connection).await;
            let Ok(nft_owner) = get_nft_owner else{
                let err_resp = get_nft_owner.unwrap_err();
                return Err(err_resp);
            };

            /* find a collection data with the passed in contract address */
            let get_collection = UserCollection::find_by_id(mint_nft_request.col_id, connection).await;
            let Ok(collection_data) = get_collection else{
                let err_resp = get_collection.unwrap_err();
                return Err(err_resp);
            };

            /* find a gallery data with the passed in owner screen address */
            let get_gallery = UserPrivateGallery::find_by_owner_and_collection_id(&collection_data.owner_screen_cid, mint_nft_request.col_id, redis_client.clone(), connection).await;
            let Ok(gallery_data) = get_gallery else{
                let err_resp = get_gallery.unwrap_err();
                return Err(err_resp);
            };

                if mint_nft_request.metadata_uri.is_empty(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_EVENT_TYPE_METADATA_URI_IS_NEEDED,
                        status: 406,
                        is_error: true
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
                        is_error: true
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
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    );
                    
                }

                if get_nft_price.is_some() && get_nft_price.unwrap() < 0 || get_nft_price.unwrap() == 0 ||
                    (nft_data.current_price.is_some() && nft_data.current_price.unwrap() != get_nft_price.unwrap()){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: INVALID_NFT_PRICE,
                        status: 406,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }
                let nft_price = get_nft_price.unwrap();

                let mut minter_is_owner = false;
                if nft_data.current_owner_screen_cid == caller_screen_cid{
                    minter_is_owner = true;
                }

                let mut uubd = None::<UserData>;
                let mut owner_uubd = None::<UserData>;
                if !minter_is_owner{
                    // charge user before minting then if minting goes wrong we'll payback the user
                    // ...
                    
                    /* 
                        update minter balance (nft price + onchain gas fee) 
                        do this before minting the nft since the nft might 
                        gets minted on chain but after that user has not enough 
                        balance to charge him
                    */
                    let new_balance = user.balance.unwrap() - (nft_price + mint_nft_request.amount);
                    let updated_user_balance_data = User::update_balance(user.id, "MintNft", "Debit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await.unwrap();

                    let new_balance = nft_owner.balance.unwrap() + nft_price;
                    let updated_owner_balance = User::update_balance(nft_owner.id, "MintNft", "Credit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await.unwrap();

                    uubd = Some(updated_user_balance_data);
                    owner_uubd = Some(updated_owner_balance);

                }

                let lock_ids = NFT_MINT_LOCK.clone(); // it can also be loaded from app data
                let (tx, mut rx) = tokio::sync::mpsc::channel::<bool>(1024);
                // first spawn acquire the lock to push the product id
                tokio::spawn(
                    {
                        let nft_data = nft_data.clone();
                        let tx = tx.clone();
                        let lock_ids = lock_ids.clone();
                        async move{
                            let mut write_lock = lock_ids.lock().await;
                            if (*write_lock).contains(&nft_data.id){
                                // reject the request
                                tx.send(true).await;
                            } else{
                                (*write_lock).push(nft_data.id); // save the id for later readers to reject their request during the minting process
                            }

                        }
                    }
                );

                let cloned_redis_client = redis_client.clone();
                let cloned_mint_nft_request = mint_nft_request.clone();
                let (respmint_sender, mut respmint_receiver) = tokio::sync::mpsc::channel::<(String, String, u8)>(1024);
                let mint_task = tokio::spawn(async move{
                    let (new_tx_hash, token_id, status) = 
                        nftport::mint_nft(
                            cloned_redis_client.clone(), 
                            cloned_mint_nft_request.clone()
                        ).await;
                    
                    
                    respmint_sender.send((new_tx_hash, token_id, status)).await;

                });


                tokio::select!{
                    Some(flag) = rx.recv() => {
                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: &format!("NFT Is Being Minted By Another User"),
                            status: 406,
                            is_error: true
                        };
                        return Err(
                            Ok(HttpResponse::NotAcceptable().json(resp))
                        ); 
                    },
                    _ = mint_task => {

                        let mut get_mint_tx_hash = String::from("");
                        let mut get_token_id = String::from("");
                        let mut get_status = 0;
                        while let Some((new_tx_hash, token_id, status)) = respmint_receiver.recv().await{
                            get_mint_tx_hash = new_tx_hash;
                            get_token_id = token_id;
                            get_status = status;
                        }

                        if get_status == 1{

                            let lock_ids = lock_ids.clone();
                            tokio::spawn(async move{
                                let mut write_lock = lock_ids.lock().await;
                                (*write_lock).retain(|&nft_id| nft_id != nft_data.id);
                            });


                            if uubd.is_some() && owner_uubd.is_some(){
                                // if anything goes wrong payback the user
                                let new_balance = uubd.unwrap().balance.unwrap() + (nft_price + mint_nft_request.amount);
                                let updated_user_balance = User::update_balance(user.id, "ErrorMintNft", "Credit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;

                                let new_balance = owner_uubd.unwrap().balance.unwrap() - nft_price;
                                let updated_owner_balance = User::update_balance(nft_owner.id, "ErrorMintNft", "Debit", new_balance, redis_client.clone(), redis_actor, connection).await;
            
                            }

                            let resp = Response::<'_, &[u8]>{
                                data: Some(&[]),
                                message: CANT_MINT_NFT,
                                status: 417,
                                is_error: true
                            };
                            return Err(
                                Ok(HttpResponse::ExpectationFailed().json(resp))
                            );
                        } else{

                            let lock_ids = lock_ids.clone();
                            tokio::spawn(async move{
                                let mut write_lock = lock_ids.lock().await;
                                (*write_lock).retain(|&nft_id| nft_id != nft_data.id);
                            });

                            mint_nft_request.is_minted = Some(true);
                            mint_nft_request.tx_hash = Some(get_mint_tx_hash); /* updating tx hash with the latest onchain update */
                            mint_nft_request.onchain_id = Some(get_token_id);
                            mint_nft_request.current_owner_screen_cid = caller_screen_cid;
                            mint_nft_request.is_listed = Some(false);
                            mint_nft_request.current_price = Some(0);
    
                            // release the lock, return all ids except nft_data.id
                            // this gets released soon before minting an nft that's 
                            // why two client can mint at the same time cause there is
                            // no nft id in this vector during the minting process of the first client
                            // this must be located in a place right after that 30 seconds
                            // we'll pass it to mint function every async task are handled
                            // by the tokio runtime scheduler and the order of executing them 
                            // may not be like the order of their arrival or defenition.
                            // (*write_lock).retain(|&nft_id| nft_id != nft_data.id);
    
                            match Self::update_nft(
                                mint_nft_request.clone(),
                                redis_client.clone(),
                                redis_actor.clone(), 
                                connection).await{
                                    Ok(updated_user_nft_data) => {
    
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
                                            username: nft_owner.username,
                                            avatar: nft_owner.avatar,
                                            bio: nft_owner.bio,
                                            banner: nft_owner.banner,
                                            mail: nft_owner.mail,
                                            screen_cid: nft_owner.screen_cid,
                                            extra: nft_owner.extra,
                                            stars: nft_owner.stars,
                                            created_at: nft_owner.created_at.to_string(),
                                        };
                                        let user_notif_info = SingleUserNotif{
                                            wallet_info: user_wallet_info,
                                            notif: NotifData{
                                                actioner_wallet_info,
                                                fired_at: Some(chrono::Local::now().timestamp()),
                                                action_type: ActionType::MintNft,
                                                action_data: serde_json::to_value(updated_user_nft_data.clone()).unwrap()
                                            }
                                        };
                                        let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                                        events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;
    
                                        Ok(updated_user_nft_data)
    
                                    },
                                    Err(err) => {
                                        
                                        if uubd.is_some() && owner_uubd.is_some(){
                                            // if anything goes wrong payback the user
                                            let new_balance = uubd.unwrap().balance.unwrap() + (nft_price + mint_nft_request.amount);
                                            let updated_user_balance = User::update_balance(user.id, "ErrorMintNft", "Credit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
                    
                                            let new_balance = owner_uubd.unwrap().balance.unwrap() - nft_price;
                                            let updated_owner_balance = User::update_balance(nft_owner.id, "ErrorMintNft", "Debit", new_balance, redis_client.clone(), redis_actor, connection).await;
                        
                                        }
    
                                        Err(err)
                                    }
                                }
                        }

                        }
                    }


        } else{

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_NFT_EVENT_TYPE,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

    }

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-= NFT OWNER -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
    pub async fn update(mut asset_info: UpdateUserNftRequest,
        redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
        -> Result<UserNftData, PanelHttpResponse>{

        let caller_screen_cid = walletreq::evm::get_keccak256_from(asset_info.clone().caller_cid);
        
        /* find an nft data with the passed in owner address cause only owner can call this method */
        let get_nft = UserNft::find_by_id(asset_info.nft_id, connection).await;
        let Ok(nft_data) = get_nft else{
            let err_resp = get_nft.unwrap_err();
            return Err(err_resp);
        };


        /* find a collection data with the passed in contract address */
        let get_collection = UserCollection::find_by_id(asset_info.col_id, connection).await;
        let Ok(collection_data) = get_collection else{
            let err_resp = get_collection.unwrap_err();
            return Err(err_resp);
        };

        /* find a gallery data with the passed in owner screen address */
        let get_gallery = UserPrivateGallery::find_by_owner_and_collection_id(&collection_data.owner_screen_cid, asset_info.col_id, redis_client.clone(), connection).await;
        let Ok(gallery_data) = get_gallery else{
            let err_resp = get_gallery.unwrap_err();
            return Err(err_resp);
        };

        let get_user = User::find_by_screen_cid(&caller_screen_cid, connection).await;
        let Ok(user) = get_user else{
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };

        if nft_data.current_owner_screen_cid != user.clone().screen_cid.unwrap_or_default(){
            
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: NFT_NOT_OWNED_BY,
                status: 403,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            );
        }
        
        let mut actioner_wallet_info = Default::default();
        let mut user_wallet_info = Default::default();
        let mut action_data = Default::default();

        let res = match asset_info.event_type.as_str(){

            /*  --------------------------------------------------------------------------------
                Note: Transferring is possible only if the token is owned by the contract owner 
                and the token has not been transferred/sold yet, in other words nft can be transferred 
                once and it can be transferred if the token has been minted to to the contract owner so
                ✅ following scenario is correct
                only owner: create nft, mint nft then list
                only other: buy
                ❌ but the following is not:
                owner: create nft
                other: mint then list 
                owner or other: buy
                means that only owner can transfer or list nft for sell and only once!
                --------------------------------------------------------------------------------
            */
            "transfer" => {

                /* 
                    in order to update nft freeze_metadata, metadata_uri or transfer it 
                    or list it, nft must be minted first and have an onchain id already
                */
                if nft_data.is_minted.is_none() || 
                    (nft_data.is_minted.is_some() && nft_data.is_minted.unwrap() == false) &&  
                    nft_data.onchain_id.is_none() || 
                    (nft_data.onchain_id.is_some() && nft_data.onchain_id.unwrap().is_empty()){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_IS_NOT_MINTED_YET,
                        status: 406,
                        is_error: true
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
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }

                /* make sure that there is a user with this screen cid in the app */
                let recipient = asset_info.clone().transfer_to_screen_cid.unwrap_or_default();
                let find_recipient_screen_cid = User::find_by_username_or_mail_or_scid(&recipient, connection).await;
                let Ok(recipient_info) = find_recipient_screen_cid else{
                    
                    let err_resp = find_recipient_screen_cid.unwrap_err();
                    return Err(err_resp);
                };

                // check that nft owner and recipient_info are friend
                let nft_owner_screen_cid = user.clone().screen_cid.unwrap_or_default();
                let recipient_info_screen_cid = recipient_info.clone().screen_cid.unwrap_or_default();
                let check_we_are_friend = UserFan::are_we_friends(
                    &nft_owner_screen_cid, 
                    &recipient_info_screen_cid, connection).await;
                
                if check_we_are_friend.is_ok() && *check_we_are_friend.as_ref().unwrap(){

                    /* if any update goes well we charge the user for onchain gas fee */
                    let new_balance = user.balance.unwrap() - asset_info.amount;
                    let update_user_balance = User::update_balance(user.id, "UpdateArtWork", "Debit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
                    let Ok(updated_user_balance_data) = update_user_balance else{

                        let err_resp = update_user_balance.unwrap_err();
                        return Err(err_resp);
                        
                    };

                    asset_info.transfer_to_screen_cid = Some(recipient_info_screen_cid.clone());
                    let (new_tx_hash, status) = 
                        nftport::transfer_nft(
                            redis_client.clone(), 
                            asset_info.clone()
                        ).await;

                    if status == 1{

                        /* if any update goes well we charge the user for onchain gas fee */
                        let new_balance = updated_user_balance_data.balance.unwrap() + asset_info.amount;
                        let update_user_balance = User::update_balance(user.id, "UpdateArtWork", "Credit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
                        let Ok(updated_user_data) = update_user_balance else{

                            let err_resp = update_user_balance.unwrap_err();
                            return Err(err_resp);
                            
                        };
                        
                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: CANT_TRANSFER_NFT,
                            status: 417,
                            is_error: true
                        };
                        return Err(
                            Ok(HttpResponse::ExpectationFailed().json(resp))
                        );
                    }

                    asset_info.tx_hash = Some(new_tx_hash); /* updating tx hash with the latest onchain update */
                    asset_info.current_owner_screen_cid = recipient_info_screen_cid; 
                    asset_info.is_listed = Some(false); /* set is_listed to false for new owner */
                    asset_info.current_price = Some(0); /* set current_price to false for new owner */
                    
                    match Self::update_nft(
                        asset_info.clone(), 
                        redis_client.clone(),
                        redis_actor.clone(),
                        connection).await{
                            Ok(updated_user_nft_data) => {

                                actioner_wallet_info = UserWalletInfoResponse{
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
                                user_wallet_info = UserWalletInfoResponse{
                                    username: recipient_info.username,
                                    avatar: recipient_info.avatar,
                                    bio: recipient_info.bio,
                                    banner: recipient_info.banner,
                                    mail: recipient_info.mail,
                                    screen_cid: recipient_info.screen_cid,
                                    extra: recipient_info.extra,
                                    stars: recipient_info.stars,
                                    created_at: recipient_info.created_at.to_string(),
                                };

                                action_data = updated_user_nft_data.clone();
                                Ok(updated_user_nft_data)
                            },
                            Err(err) => Err(err)
                        }

                } else{

                    let recipient_username = recipient_info.clone().username;
                    let nft_owner_username = user.clone().username;
                    let resp_msg = format!("{recipient_username:} Is Not A Friend Of {nft_owner_username:}");
                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: &resp_msg,
                        status: 406,
                        is_error: true
                    };
                    return 
                        Err(Ok(HttpResponse::NotAcceptable().json(resp)));
                }

            },
            "delist" => { 

                if nft_data.is_listed.is_some() && nft_data.is_listed.unwrap() == false{

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_IS_NOT_LISTED,
                        status: 406,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }

                asset_info.is_listed = Some(false);

                match Self::update_nft(
                    asset_info.clone(), 
                    redis_client.clone(),
                    redis_actor.clone(),
                    connection).await{
                        Ok(updated_user_nft_data) => {

                            actioner_wallet_info = UserWalletInfoResponse{
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
                            user_wallet_info = UserWalletInfoResponse{
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

                            action_data = updated_user_nft_data.clone();
                            Ok(updated_user_nft_data)
                        },
                        Err(err) => Err(err)
                    }

            },
            "sell" => { 

                if asset_info.current_price.is_none(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_PRICE_IS_EMPTY,
                        status: 406,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }

                if asset_info.current_price.is_some() && asset_info.current_price.unwrap() < 0 || asset_info.current_price.unwrap() == 0{

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: INVALID_NFT_PRICE,
                        status: 406,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }

                asset_info.is_listed = Some(true);

                match Self::update_nft(
                    asset_info.clone(), 
                    redis_client.clone(),
                    redis_actor.clone(),
                    connection).await{
                        Ok(updated_user_nft_data) => {

                            let get_nft_owner_friends = UserFan::get_all_my_friends_without_limit(&user.clone().screen_cid.unwrap_or_default(), connection).await;
                            if get_nft_owner_friends.is_ok(){
                                let nft_owner_friends = get_nft_owner_friends.unwrap();
                                let friends_data = nft_owner_friends.clone().construct_friends_data(connection);
                                let decoded_friends_data = if friends_data.is_some(){
                                    serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
                                } else{
                                    vec![]
                                };

                                // publish the list event to all nft owner friends
                                events::publishers::action::emit_nft_list_event_2_all_nft_owner_friends(
                                    decoded_friends_data, 
                                    redis_actor.clone(), 
                                    "on_user_action", 
                                    user.clone(),
                                    serde_json::to_value(updated_user_nft_data.clone()).unwrap(),
                                    connection
                                ).await;
                            }

                            actioner_wallet_info = UserWalletInfoResponse{
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

                            user_wallet_info = UserWalletInfoResponse{
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
                            
                            action_data = updated_user_nft_data.clone();
                            Ok(updated_user_nft_data)
                        },
                        Err(err) => Err(err)
                    }

            },
            /*  ------------------------------------------------------------------------------
                currently we're getting the following error in updating `freeze_metadata` and
                `metadata_uri` fields from the nftport server:
                    the request could not be completed due to an internal server error. 

                note that in order to update metadata_uri field, the freeze_metadata must not 
                be set to true and before calling this api user must create a new metadata uri 
                using /nft/create/metadata-uri api.
            */
            "onchain" => { // only freeze_metadata and metadata_uri

                /* 
                    in order to update nft freeze_metadata, metadata_uri or transfer it 
                    or list it, nft must be minted first and have an onchain id already
                */
                if nft_data.is_minted.is_none() || 
                    (nft_data.is_minted.is_some() && nft_data.is_minted.unwrap() == false) &&  
                    nft_data.onchain_id.is_none() || 
                    (nft_data.onchain_id.is_some() && nft_data.onchain_id.unwrap().is_empty()){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_IS_NOT_MINTED_YET,
                        status: 406,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }

                if asset_info.metadata_uri.is_empty(){

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: NFT_METADATA_URI_IS_EMPTY,
                        status: 406,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    ); 

                }

                asset_info.metadata_uri = if nft_data.freeze_metadata.is_some() &&
                    nft_data.freeze_metadata.unwrap() == true{

                        /* 
                            just ignore updating metadata_uri and use the old one 
                            cause we can't update metadata_uri onchain if the freeze_metadata
                            is set to true
                        */
                        nft_data.metadata_uri 

                    } else{

                    asset_info.metadata_uri
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
                
                /* if any update goes well we charge the user for onchain gas fee */
                let new_balance = user.balance.unwrap() - asset_info.amount;
                let update_user_balance = User::update_balance(user.id, "UpdateArtWork", "Debit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
                let Ok(updated_user_balance_data) = update_user_balance else{

                    let err_resp = update_user_balance.unwrap_err();
                    return Err(err_resp);
                    
                };

                let (new_tx_hash, status) = 
                    nftport::update_nft(redis_client.clone(), asset_info.clone()).await;

                if status == 1{
                    
                    /* if any update goes well we charge the user for onchain gas fee */
                    let new_balance = updated_user_balance_data.balance.unwrap() + asset_info.amount;
                    let update_user_balance = User::update_balance(user.id, "UpdateArtWork", "Credit", new_balance, redis_client.clone(), redis_actor, connection).await;
                    let Ok(updated_user_data) = update_user_balance else{

                        let err_resp = update_user_balance.unwrap_err();
                        return Err(err_resp);
                        
                    };

                    let resp = Response::<'_, &[u8]>{
                        data: Some(&[]),
                        message: CANT_UPDATE_NFT,
                        status: 417,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::ExpectationFailed().json(resp))
                    );
                }

                asset_info.tx_hash = Some(new_tx_hash);

                match Self::update_nft(
                    asset_info.clone(), 
                    redis_client.clone(),
                    redis_actor.clone(),
                    connection).await{
                        Ok(updated_user_nft_data) => {

                            actioner_wallet_info = UserWalletInfoResponse{
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
                            user_wallet_info = UserWalletInfoResponse{
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

                            action_data = updated_user_nft_data.clone();
                            Ok(updated_user_nft_data)
                        }, 
                        Err(err) => Err(err),
                    }

            },
            _ => {
                
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_NFT_EVENT_TYPE,
                    status: 406,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                );
            }
        };

        if res.is_ok(){

            /** -------------------------------------------------------------------- */
            /** ----------------- publish new event data to `on_user_action` channel */
            /** -------------------------------------------------------------------- */
            // if the actioner is the user himself we'll notify user with something like:
            // u've just done that action!
            let user_notif_info = SingleUserNotif{
                wallet_info: user_wallet_info,
                notif: NotifData{
                    actioner_wallet_info,
                    fired_at: Some(chrono::Local::now().timestamp()),
                    action_type: match asset_info.event_type.as_str(){
                        "transfer" => ActionType::TransferNft,
                        "sell" => ActionType::ListNft,
                        "delist" => ActionType::DelistNft,
                        _ => ActionType::UpdateOnchainNft
                    },
                    action_data: serde_json::to_value(action_data.clone()).unwrap()
                }
            };
            let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
            events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

            res /* contains updated nft in ok part */

        } else{
            return Err(res.unwrap_err()); /* the error part is an http response */
        }
        

    }

}