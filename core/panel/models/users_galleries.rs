


use crate::*;
use super::users_collections::{UserCollection, UserCollectionData};



#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserPrivateGallery{
    pub id: i32,
    pub owner_screen_cid: String,
    pub collections: Vec<i32>,
    pub name: String,
    pub description: String,
    pub invited_friends: Vec<String>,
    pub metadata: String, // json stringified data like gallery statistics
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserPrivateGalleryData{
    pub id: i32,
    pub owner_screen_cid: String,
    pub collections: Vec<UserCollectionData>,
    pub name: String,
    pub description: String,
    pub invited_friends: Vec<String>,
    pub metadata: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserPrivateGalleryData{
    pub id: i32,
    pub owner_screen_cid: String,
    pub collections: Vec<i32>,
    pub name: String,
    pub description: String,
    pub invited_friends: Vec<String>,
    pub metadata: String,
    pub created_at: String,
    pub updated_at: String,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserPrivateGallery{

    pub async fn get_all_for(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserPrivateGalleryData>, PanelHttpResponse>{

        // if the caller is a friend of the gallery owner and is inside the invited_friends vector
        // then he can see all his galleries' collections contain none minted nfts
        // ...

        // let user_collection_data = UserCollectionData::get_all_private_collections_for()

        Ok(
            vec![UserPrivateGalleryData::default()]
        )

    }

    pub async fn get_invited_friends_of(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>)
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

}

impl UserPrivateGallery{

    pub async fn insert(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        // name is not unique
        // ...
        
        Ok(())

    }

    pub async fn send_invitation_request_to(screen_cid: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
            
        // check that a user with the passed in screen_cid is the caller's friend or not
        // update invited_friends field with the passed in screen_cid
        // call UserFan::push_invitation_request_for()
        // ...

        Ok(())

    }

    pub async fn remove_invited_friend_from(screen_cid: &str, gallery_id: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
            
        // check that a user with the passed in screen_cid is the caller's friend or not
        // remove from invited_friends field with the passed in screen_cid
        // ...

        Ok(())

    }

    /* ------------------------------------------------------------------------------- */
    /* this method can be called to update an gallery status like name and description */
    /* ------------------------------------------------------------------------------- */
    /* supported apis:
        - update_private_gallery
    */
    pub async fn update(caller_screen_cid: &str, 
        gallery_info: UpdateUserPrivateGalleryData, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
        
        // condition: caller_screen_cid == gallery_info.owner_screen_cid

        Ok(())

    }
}