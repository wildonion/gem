


use crate::{*, schema::users_galleries};
use super::users_collections::{UserCollection, UserCollectionData};



/* 

    diesel migration generate users_galleries ---> create users_galleries migration sql files
    diesel migration run                      ---> apply sql files to db 
    diesel migration redo                     ---> drop tables 

*/
#[derive(Queryable, Selectable, Serialize, Deserialize, Insertable, Identifiable, Debug, PartialEq, Clone)]
#[diesel(table_name=users_galleries)]
pub struct UserPrivateGallery{
    pub id: i32,
    pub owner_screen_cid: String,
    pub collections: serde_json::Value, /* pg key, value based json binary object */
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Vec<String>,
    pub metadata: serde_json::Value, /* pg key, value based json binary object */
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserPrivateGalleryData{
    pub id: i32,
    pub owner_screen_cid: String,
    pub collections: serde_json::Value,
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateUserPrivateGalleryRequest{
    pub owner_screen_cid: String,
    pub collections: serde_json::Value,
    pub gal_name: String,
    pub gal_description: String,
    pub invited_friends: Vec<String>,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NewUserPrivateGalleryRequest{
    pub owner_screen_cid: String,
    pub gal_name: String,
    pub gal_description: String,
    pub metadata: serde_json::Value,
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

    pub async fn insert(new_gallery_info: NewUserPrivateGalleryRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        
        Ok(())

    }

    pub async fn send_invitation_request_to(screen_cid: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
            
        // check that a user with the passed in screen_cid is the caller's friend or not
        // also check that the passed in screen_cid has accepted the friend request of the caller
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
    pub async fn update(caller_screen_cid: &str, new_collection_data: Option<serde_json::Value>,
        gallery_info: UpdateUserPrivateGalleryRequest, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{
        
        // condition: caller_screen_cid == gallery_info.owner_screen_cid

        // insert new collection data into the user gallery
        // check that new_collection_data.owner_screen_cid is equals to the gallery_info.owner_screen_cid
        // if new_collection_data.is_some(){
        // let mut decoded_cols = serde_json::from_value::<Vec<UserCollectionData>>(gallery_info.collections).unwrap();
        // decoded_cols.push(serde_json::to_value(&new_collection_data.unwrap()));
        // update gal record
        // }
        // ...

        Ok(())

    }
}