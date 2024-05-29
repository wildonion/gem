
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::NaiveDateTime;
use models::users::UserWalletInfoResponse;
use crate::adapters::nftport;
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN, COLLECTION_NOT_FOUND_FOR_CONTRACT};
use crate::{*, constants::COLLECTION_NOT_FOUND_OF};
use super::users::User;
use super::users_galleries::{UserPrivateGalleryData, UserPrivateGallery, UpdateUserPrivateGallery, UpdateUserPrivateGalleryRequest};
use super::users_nfts::UserNftData;
use crate::schema::user_treasury::dsl::*;
use crate::schema::user_treasury;


/* 

    diesel migration generate user_treasury        ---> create user_treasury migration sql files
    diesel migration run                         ---> apply sql files to db 
    diesel migration redo                        ---> drop tables 

*/
#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=user_treasury)]
pub struct UserTreasury{
    pub id: i32,
    pub user_id: i32,
    pub done_at: i64,
    pub amount: i64,
    pub tx_type: String,
    pub treasury_type: String

}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(Insertable, PartialEq)]
#[diesel(table_name=user_treasury)]
pub struct UserTreasuryRequest{
    pub user_id: i32,
    pub done_at: i64,
    pub amount: i64,
    pub tx_type: String,
    pub treasury_type: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserWalletTreasuryRequest{
    pub id: i32,
    pub user_id: i32,
    pub done_at: i64,
    pub amount: i64,
    pub tx_type: String,
    pub wallet_info: UserWalletInfoResponse,
    pub treasury_type: String,
}