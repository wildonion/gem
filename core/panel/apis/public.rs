




use crate::*;
use crate::models::users_tickets::{UserTicket, NewUserTicketRequest};
use crate::models::users_collections::{UserCollectionData, UserCollection, CollectionInfoResponse};
use crate::models::users_nfts::{UserNftData, UserNft, NftLike, LikeUserInfo, UserLikeStat, NftUpvoterLikes, NftColInfo, UserCollectionDataGeneralInfo};
use crate::schema::users_galleries::dsl::users_galleries;
use crate::models::users_galleries::{UserPrivateGallery, UserPrivateGalleryData};
use crate::models::{users::*, tasks::*, users_tasks::*, xbot::*};
use crate::resp;
use crate::constants::*;
use crate::helpers::misc::*;
use actix_web::web::Payload;
use bytes::Buf;
use chrono::NaiveDateTime;
use rand::seq::SliceRandom;
use s3req::Storage;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;



pub mod blockchain;
pub mod search;
pub mod wallet;
pub mod x;
pub mod stream;
pub mod task;
pub mod ticket;


//  -------------------------
// |   component setups
// | ------------------------
// |


// fn pointer method, futures must be pinned at a fixed position on the heap 
// to avoid getting invalidated pointers even after moving the type.
// fn is a pointer to a function can be used to specifiy the type of a var.
type Method = fn(HttpRequest, AppState) -> std::pin::Pin<Box<dyn futures::Future<Output = PanelHttpResponse>>>;

#[derive(Clone)]
pub enum ComponentState{
    Halted,
    Executed,
}

#[derive(Clone)]
pub struct Api{
    pub route: String, 
    pub method: Method,
    pub last_response: Option<serde_json::Value> // last response json value caught throughout the api calling
}

#[derive(Clone)]
pub struct PublicComponentActor{
    pub app_storage: Option<Arc<Storage>>,
    pub state: Option<ComponentState>,
    pub apis: Vec<Api>
}