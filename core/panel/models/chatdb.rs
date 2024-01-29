




use std::time::{SystemTime, UNIX_EPOCH};
use chrono::NaiveDateTime;
use crate::adapters::nftport;
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN, COLLECTION_NOT_FOUND_FOR_CONTRACT};
use crate::misc::{Response, Limit};
use crate::{*, constants::COLLECTION_NOT_FOUND_OF};
use super::users::User;
use super::users_galleries::{UserPrivateGalleryData, UserPrivateGallery, UpdateUserPrivateGallery, UpdateUserPrivateGalleryRequest};
use super::users_nfts::UserNftData;
use crate::models::clp_events::ClpEvent;



#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserChat;

/* ---------------------------------------- */
/* ----- spacetimechatdb wasm methods ----- */
/* ---------------------------------------- */
impl UserChat{
    
    pub async fn store(event_id: i32, user_screen_cid: &str, user_text: &str, 
            //--- connection must be initialized once in server.rs and will be passed to different scopes
            //--- and passed through the routers' threads as a shared state data 
            connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserChat, PanelHttpResponse>{

        let get_user = User::find_by_screen_cid(user_screen_cid, connection).await;
        let Ok(user) = get_user else{
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };
        
        
        // https://spacetimedb.com/docs/sdks/rust/quickstart
        // TODO - store text in chatdb by calling wasm methods
        // TODO - consider n.chat per user limit
        // TODO - test themis wasm in js
        // ...


        Ok(
            UserChat::default()
        )

    }

    /* use the following method for ai summarization to generate a title */
    pub async fn get_all_chats_of(user_screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<Vec<UserChat>, PanelHttpResponse>{

            let get_user = User::find_by_screen_cid(user_screen_cid, connection).await;
            let Ok(user) = get_user else{
                let err_resp = get_user.unwrap_err();
                return Err(err_resp);
            };

            Ok(
                vec![]
            )

        }


}