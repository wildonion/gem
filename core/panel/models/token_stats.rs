
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::NaiveDateTime;
use crate::adapters::nftport;
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN, COLLECTION_NOT_FOUND_FOR_CONTRACT, CLP_EVENT_NOT_FOUND, USER_CLP_EVENT_NOT_FOUND};
use crate::misc::{Response, Limit};
use crate::{*, constants::COLLECTION_NOT_FOUND_OF};
use self::constants::NO_CLP_EVENT;
use super::users::User;
use super::users_galleries::{UserPrivateGalleryData, UserPrivateGallery, UpdateUserPrivateGallery, UpdateUserPrivateGalleryRequest};
use super::users_nfts::UserNftData;
use crate::schema::token_stats::dsl::*;
use crate::schema::token_stats;


/* 

    diesel migration generate token_stats        ---> create token_stats migration sql files
    diesel migration run                         ---> apply sql files to db 
    diesel migration redo                        ---> drop tables 

*/
#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=token_stats)]
pub struct TokenStatInfo{
    pub id: i32,
    pub user_id: i32,
    pub usd_token_price: i64,
    pub requested_tokens: i64,
    pub requested_at: chrono::NaiveDateTime
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(Insertable, PartialEq)]
#[diesel(table_name=token_stats)]
pub struct TokenStatInfoRequest{
    pub usd_token_price: i64,
    pub requested_tokens: i64,
    pub user_id: i32
}

impl TokenStatInfo{

    pub async fn save(
        token_stat_info: TokenStatInfoRequest,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>
    ) -> Result<TokenStatInfo, PanelHttpResponse>{


        match diesel::insert_into(token_stats)
            .values(&token_stat_info)
            .returning(TokenStatInfo::as_returning())
            .get_result::<TokenStatInfo>(connection)
            {
                Ok(token_stat_info) => {

                    Ok(TokenStatInfo{
                        id: token_stat_info.id,
                        user_id: token_stat_info.user_id,
                        usd_token_price: token_stat_info.usd_token_price,
                        requested_tokens: token_stat_info.requested_tokens,
                        requested_at: token_stat_info.requested_at,
                    })

                },
                Err(e) => {

                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "TokenStatInfo::save");
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