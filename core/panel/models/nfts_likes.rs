

use crate::*;
use crate::adapters::nftport::{self, NftExt, OnchainNfts};
use crate::constants::{GALLERY_NOT_OWNED_BY, NFT_NOT_OWNED_BY, NFT_UPLOAD_PATH, INVALID_QUERY_LIMIT, STORAGE_IO_ERROR_CODE, NFT_ONCHAINID_NOT_FOUND, NFT_UPLOAD_ISSUE, CANT_MINT_CARD, CANT_MINT_NFT, CANT_TRANSFER_NFT, NFT_EVENT_TYPE_RECIPIENT_IS_NEEDED, NFT_EVENT_TYPE_METADATA_URI_IS_NEEDED, INVALID_NFT_EVENT_TYPE, NFT_IS_NOT_MINTED_YET, CANT_UPDATE_NFT, NFT_NOT_FOUND_OF, NFT_IS_ALREADY_MINTED, NFT_IS_NOT_LISTED_YET, NFT_PRICE_IS_EMPTY, NFT_EVENT_TYPE_BUYER_IS_NEEDED, CALLER_IS_NOT_BUYER, INVALID_NFT_ROYALTY, INVALID_NFT_PRICE, RECIPIENT_SCREEN_CID_NOT_FOUND, EMPTY_NFT_IMG, NFT_NOT_FOUND_OF_ID, USER_SCREEN_CID_NOT_FOUND, NFT_METADATA_URI_IS_EMPTY, NFT_IS_NOT_LISTED, NOT_FOUND_NFT, NFT_IS_NOT_OWNED_BY_THE_PASSED_IN_OWNER};
use crate::events::publishers::action::{SingleUserNotif, NotifData, ActionType};
use crate::helpers::misc::{Response, Limit};
use crate::schema::nfts_likes::dsl::*;
use crate::schema::nfts_likes;
use self::constants::{APP_NAME, NO_LIKE_FOUND_FOR_THIS_NFT, NO_LIKE_FOUND_FOR_THIS_USER};
use super::users::{User, UserData, UserWalletInfoResponse};
use super::users_collections::{UserCollection, UserCollectionData, UpdateUserCollection};
use super::users_fans::{UserFan, FriendData};
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest, UserPrivateGalleryData};
use crate::schema::users_collections::dsl::*;
use crate::schema::users_collections;

/* 

    diesel migration generate nfts_likes    ---> create nfts_likes migration sql files
    diesel migration run                    ---> apply sql files to db 
    diesel migration redo                   ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=nfts_likes)]
pub struct NftLike{
    pub id: i32,
    pub user_id: i32,
    pub nft_id: i32,
    pub is_upvote: bool,
    pub published_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[derive(Insertable)]
#[diesel(table_name=nfts_likes)]
pub struct NewNftLikeRequest{
    pub user_id: i32,
    pub nft_id: i32,
    pub is_upvote: bool,
}

impl NftLike{

    pub async fn insert(new_like: NewNftLikeRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<NftLike, PanelHttpResponse>{

            match diesel::insert_into(nfts_likes)
            .values(&new_like)
            .returning(NftLike::as_returning())
            .get_result::<NftLike>(connection)
            {
                Ok(like_data) => Ok(like_data),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "NftLike::insert");
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

    pub async fn get_all_for_nft(asset_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<NftLike>, PanelHttpResponse>{

        let get_all_likes = nfts_likes
            .filter(nfts_likes::nft_id.eq(asset_id))
            .load::<NftLike>(connection);
                        
        let Ok(likes) = get_all_likes else{
            let resp = Response{
                data: Some(asset_id),
                message: NO_LIKE_FOUND_FOR_THIS_NFT,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(likes)

    }

    pub fn get_all_upvotes_for_nft(asset_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<NftLike>, PanelHttpResponse>{

        let get_all_likes = nfts_likes
            .filter(nfts_likes::nft_id.eq(asset_id))
            .filter(nfts_likes::is_upvote.eq(true))
            .load::<NftLike>(connection);
                        
        let Ok(likes) = get_all_likes else{
            let resp = Response{
                data: Some(asset_id),
                message: NO_LIKE_FOUND_FOR_THIS_NFT,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(likes)

    }

    pub fn get_all_downvotes_for_nft(asset_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<NftLike>, PanelHttpResponse>{

        let get_all_likes = nfts_likes
            .filter(nfts_likes::nft_id.eq(asset_id))
            .filter(nfts_likes::is_upvote.eq(false))
            .load::<NftLike>(connection);
                        
        let Ok(likes) = get_all_likes else{
            let resp = Response{
                data: Some(asset_id),
                message: NO_LIKE_FOUND_FOR_THIS_NFT,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(likes)

    }

    pub async fn get_all_for_user(owner_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<NftLike>, PanelHttpResponse>{

        
        let get_all_likes = nfts_likes
            .filter(nfts_likes::user_id.eq(owner_id))
            .load::<NftLike>(connection);
                        
        let Ok(likes) = get_all_likes else{
            let resp = Response{
                data: Some(owner_id),
                message: NO_LIKE_FOUND_FOR_THIS_USER,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(likes)

    }

    pub async fn upsert(new_like: NewNftLikeRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<NftLike, PanelHttpResponse>{

        
        match nfts_likes
            .filter(nfts_likes::user_id.eq(new_like.user_id))
            .filter(nfts_likes::nft_id.eq(new_like.nft_id))
            .first::<NftLike>(connection)
            {
                Ok(like) => {

                    // update like
                    match diesel::update(nfts_likes.find(like.id))
                        .set(is_upvote.eq(new_like.is_upvote))
                        .returning(NftLike::as_returning())
                        .get_result::<NftLike>(connection)
                        {
                            Ok(updated_like) => Ok(updated_like),
                            Err(e) => {
                                let resp_err = &e.to_string();

                                /* custom error handler */
                                use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                
                                let error_content = &e.to_string();
                                let error_content = error_content.as_bytes().to_vec();  
                                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "NftLike::upsert");
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
                }, 
                Err(e) => {
                    // insert new like
                    Self::insert(new_like, connection).await // the return type is NftLike structure
                }
            }
                        

    }

}