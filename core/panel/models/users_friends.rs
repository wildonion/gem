

use crate::*;
use crate::adapters::nftport::{self, NftExt, OnchainNfts};
use crate::constants::{GALLERY_NOT_OWNED_BY, NFT_NOT_OWNED_BY, NFT_UPLOAD_PATH, INVALID_QUERY_LIMIT, STORAGE_IO_ERROR_CODE, NFT_ONCHAINID_NOT_FOUND, NFT_UPLOAD_ISSUE, CANT_MINT_CARD, CANT_MINT_NFT, CANT_TRANSFER_NFT, NFT_EVENT_TYPE_RECIPIENT_IS_NEEDED, NFT_EVENT_TYPE_METADATA_URI_IS_NEEDED, INVALID_NFT_EVENT_TYPE, NFT_IS_NOT_MINTED_YET, CANT_UPDATE_NFT, NFT_NOT_FOUND_OF, NFT_IS_ALREADY_MINTED, NFT_IS_NOT_LISTED_YET, NFT_PRICE_IS_EMPTY, NFT_EVENT_TYPE_BUYER_IS_NEEDED, CALLER_IS_NOT_BUYER, INVALID_NFT_ROYALTY, INVALID_NFT_PRICE, RECIPIENT_SCREEN_CID_NOT_FOUND, EMPTY_NFT_IMG, NFT_NOT_FOUND_OF_ID, USER_SCREEN_CID_NOT_FOUND, NFT_METADATA_URI_IS_EMPTY, NFT_IS_NOT_LISTED, NOT_FOUND_NFT, NFT_IS_NOT_OWNED_BY_THE_PASSED_IN_OWNER};
use crate::events::publishers::action::{SingleUserNotif, NotifData, ActionType};
use crate::misc::{Response, Limit};
use crate::schema::users_friends::dsl::*;
use crate::schema::users_friends;
use self::constants::{APP_NAME, NO_COMMENT_FOUND_FOR_THIS_NFT, NO_COMMENT_FOUND_FOR_THIS_USER, NO_FRIEND_FOUND_FOR_THIS_USER, RECIPIENT_NOT_FOUND};
use super::users::{User, UserData, UserWalletInfoResponse};
use super::users_collections::{UserCollection, UserCollectionData, UpdateUserCollection};
use super::users_fans::{UserFan, FriendData};
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest, UserPrivateGalleryData};
use crate::schema::users_collections::dsl::*;
use crate::schema::users_collections;

/* 

    diesel migration generate users_friends ---> create users_friends migration sql files
    diesel migration run                    ---> apply sql files to db 
    diesel migration redo                   ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_friends)]
pub struct UserFriend{
    pub id: i32,
    pub user_id: i32,
    pub friend_id: i32,
    pub is_accepted: bool,
    pub requested_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[derive(Insertable)]
#[diesel(table_name=users_friends)]
pub struct NewFriendRequest{
    pub user_id: i32,
    pub friend_id: i32,
    pub is_accepted: bool,
    pub requested_at: i64,
}

impl UserFriend{

    pub fn remove(owner_id: i32, frd_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<usize, PanelHttpResponse>{

        match diesel::delete(users_friends
            .filter(users_friends::user_id.eq(owner_id)))
            .filter(users_friends::friend_id.eq(frd_id))
            .execute(connection)
            {
                Ok(num_deleted) => Ok(num_deleted),
                Err(e) => {

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: &e.to_string(),
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }
    
    pub async fn insert(new_friend_request: NewFriendRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserFriend, PanelHttpResponse>{

        match users_friends
            .filter(users_friends::user_id.eq(new_friend_request.user_id))
            .filter(users_friends::friend_id.eq(new_friend_request.friend_id))
            .first::<UserFriend>(connection)
            {
                Ok(found_frd_req) => Ok(found_frd_req),
                Err(e) => {

                    /* inserting new comment */
                    match diesel::insert_into(users_friends)
                    .values(&new_friend_request)
                    .returning(UserFriend::as_returning())
                    .get_result::<UserFriend>(connection)
                    {
                        Ok(friend_data) => Ok(friend_data),
                        Err(e) => {

                            let resp_err = &e.to_string();


                            /* custom error handler */
                            use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();  
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserFriend::insert");
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

    }

    pub fn get_all_for_user(owner_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<UserFriend>, PanelHttpResponse>{


        let get_all_frd_requests = users_friends
            .filter(users_friends::user_id.eq(owner_id))
            .load::<UserFriend>(connection);
                        
        let Ok(frd_requests) = get_all_frd_requests else{
            let resp = Response{
                data: Some(owner_id),
                message: NO_FRIEND_FOUND_FOR_THIS_USER,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(frd_requests)

    }

    pub async fn get_all_friends_wallet_info_for(owner_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<UserWalletInfoResponse>, PanelHttpResponse>{

        let get_friend_reqs = users_friends
            .filter(users_friends::user_id.eq(owner_id))
            .load::<UserFriend>(connection);

        let Ok(friend_reqs) = get_friend_reqs else{
            let resp = Response{
                data: Some(owner_id),
                message: NO_FRIEND_FOUND_FOR_THIS_USER,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(
            friend_reqs
                .into_iter()
                .map(|frd_req_info| {
                    let user = User::find_by_id_none_async(frd_req_info.friend_id, connection).unwrap();
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
                })
                .collect::<Vec<UserWalletInfoResponse>>()
        )

    }

    pub async fn accept_request(owner_id: i32, frd_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<UserFriend, PanelHttpResponse>{


        match users_friends
            .filter(users_friends::user_id.eq(owner_id))
            .filter(users_friends::friend_id.eq(frd_id))
            .first::<UserFriend>(connection)
            {
                Ok(frd_req) => {

                    // update like
                    match diesel::update(users_friends.find(frd_req.id))
                        .set(is_accepted.eq(true))
                        .returning(UserFriend::as_returning())
                        .get_result::<UserFriend>(connection)
                        {
                            Ok(updated_frd_req) => Ok(updated_frd_req),
                            Err(e) => {
                                let resp_err = &e.to_string();

                                /* custom error handler */
                                use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                
                                let error_content = &e.to_string();
                                let error_content = error_content.as_bytes().to_vec();  
                                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserFriend::accept_request-update");
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
                    
                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserFriend::accept_request-find");
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
