


use crate::*;



#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserPrivateGallery{
    pub id: i32,
    pub owner_cid: String, // the screen_cid of the gallery owner
    pub collections: Vec<i32>,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserPrivateGalleryData{
    pub id: i32,
    pub owner_cid: String, // the screen_cid of the gallery owner
    pub collections: Vec<i32>,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserPrivateGallery{

    pub async fn insert(connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{



        // check that a user with the passed in screen_cid is in db or not
        // ...

        Ok(())

    }

    pub async fn get_all_for(screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn update(gallery_info: UserPrivateGalleryData, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

}