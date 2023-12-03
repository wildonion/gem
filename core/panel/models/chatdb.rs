




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

    pub async fn store(event_id: i32, user_screen_cid: &str, text: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserChat, PanelHttpResponse>{

        let get_user = User::find_by_screen_cid(user_screen_cid, connection).await;
        let Ok(user) = get_user else{
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };

        fn get_name() -> String{ String::from("") }
        let callback = |func: fn() -> String|{
            func();
        };
        callback(get_name);


        // wasm, box pin, impl Trait | &dyn Trait, Send Sync Arc, Weak, Rc, RefCell, Mutex, RwLock
        // share the references of the type between threads using clone or & 

        // text is the decrypted message

        // -------------------------------
        // TODO - store in chatdb by calling wasm methods
        // TODO - n.chat per user limit
        // ...

        todo!()

    }


}