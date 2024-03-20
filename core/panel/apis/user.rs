


use crate::*;
use crate::adapters::stripe::{create_product, create_price, create_session, StripeCreateCheckoutSessionData};
use crate::events::publishers::action::{UserNotif, NotifExt};
use crate::events::subscribers::handlers::actors::notif::user::{UserListenerActor};
use crate::events::subscribers::handlers::actors::notif::system::{SystemActor, GetSystemUsersMap};
use crate::events::subscribers::handlers::actors::notif::action::{GetUsersNotifsMap, UserActionActor};
use crate::models::clp_events::{ClpEventData, ClpEvent};
use crate::models::users_checkouts::{UserCheckoutData, UserCheckout, NewUserCheckout};
use crate::models::users_clps::{UserClp, RegisterUserClpEventRequest, CancelUserClpEventRequest};
use crate::models::users_collections::{UserCollection, UserCollectionData, NewUserCollectionRequest, UpdateUserCollectionRequest};
use crate::models::users_deposits::{UserDepositData, UserDepositDataWithWalletInfo};
use crate::models::users_fans::{AcceptFriendRequest, AcceptInvitationRequest, EnterPrivateGalleryRequest, FriendData, InvitationRequestData, InvitationRequestDataResponse, RemoveFollower, RemoveFollowing, RemoveFriend, SendFriendRequest, UserFan, UserFanData, UserFanDataWithWalletInfo, UserRelations};
use crate::models::users_galleries::{UserPrivateGalleryInfoDataInvited, NewUserPrivateGalleryRequest, UpdateUserPrivateGalleryRequest, UserPrivateGallery, UserPrivateGalleryData, RemoveInvitedFriendFromPrivateGalleryRequest, SendInvitationRequest, UserPrivateGalleryInfoData, ExitFromPrivateGalleryRequest};
use crate::models::users_nfts::{AddReactionRequest, CreateNftMetadataUriRequest, NewUserNftRequest, NftReactionData, UpdateUserNftRequest, UserNft, UserNftData, UserNftDataWithWalletInfo, UserReactionData};
use crate::models::users_withdrawals::{UserWithdrawal, UserWithdrawalData};
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::helpers::passport::Passport; /* loading Passport macro to use get_user() method on HttpRequest object */
use crate::resp;
use crate::constants::*;
use crate::helpers::misc::*;
use actix::Addr;
use chrono::NaiveDateTime;
use s3req::Storage;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use futures_util::TryStreamExt;
use crate::*;
use crate::models::users::UserRole;
use crate::constants::*;
use crate::helpers::misc::*;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use models::users::{Id, NewIdRequest, UserIdResponse};
use models::users_deposits::{NewUserDepositRequest, UserDeposit};
use models::users_withdrawals::NewUserWithdrawRequest;
use crate::adapters::nftport::*;
use crate::models::token_stats::TokenStatInfoRequest;
use self::models::token_stats::TokenStatInfo;




pub mod friend;
pub mod gallery;
pub mod clp;
pub mod auth;
pub mod leaderboard;
pub mod profile;
pub mod rendezvous;
pub mod task;
pub mod wallet;
pub mod x;



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
pub struct UserComponentActor{
    pub app_storage: Option<Arc<Storage>>,
    pub state: Option<ComponentState>,
    pub apis: Vec<Api>
}