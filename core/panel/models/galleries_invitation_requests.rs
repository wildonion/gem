


use crate::*;
use crate::adapters::nftport::{self, NftExt, OnchainNfts};
use crate::constants::{GALLERY_NOT_OWNED_BY, NFT_NOT_OWNED_BY, NFT_UPLOAD_PATH, INVALID_QUERY_LIMIT, STORAGE_IO_ERROR_CODE, NFT_ONCHAINID_NOT_FOUND, NFT_UPLOAD_ISSUE, CANT_MINT_CARD, CANT_MINT_NFT, CANT_TRANSFER_NFT, NFT_EVENT_TYPE_RECIPIENT_IS_NEEDED, NFT_EVENT_TYPE_METADATA_URI_IS_NEEDED, INVALID_NFT_EVENT_TYPE, NFT_IS_NOT_MINTED_YET, CANT_UPDATE_NFT, NFT_NOT_FOUND_OF, NFT_IS_ALREADY_MINTED, NFT_IS_NOT_LISTED_YET, NFT_PRICE_IS_EMPTY, NFT_EVENT_TYPE_BUYER_IS_NEEDED, CALLER_IS_NOT_BUYER, INVALID_NFT_ROYALTY, INVALID_NFT_PRICE, RECIPIENT_SCREEN_CID_NOT_FOUND, EMPTY_NFT_IMG, NFT_NOT_FOUND_OF_ID, USER_SCREEN_CID_NOT_FOUND, NFT_METADATA_URI_IS_EMPTY, NFT_IS_NOT_LISTED, NOT_FOUND_NFT, NFT_IS_NOT_OWNED_BY_THE_PASSED_IN_OWNER};
use crate::events::publishers::action::{SingleUserNotif, NotifData, ActionType};
use crate::helpers::misc::{Response, Limit};
use crate::schema::galleries_invitation_requests::dsl::*;
use crate::schema::galleries_invitation_requests;
use self::constants::{APP_NAME, NO_COMMENT_FOUND_FOR_THIS_NFT, NO_COMMENT_FOUND_FOR_THIS_USER, NO_INVITATION_FOUND_FOR_THIS_USER, RECIPIENT_NOT_FOUND};
use super::users::{User, UserData, UserWalletInfoResponse};
use super::users_collections::{UserCollection, UserCollectionData, UpdateUserCollection};
use super::users_fans::{UserFan, FriendData};
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest, UserPrivateGalleryData};
use crate::schema::users_collections::dsl::*;
use crate::schema::users_collections;

/* 

    diesel migration generate galleries_invitation_requests ---> create galleries_invitation_requests migration sql files
    diesel migration run                                    ---> apply sql files to db 
    diesel migration redo                                   ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=galleries_invitation_requests)]
pub struct PrivateGalleryInvitationRequest{
    pub id: i32,
    pub invitee_id: i32,
    pub from_user_id: i32,
    pub gal_id: i32,
    pub is_accepted: bool,
    pub requested_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[derive(Insertable)]
#[diesel(table_name=galleries_invitation_requests)]
pub struct NewPrivateGalleryInvitationRequest{
    pub invitee_id: i32,
    pub from_user_id: i32,
    pub gal_id: i32,
    pub is_accepted: bool,
    pub requested_at: i64,
}

impl PrivateGalleryInvitationRequest{

    pub async fn remove(owner_id: i32, gallery_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<usize, PanelHttpResponse>{

        match diesel::delete(galleries_invitation_requests
            .filter(galleries_invitation_requests::invitee_id.eq(owner_id)))
            .filter(galleries_invitation_requests::gal_id.eq(gallery_id))
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
    
    pub async fn insert(new_invitation_request: NewPrivateGalleryInvitationRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<PrivateGalleryInvitationRequest, PanelHttpResponse>{

        
        /* inserting new comment */
        match diesel::insert_into(galleries_invitation_requests)
            .values(&new_invitation_request)
            .returning(PrivateGalleryInvitationRequest::as_returning())
            .get_result::<PrivateGalleryInvitationRequest>(connection)
            {
                Ok(invitation_data) => Ok(invitation_data),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "PrivateGalleryInvitationRequest::insert");
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

    pub fn get_all_for_user(owner_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<PrivateGalleryInvitationRequest>, PanelHttpResponse>{


        let get_all_invitation_requests = galleries_invitation_requests
            .filter(galleries_invitation_requests::invitee_id.eq(owner_id))
            .load::<PrivateGalleryInvitationRequest>(connection);
                        
        let Ok(inv_requests) = get_all_invitation_requests else{
            let resp = Response{
                data: Some(owner_id),
                message: NO_INVITATION_FOUND_FOR_THIS_USER,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(inv_requests)

    }

    pub async fn accept_request(owner_id: i32, gallery_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<PrivateGalleryInvitationRequest, PanelHttpResponse>{


        match galleries_invitation_requests
            .filter(galleries_invitation_requests::invitee_id.eq(owner_id))
            .filter(galleries_invitation_requests::gal_id.eq(gallery_id))
            .first::<PrivateGalleryInvitationRequest>(connection)
            {
                Ok(invitation_request) => {

                    // update like
                    match diesel::update(galleries_invitation_requests.find(invitation_request.id))
                        .set(is_accepted.eq(true))
                        .returning(PrivateGalleryInvitationRequest::as_returning())
                        .get_result::<PrivateGalleryInvitationRequest>(connection)
                        {
                            Ok(updated_inv_req) => Ok(updated_inv_req),
                            Err(e) => {
                                let resp_err = &e.to_string();

                                /* custom error handler */
                                use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                
                                let error_content = &e.to_string();
                                let error_content = error_content.as_bytes().to_vec();  
                                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "PrivateGalleryInvitationRequest::accept_request-update");
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
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "PrivateGalleryInvitationRequest::accept_request-find");
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
