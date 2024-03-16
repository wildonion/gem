

use crate::*;
use crate::adapters::nftport::{self, NftExt, OnchainNfts};
use crate::constants::{GALLERY_NOT_OWNED_BY, NFT_NOT_OWNED_BY, NFT_UPLOAD_PATH, INVALID_QUERY_LIMIT, STORAGE_IO_ERROR_CODE, NFT_ONCHAINID_NOT_FOUND, NFT_UPLOAD_ISSUE, CANT_MINT_CARD, CANT_MINT_NFT, CANT_TRANSFER_NFT, NFT_EVENT_TYPE_RECIPIENT_IS_NEEDED, NFT_EVENT_TYPE_METADATA_URI_IS_NEEDED, INVALID_NFT_EVENT_TYPE, NFT_IS_NOT_MINTED_YET, CANT_UPDATE_NFT, NFT_NOT_FOUND_OF, NFT_IS_ALREADY_MINTED, NFT_IS_NOT_LISTED_YET, NFT_PRICE_IS_EMPTY, NFT_EVENT_TYPE_BUYER_IS_NEEDED, CALLER_IS_NOT_BUYER, INVALID_NFT_ROYALTY, INVALID_NFT_PRICE, RECIPIENT_SCREEN_CID_NOT_FOUND, EMPTY_NFT_IMG, NFT_NOT_FOUND_OF_ID, USER_SCREEN_CID_NOT_FOUND, NFT_METADATA_URI_IS_EMPTY, NFT_IS_NOT_LISTED, NOT_FOUND_NFT, NFT_IS_NOT_OWNED_BY_THE_PASSED_IN_OWNER};
use crate::events::publishers::action::{SingleUserNotif, NotifData, ActionType};
use crate::helpers::misc::{Response, Limit};
use crate::schema::users_tickets::dsl::*;
use crate::schema::users_tickets;
use self::constants::{APP_NAME, NO_LIKE_FOUND_FOR_THIS_NFT, NO_LIKE_FOUND_FOR_THIS_USER};
use super::users::{User, UserData, UserWalletInfoResponse};
use super::users_collections::{UserCollection, UserCollectionData, UpdateUserCollection};
use super::users_fans::{UserFan, FriendData};
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest, UserPrivateGalleryData};
use crate::schema::users_collections::dsl::*;
use crate::schema::users_collections;

/* 

    diesel migration generate users_tickets ---> create users_tickets migration sql files
    diesel migration run                    ---> apply sql files to db 
    diesel migration redo                   ---> drop tables 

*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_tickets)]
pub struct UserTicket{
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub cname: String,
    pub mail: String,
    pub cdescription: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[derive(Insertable)]
#[diesel(table_name=users_tickets)]
pub struct NewUserTicketRequest{
    pub user_id: i32,
    pub title: String,
    pub cname: String,
    pub mail: String,
    pub cdescription: String,
}

impl UserTicket{
    
    pub async fn insert(ticket_request: NewUserTicketRequest, connection: &mut DbPoolConnection)
        -> Result<Self, helpers::error0::PanelErrorResponse>{            
        
        // we can use ? operator to unwrap the actual error to an actix http response cause:
        // the Error trait is implemented for PanelErrorResponse
        // the From<diesel::result::Error> is implemented for PanelErrorResponse
        // the Debug, Display is implemented for StorageError and PanelErrorResponse
        let ticket_data = diesel::insert_into(users_tickets)
            .values(&ticket_request)
            .returning(UserTicket::as_returning())
            .get_result::<UserTicket>(connection)?;

        Ok(ticket_data)

    }

    // admin access
    pub async fn get_all(limit: web::Path<Limit>, connection: &mut DbPoolConnection) 
        -> Result<Vec<Self>, PanelHttpResponse>{

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

        let users_tickets_data = users_tickets
            .order(users_tickets::created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserTicket>(connection);
            
        let Ok(tickets) = users_tickets_data else{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: NO_TICKET_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        Ok(
            tickets
        )
        
    }
    
}