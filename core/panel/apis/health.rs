


use crate::*;
use crate::adapters::stripe::StripeWebhookPayload;
use crate::models::users_checkouts::UserCheckout;
use crate::resp;
use crate::constants::*;
use crate::helpers::misc::*;
use s3req::Storage;
use helpers::passport::Passport;
use crate::models::users::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::models::{tasks::*, users_tasks::*};

pub mod check;
pub mod index;
pub mod logout;
pub mod password;
pub mod task;
pub mod webhook;



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
pub struct HealthComponentActor{
    pub app_storage: Option<Arc<Storage>>,
    pub state: Option<ComponentState>,
    pub apis: Vec<Api>
}