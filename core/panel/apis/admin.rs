





use redis::AsyncCommands;
use actix_web::HttpMessage;
use futures_util::TryStreamExt; /* TryStreamExt can be used to call try_next() on future object */
use mongodb::bson::oid::ObjectId;
use crate::*;
use crate::events::publishers::role::{PlayerRoleInfo, Reveal};
use crate::models::clp_events::{ClpEvent, NewClpEventRequest};
use crate::models::users_checkouts::{UserCheckout, UserCheckoutData};
use crate::models::users_collections::{NewUserCollectionRequest, UserCollectionData, UserCollection, UpdateUserCollectionRequest, CollectionInfoResponse};
use crate::models::users_deposits::{UserDeposit, UserDepositData, UserDepositDataWithWalletInfo};
use crate::models::users_withdrawals::{UserWithdrawal, UserWithdrawalData};
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::helpers::passport::Passport;
use crate::resp;
use crate::constants::*;
use crate::helpers::misc::*;
use s3req::Storage;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use crate::schema::users_tasks::dsl::*;
use crate::schema::users_tasks;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use self::models::clp_events::UpdateClpEventRequest;



pub mod auth;
pub mod wallet;
pub mod clp;
pub mod mail;
pub mod rendezvous;
pub mod task;
pub mod user;
pub mod x;


//  -------------------------
// |   component setups
// | ------------------------
// |


// fn pointer method, futures must be pinned at a fixed position on the heap 
// to avoid getting invalidated pointers even after moving the type
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
pub struct AdminComponentActor{
    pub state: Option<ComponentState>,
    pub apis: Vec<Api>
}


