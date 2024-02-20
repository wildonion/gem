

use crate::*;
use crate::adapters::nftport::{self, NftExt, OnchainNfts};
use crate::constants::{GALLERY_NOT_OWNED_BY, NFT_NOT_OWNED_BY, NFT_UPLOAD_PATH, INVALID_QUERY_LIMIT, STORAGE_IO_ERROR_CODE, NFT_ONCHAINID_NOT_FOUND, NFT_UPLOAD_ISSUE, CANT_MINT_CARD, CANT_MINT_NFT, CANT_TRANSFER_NFT, NFT_EVENT_TYPE_RECIPIENT_IS_NEEDED, NFT_EVENT_TYPE_METADATA_URI_IS_NEEDED, INVALID_NFT_EVENT_TYPE, NFT_IS_NOT_MINTED_YET, CANT_UPDATE_NFT, NFT_NOT_FOUND_OF, NFT_IS_ALREADY_MINTED, NFT_IS_NOT_LISTED_YET, NFT_PRICE_IS_EMPTY, NFT_EVENT_TYPE_BUYER_IS_NEEDED, CALLER_IS_NOT_BUYER, INVALID_NFT_ROYALTY, INVALID_NFT_PRICE, RECIPIENT_SCREEN_CID_NOT_FOUND, EMPTY_NFT_IMG, NFT_NOT_FOUND_OF_ID, USER_SCREEN_CID_NOT_FOUND, NFT_METADATA_URI_IS_EMPTY, NFT_IS_NOT_LISTED, NOT_FOUND_NFT, NFT_IS_NOT_OWNED_BY_THE_PASSED_IN_OWNER};
use crate::events::publishers::action::{SingleUserNotif, NotifData, ActionType};
use crate::helpers::misc::{Response, Limit};
use crate::schema::users_logins::dsl::*;
use crate::schema::users_logins;
use self::constants::{APP_NAME, LOGIN_DATA_NOT_FOUND, NO_COMMENT_FOUND_FOR_THIS_NFT, NO_COMMENT_FOUND_FOR_THIS_USER, RECIPIENT_NOT_FOUND, USER_NOT_FOUND};
use super::users::{User, UserData, UserWalletInfoResponse};
use super::users_collections::{UserCollection, UserCollectionData, UpdateUserCollection};
use super::users_fans::{UserFan, FriendData};
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest, UserPrivateGalleryData};
use crate::schema::users_collections::dsl::*;
use crate::schema::users_collections;

/* 

    diesel migration generate users_logins  ---> create users_logins migration sql files
    diesel migration run                    ---> apply sql files to db 
    diesel migration redo                   ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_logins)]
pub struct UserLogin{
    pub id: i32,
    pub user_id: i32,
    pub device_id: String,
    pub jwt: String,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[derive(Insertable)]
#[diesel(table_name=users_logins)]
pub struct NewUserLoginRequest{
    pub user_id: i32,
    pub device_id: String,
    pub jwt: String,
    pub last_login: Option<chrono::NaiveDateTime>
}

impl UserLogin{

    pub async fn insert(new_login: NewUserLoginRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, PanelHttpResponse>{

        match diesel::insert_into(users_logins)
            .values(&new_login)
            .returning(UserLogin::as_returning())
            .get_result::<UserLogin>(connection)
            {
                Ok(login_data) => Ok(login_data),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserLogin::insert");
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

    pub async fn find_by_user_id(owner_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Vec<Self>, PanelHttpResponse>{
    
        let single_user_login = users_logins
            .filter(users_logins::user_id.eq(owner_id))
            .load::<UserLogin>(connection);
                        
        let Ok(login_info) = single_user_login else{
            let resp = Response{
                data: Some(owner_id),
                message: LOGIN_DATA_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(login_info)

    }

    pub async fn update(login_info: NewUserLoginRequest, login_data_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, PanelHttpResponse>{

        let now = chrono::Local::now().naive_local();
        match diesel::update(users_logins.find(login_data_id))
            .set(
                // update only last login time and the new jwt
                (users_logins::last_login.eq(now), users_logins::jwt.eq(login_info.jwt))
            )
            .returning(UserLogin::as_returning())
            .get_result::<UserLogin>(connection)
            {
                Ok(updated_user_login) => Ok(updated_user_login),
                Err(e) => {
                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserLogin::update");
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

    pub async fn remove_jwt(login_data_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, PanelHttpResponse>{

        let now = chrono::Local::now().naive_local();
        match diesel::update(users_logins.find(login_data_id))
            .set(
                (users_logins::jwt.eq(""))
            )
            .returning(UserLogin::as_returning())
            .get_result::<UserLogin>(connection)
            {
                Ok(updated_user_login) => Ok(updated_user_login),
                Err(e) => {
                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserLogin::remove_jwt");
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

    pub async fn upsert(new_login: NewUserLoginRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<Self, PanelHttpResponse>{

        match users_logins
            .filter(users_logins::user_id.eq(new_login.user_id))
            .filter(users_logins::device_id.eq(new_login.clone().device_id))
            .first::<UserLogin>(connection)
            {
                Ok(found_login_info) => Self::update(new_login, found_login_info.id, connection).await,
                Err(e) => Self::insert(new_login, connection).await
            }

    }
}