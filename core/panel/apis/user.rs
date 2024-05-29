


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


#[get("/user/get/all/treasury/{uid}")]
#[passport(user)]
pub async fn get_all_user_treasuries(
    req: HttpRequest,
    uid: web::Path<i32>,
    storage: web::Data<Option<Arc<Storage>>>, // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{


    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;


    /* 
          ------------------------------------- 
        | --------- PASSPORT CHECKING --------- 
        | ------------------------------------- 
        | granted_role has been injected into this 
        | api body using #[passport()] proc macro 
        | at compile time thus we're checking it
        | at runtime
        |
    */
    let granted_role = 
        if granted_roles.len() == 3{ /* everyone can pass */
            None /* no access is required perhaps it's an public route! */
        } else if granted_roles.len() == 1{
            match granted_roles[0]{ /* the first one is the right access */
                "admin" => Some(UserRole::Admin),
                "user" => Some(UserRole::User),
                _ => Some(UserRole::Dev)
            }
        } else{ /* there is no shared route with eiter admin|user, admin|dev or dev|user accesses */
            resp!{
                &[u8], // the data type
                &[], // response data
                ACCESS_DENIED, // response message
                StatusCode::FORBIDDEN, // status code
                None::<Cookie<'_>>, // cookie
            }
        };

    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;

                    /* caller must have an screen_cid */
                    let user = User::find_by_id(_id, connection).await.unwrap();
                    if user.cid.is_none(){
                        resp!{
                            &[u8], //// the data type
                            &[], //// response data
                            USER_SCREEN_CID_NOT_FOUND, //// response message
                            StatusCode::NOT_ACCEPTABLE, //// status code
                            None::<Cookie<'_>>, //// cookie
                        }
                    }
                    
                    use crate::models::user_treasury::UserTreasury;
                    use crate::models::user_treasury::UserWalletTreasuryRequest;
                    use crate::schema::user_treasury;
                    use crate::schema::user_treasury::dsl::*;

                    let users_utreasury_data = user_treasury
                        .filter(user_treasury::user_id.eq(_id))
                        .load::<UserTreasury>(connection);
                        
                    let Ok(utreasury) = users_utreasury_data else{
                        let resp = Response{
                            data: Some(_id),
                            message: &format!("User Has No Treasury Yet"),
                            status: 404,
                            is_error: true
                        };
                        return Ok(
                            HttpResponse::NotFound().json(resp)
                        )
                    };

                    
                    let ut_vector = utreasury
                        .into_iter()
                        .map(|ut| {
                            UserWalletTreasuryRequest{
                                id: ut.id,
                                user_id: ut.user_id,
                                done_at: ut.done_at,
                                amount: ut.amount,
                                tx_type: ut.tx_type,
                                wallet_info: UserWalletInfoResponse{
                                    username: user.clone().username,
                                    avatar: user.clone().avatar,
                                    bio: user.clone().bio,
                                    banner: user.clone().banner,
                                    mail: user.clone().mail,
                                    screen_cid: user.clone().screen_cid,
                                    extra: user.clone().extra,
                                    stars: user.clone().stars,
                                    created_at: user.clone().created_at.to_string(),
                                },
                                treasury_type: ut.treasury_type,
                            }
                        }).collect::<Vec<UserWalletTreasuryRequest>>();
                
                    resp!{
                        Vec<UserWalletTreasuryRequest>, //// the data type
                        ut_vector, //// response data
                        &format!("Fetched User Treasuries"), //// response message
                        StatusCode::OK, //// status code
                        None::<Cookie<'_>>, //// cookie
                    }

                },
                Err(resp) => {
                    
                    /* 
                        🥝 response can be one of the following:
                        
                        - NOT_FOUND_COOKIE_VALUE
                        - NOT_FOUND_TOKEN
                        - INVALID_COOKIE_TIME_HASH
                        - INVALID_COOKIE_FORMAT
                        - EXPIRED_COOKIE
                        - USER_NOT_FOUND
                        - NOT_FOUND_COOKIE_TIME_HASH
                        - ACCESS_DENIED, 
                        - NOT_FOUND_COOKIE_EXP
                        - INTERNAL_SERVER_ERROR 
                    */
                    resp
                }
            }

        },
        None => {

            resp!{
                &[u8], // the data type
                &[], // response data
                STORAGE_ISSUE, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                None::<Cookie<'_>>, // cookie
            }
        }
    }

}


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