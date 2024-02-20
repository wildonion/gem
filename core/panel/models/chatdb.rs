


use std::time::{SystemTime, UNIX_EPOCH};
use chrono::NaiveDateTime;
use spacetimedb_sdk::spacetimedb_lib::Hash;
use crate::adapters::{nftport, openai};
use crate::constants::{COLLECTION_NOT_FOUND_FOR, INVALID_QUERY_LIMIT, GALLERY_NOT_OWNED_BY, CANT_GET_CONTRACT_ADDRESS, USER_NOT_FOUND, USER_SCREEN_CID_NOT_FOUND, COLLECTION_UPLOAD_PATH, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE, STORAGE_IO_ERROR_CODE, COLLECTION_NOT_OWNED_BY, CANT_CREATE_COLLECTION_ONCHAIN, INVALID_CONTRACT_TX_HASH, CANT_UPDATE_COLLECTION_ONCHAIN, COLLECTION_NOT_FOUND_FOR_CONTRACT};
use crate::helpers::misc::{Response, Limit};
use crate::models::users_clps::UserClp;
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
    
    pub async fn store(event_id: i32, user_screen_cid: &str, user_text: &str, // user_text is a raw text
            connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserChat, PanelHttpResponse>{

        let get_user = User::find_by_screen_cid(user_screen_cid, connection).await;
        let Ok(user) = get_user else{
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };
        
        // start processing in the background asyncly without having 
        // any disruption in other async methods order of execution
        tokio::spawn(async move{
            
            // https://spacetimedb.com/docs/sdks/rust/quickstart
            // TODO - store text in chatdb by calling wasm methods
            // TODO - consider n.chat per user limit
            // TODO - test themis wasm in js
            // ...

        });
        

        Ok(
            UserChat::default()
        )

    }

    /* ----------- use case:
        use the following method for ai summarization to generate a title 
        and then generate image from the title then mint it to user_screen_cid 
       ----------- */
    pub async fn get_all_chats_of(user_screen_cid: &str)
        -> Result<Vec<String>, PanelHttpResponse>{

            Ok(
                vec![]
            )

    }

    pub async fn start_summarizing_chats(users_in_event: Vec<User>)
        -> Result<HashMap<i32, String>, String>{

        let mut users_titles_map = HashMap::new();
        for user in users_in_event{
            let user_chats = Self::get_all_chats_of(&user.screen_cid.unwrap()).await.unwrap();
            let title = openai::summarize::create_titles_from(&user_chats.as_slice()).await;
            users_titles_map.insert(user.id, title);
        }
        Ok(users_titles_map)
    }

    pub async fn start_generating_ai_images(users_titles_map: HashMap<i32, String>) -> Result<HashMap<i32, String>, String>{

        let mut users_image_map = HashMap::new();
        for (user_id, title) in users_titles_map{
            users_image_map.insert(
                user_id,
                openai::generate::create_image_from(&title).await
            );
        }

        Ok(users_image_map)

    }


}