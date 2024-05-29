




use crate::models::users_tickets::{UserTicket, NewUserTicketRequest};
use crate::models::users_tokens::UserToken;
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
pub mod ticket;
pub mod token;



#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AirdropRequest{
    pub user_id: i32,
    pub amount: i64
}

#[post("/airdrop")]
#[passport(admin)]
pub async fn airdrop(
        req: HttpRequest, 
        airdrop_request: web::Json<AirdropRequest>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            

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


            /* ------ ONLY USER CAN DO THIS LOGIC ------ */
            match req.get_user(granted_role, connection).await{
                Ok(token_data) => {
                    
                    let _id = token_data._id;
                    let role = token_data.user_role;


                    let owner_id = airdrop_request.user_id;
                    let airdrop_amount = airdrop_request.amount;
                    let tx_type = format!("Airdrop|{}", airdrop_amount);

                    match User::update_balance(
                        owner_id,
                        &tx_type,
                        "Credit",
                        airdrop_amount,
                        redis_client.clone(),
                        redis_actix_actor.clone(),
                        connection
                    ).await{
                        Ok(updated_user_data) => {
                            resp!{
                                UserData, // the data type
                                updated_user_data, // response data
                                &format!("Airdrop Sent"), // response message
                                StatusCode::OK, // status code
                                None::<Cookie<'_>>, // cookie
                            }
                        },
                        Err(resp) => {
                            resp
                        }
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


#[get("/treasury/get/all")]
#[passport(admin)]
pub async fn get_all_system_treasuries(
    req: HttpRequest,
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


                    use crate::models::sys_treasury::SysTreasury;
                    use crate::models::users::UserWalletInfoResponse;
                    use crate::models::sys_treasury::UserWalletSysTreasuryRequest;
                    use crate::schema::sys_treasury;
                    use crate::schema::sys_treasury::dsl::*;

                    let systreasury = sys_treasury
                        .order(sys_treasury::created_at.desc())
                        .load::<SysTreasury>(connection);
                        
                    let Ok(systresuires) = systreasury else{
                        let resp = Response::<'_, &[u8]>{
                            data: Some(&[]),
                            message: &format!("No System Treasury Yet"),
                            status: 404,
                            is_error: true
                        };
                        return 
                            Ok(HttpResponse::NotFound().json(resp))
                        
                    };

                    let syst_vector = 
                        systresuires
                            .into_iter()
                            .map(|syst| {
                                UserWalletSysTreasuryRequest{
                                    id: syst.id,
                                    airdrop: syst.airdrop,
                                    debit: syst.debit,
                                    paid_to: syst.paid_to,
                                    current_networth: syst.current_networth,
                                    wallet_info: {
                                        let user = User::find_by_id_none_async(syst.paid_to, connection).unwrap();
                                        UserWalletInfoResponse{
                                            username: user.clone().username,
                                            avatar: user.clone().avatar,
                                            bio: user.clone().bio,
                                            banner: user.clone().banner,
                                            mail: user.clone().mail,
                                            screen_cid: user.clone().screen_cid,
                                            extra: user.clone().extra,
                                            stars: user.clone().stars,
                                            created_at: user.clone().created_at.to_string(),
                                        }
                                    },
                                    created_at: syst.created_at.to_string(),
                                    updated_at: syst.updated_at.to_string(),
                                }
                            }).collect::<Vec<UserWalletSysTreasuryRequest>>();

                    resp!{
                        Vec<UserWalletSysTreasuryRequest>, // the data type
                        syst_vector, // response data
                        &format!("Fetched System Treasuries"), // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
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

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct UpdateBalanceRequest{
    pub user_id: i32,
    pub new_balance: i64
}

#[post("/update-user-balance")]
#[passport(admin)]
pub async fn update_user_balance(
        req: HttpRequest, 
        update_balance_request: web::Json<UpdateBalanceRequest>,  
        storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
    ) -> PanelHttpResponse {

    let storage = storage.as_ref().to_owned();
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let redis_actix_actor = storage.as_ref().clone().unwrap().get_redis_actix_actor().await.unwrap();


    match storage.clone().unwrap().get_pgdb().await{
        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            

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


            let owner_id = update_balance_request.user_id;
            let airdrop_amount = update_balance_request.new_balance;

            match User::update_balance(
                owner_id,
                "WithdrawToken",
                "Debit",
                airdrop_amount,
                redis_client.clone(),
                redis_actix_actor.clone(),
                connection
            ).await{
                Ok(updated_user_data) => {
                    resp!{
                        UserData, // the data type
                        updated_user_data, // response data
                        &format!("Balance Updated"), // response message
                        StatusCode::OK, // status code
                        None::<Cookie<'_>>, // cookie
                    }
                },
                Err(resp) => {
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
pub struct AdminComponentActor{
    pub app_storage: Option<Arc<Storage>>,
    pub state: Option<ComponentState>,
    pub apis: Vec<Api>
}