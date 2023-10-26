


use crate::*;



#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserNft{
    pub id: i32,
    pub contract_address: String,
    pub current_owner: String, // the screen_cid of current owner of this nft
    pub metadata: String, // json stringified like statistical data
    pub img_url: String,
    pub onchain_id: Option<String>, // fulfilled after minting
    pub name: String,
    pub is_minted: bool, // if it's false means that is not stored on contract yet
    pub description: String,
    pub current_price: Option<i64>,
    pub is_listed: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserNftData{
    pub id: i32,
    pub contract_address: String,
    pub current_owner: String, // the screen_cid of current owner of this nft
    pub metadata: String, // json stringified like statistical data
    pub img_url: String,
    pub onchain_id: Option<String>, // fulfilled after minting
    pub name: String,
    pub is_minted: bool, // if it's false means that is not stored on contract yet
    pub description: String,
    pub current_price: Option<i64>,
    pub is_listed: bool,
    pub created_at: String,
    pub updated_at: String,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl UserNft{

    pub async fn insert(){

    }

    pub async fn get_info_of(asset_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn update_price(asset_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn update_metadata(asset_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

    pub async fn update_listing(asset_id: i32, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<(), PanelHttpResponse>{

        Ok(())

    }

}