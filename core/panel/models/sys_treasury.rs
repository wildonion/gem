
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::NaiveDateTime;
use crate::adapters::nftport;
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN, COLLECTION_NOT_FOUND_FOR_CONTRACT};
use crate::{*, constants::COLLECTION_NOT_FOUND_OF};
use super::users::User;
use super::users_galleries::{UserPrivateGalleryData, UserPrivateGallery, UpdateUserPrivateGallery, UpdateUserPrivateGalleryRequest};
use super::users_nfts::UserNftData;
use crate::schema::sys_treasury::dsl::*;
use crate::schema::sys_treasury;
use crate::models::users::UserWalletInfoResponse;

/* 

    diesel migration generate sys_treasury        ---> create sys_treasury migration sql files
    diesel migration run                         ---> apply sql files to db 
    diesel migration redo                        ---> drop tables 

*/
#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=sys_treasury)]
pub struct SysTreasury{
    pub id: i32,
    pub airdrop: i64,
    pub debit: i64,
    pub paid_to: i32,
    pub current_networth: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(Insertable, PartialEq)]
#[diesel(table_name=sys_treasury)]
pub struct SysTreasuryRequest{
    pub airdrop: i64,
    pub debit: i64,
    pub paid_to: i32,
    pub current_networth: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserWalletSysTreasuryRequest{
    pub id: i32,
    pub airdrop: i64,
    pub debit: i64,
    pub paid_to: i32,
    pub current_networth: i64,
    pub wallet_info: UserWalletInfoResponse,
    pub created_at: String,
    pub updated_at: String,
}