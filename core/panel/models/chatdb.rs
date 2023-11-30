




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

        let r1_wallet = walletreq::secp256r1::generate_new_wallet();
        let mut ed_wallet = walletreq::ed25519::generate_new_wallet();

        let mut default_aes256_condif = walletreq::ed25519::get_default_aes256_config();
        default_aes256_condif.secret_key = misc::gen_random_chars(64); /*** ---- secret key must be 64 bytes or 512 bits */
        default_aes256_condif.nonce = misc::gen_random_chars(16); /*** ---- secret key must be 16 bytes or 128 bits */
        default_aes256_condif.data = text.as_bytes().to_vec();

        let encrypted_text = ed_wallet.self_generate_aes256_from(default_aes256_condif.clone());        
        default_aes256_condif.data = encrypted_text.clone(); /* update data field with encrypted form of raw data */
        let dec = ed_wallet.self_generate_data_from_aes256(default_aes256_condif.clone());
        
        let raw_text = std::str::from_utf8(&dec).unwrap();
        info!("aes256 decrypted data :::: {:?}", raw_text);
        info!("default_aes256_condif.secret_key :::: {:?}", default_aes256_condif.secret_key);
        info!("default_aes256_condif.nonce :::: {:?}", default_aes256_condif.nonce);


        // -------------------------------
        // TODO - store in chatdb by calling wasm methods
        // end2enc chat encryption using ed_wallet aes256
        // see ed25519_aes256_test() method
        // ...

        todo!()

    }


}