

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
use crate::schema::clp_events::dsl::*;
use crate::schema::clp_events;


/* 

    diesel migration generate clp_events        ---> create clp_events migration sql files
    diesel migration run                        ---> apply sql files to db 
    diesel migration redo                       ---> drop tables 

*/
#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=clp_events)]
pub struct ClpEvent{
    pub id: i32,
    pub contract_address: String,
    pub event_name: String, 
    pub symbol: String, 
    pub max_supply: i32, 
    pub team_reserve: i32,
    pub mint_price: i64,
    pub presale_mint_price: i64,
    pub tokens_per_mint: i32, 
    pub owner_screen_cid: String, 
    pub treasury_address: String, 
    pub public_mint_start_date: String,
    pub metadata_updatable: Option<bool>,
    pub freeze_metadata: Option<bool>,
    pub base_uri: String,
    pub presale_mint_start_date: String,
    pub presale_whitelisted_addresses: Option<Vec<Option<String>>>,
    pub prereveal_token_uri: String,
    pub royalties_share: i32, 
    pub royalties_address_screen_cid: String, 
    pub event_background: String,
    pub extra: Option<serde_json::Value>,
    pub event_description: String,
    pub contract_tx_hash: Option<String>,
    pub start_at: i64,
    pub expire_at: i64,
    pub is_locked: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UpdateClpEventRequest{
    pub id: i32,
    pub contract_address: String,
    pub base_uri: String, /////////////// IMPORTANT after generating all AI images this must be updated with an ipfs url contains all nft images info
    pub public_mint_start_date: String,
    pub prereveal_token_uri: String,
    pub mint_price: i64,
    pub royalties_share: i32,
    pub royalties_address_screen_cid: String,
    pub extra: Option<serde_json::Value>,
    pub event_description: String,
    pub start_at: i64,
    pub expire_at: i64,
    pub is_locked: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct NewClpEventRequest{
    pub event_name: String, /////////////// IMPORTANT
    pub symbol: String, /////////////// IMPORTANT
    pub max_supply: i32, /////////////// IMPORTANT
    pub team_reserve: i32, /////////////// IMPORTANT
    pub mint_price: i64, /////////////// IMPORTANT minting price per NFT, in units of the chain's native token
    pub tokens_per_mint: i32, /////////////// IMPORTANT
    pub owner_screen_cid: String, /////////////// IMPORTANT
    pub treasury_address: String, /////////////// IMPORTANT
    pub public_mint_start_date: String, /////////////// IMPORTANT
    pub prereveal_token_uri: String, /////////////// IMPORTANT before generating all AI images this must be filled with an ipfs url contains a thumbnail for each nft
    pub royalties_share: i32, /////////////// IMPORTANT
    pub royalties_address_screen_cid: String, /////////////// IMPORTANT
    pub extra: Option<serde_json::Value>,
    pub event_description: String,
    pub start_at: i64,
    pub expire_at: i64,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
#[diesel(table_name=clp_events)]
pub struct ClpEventData{
    pub id: i32,
    pub contract_address: String,
    pub event_name: String,
    pub symbol: String,
    pub max_supply: i32,
    pub mint_price: i64,
    pub presale_mint_price: i64,
    pub tokens_per_mint: i32,
    pub start_at: i64,
    pub expire_at: i64,
    pub is_locked: bool,
    pub owner_screen_cid: String,
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

            let single_clp_event = clp_events
                .filter(clp_events::id.eq(clp_event_id))
                .first::<ClpEvent>(connection);
                            
            let Ok(event) = single_clp_event else{
                let resp = Response{
                    data: Some(clp_event_id),
                    message: CLP_EVENT_NOT_FOUND,
                    status: 404,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
            };

            Ok(event)

    }
    
    pub async fn get_all(connection: &mut PooledConnection<ConnectionManager<PgConnection>>, limit: web::Query<Limit>) 
        -> Result<Vec<ClpEvent>, PanelHttpResponse> {

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

        let all_clp_events = clp_events
            .order(created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .load::<ClpEvent>(connection);
            
        let Ok(events) = all_clp_events else{
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: NO_CLP_EVENT,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )
        };


        Ok(
            events
        )


    }

    // fetching the latest clp event info, the one that is about to be started
    pub async fn get_latest(connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<ClpEventData, PanelHttpResponse>{

        // fetch the latest event which is not locked yet and it's close to get started
        let not_locked_clp_events = clp_events::table
            .filter(is_locked.eq(false))
            .order(clp_events::start_at.asc()) // get the one which is about to start earlier 
            .first::<ClpEvent>(connection);
                        
        let Ok(clp_event) = not_locked_clp_events else{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: USER_CLP_EVENT_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(
            ClpEventData{
                id: clp_event.id,
                contract_address: clp_event.contract_address,
                event_name: clp_event.event_name,
                symbol: clp_event.symbol,
                max_supply: clp_event.max_supply,
                mint_price: clp_event.mint_price,
                presale_mint_price: clp_event.presale_mint_price,
                tokens_per_mint: clp_event.tokens_per_mint,
                start_at: clp_event.start_at,
                expire_at: clp_event.expire_at,
                is_locked: clp_event.is_locked,
                owner_screen_cid: clp_event.owner_screen_cid,
                event_background: clp_event.event_background,
                extra: clp_event.extra,
                event_description: clp_event.event_description,
                contract_tx_hash: clp_event.contract_tx_hash,
                created_at: clp_event.created_at,
                updated_at: clp_event.updated_at,
            }
        )

    }


}