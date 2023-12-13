

use std::time::{SystemTime, UNIX_EPOCH};
use chrono::NaiveDateTime;
use crate::adapters::nftport;
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN, COLLECTION_NOT_FOUND_FOR_CONTRACT};
use crate::misc::{Response, Limit};
use crate::{*, constants::COLLECTION_NOT_FOUND_OF};
use super::users::User;
use super::users_galleries::{UserPrivateGalleryData, UserPrivateGallery, UpdateUserPrivateGallery, UpdateUserPrivateGalleryRequest};
use super::users_nfts::UserNftData;
use crate::schema::clp_events::dsl::*;
use crate::schema::clp_events;


/* 

    diesel migration generate clp_events        ---> create clp_events migration sql files
    diesel migration run                        ---> apply sql files to db 
    diesel migration redo                       ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=clp_events)]
pub struct ClpEvent{
    pub id: i32,
    pub contract_address: String,
    pub nfts: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub event_name: String,
    pub symbol: String,
    pub max_supply: i32,
    pub mint_price: i64,
    pub presale_mint_price: i64,
    pub tokens_per_mint: i32,
    pub treasury_address: String,
    pub public_mint_start_date: String,
    pub presale_mint_start_date: String,
    pub presale_whitelisted_addresses: Vec<String>,
    pub prereveal_token_uri: String,
    pub start_at: i64,
    pub expire_at: i64,
    pub is_locked: bool,
    pub owner_screen_cid: String,
    pub metadata_updatable: Option<bool>,
    pub freeze_metadata: Option<bool>,
    pub base_uri: String,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub event_background: String,
    pub extra: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub event_description: String,
    pub contract_tx_hash: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl ClpEvent{

    pub async fn find_by_id(clp_event_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Self, PanelHttpResponse>{


            todo!()
        }


}