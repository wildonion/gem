







use futures_util::TryStreamExt; /* TryStreamExt can be used to call try_next() on future object */
use mongodb::bson::oid::ObjectId;
use crate::*;
use crate::events::redis::role::PlayerRoleInfo;
use crate::models::{users::*, tasks::*, users_tasks::*};
use crate::resp;
use crate::constants::*;
use crate::misc::*;
use crate::schema::users::dsl::*;
use crate::schema::users;
use crate::schema::tasks::dsl::*;
use crate::schema::tasks;
use crate::schema::users_tasks::dsl::*;
use crate::schema::users_tasks;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};






#[derive(Serialize, Deserialize)]
struct YouWhoId;


#[derive(Serialize, Deserialize)]
struct Metadata{
    pub from: YouWhoId,
    pub recipient: YouWhoId,
    pub amount: u64,
    pub iat: i64,
}

#[post("/mint")]
#[passport(admin, dev, user)]
async fn mint_mock(
    req: HttpRequest,
    metadata: web::Json<Metadata>,
    storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();

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
            let metadata_amount = metadata.amount;
            let mut interval = tokio::time::interval(TokioDuration::from_secs(10));
            
            /* 
                since we need to access the tx mint hash outside of the tokio::spawn()
                thus we have to use tokio jobq channel to fill it inside the tokio green
                threadpool then use it outside of it by receiving from the channel
            */
            let (mint_tx_hash_sender, 
                mut mint_tx_hash_receiver) = 
                tokio::sync::oneshot::channel::<String>();

            /* spawning an async task in the background to do the payment and minting logics */
            tokio::spawn(async move{
                
                let mut contract_mint_call = false;

                loop{
                    
                    interval.tick().await;

                    // 1 - save metdata into db 
                    // 2 - call coinbase api to buy metadata_amount 
                    // 3 - mint card on chain by depositting 
                    // 4 - send tx hash to sender and receiver 

                    if contract_mint_call == true{
                        let mint_tx_hash = String::from("card minted this is tx hash");
                        /* 
                            since the send method is not async, it can be used anywhere
                            which means we can call it once in each scope cause it has 
                            no clone() method, the clone() method must be implemented for
                            future objects because we dont't know when they gets solved 
                            and we might move them between other scopes to await on them.
                        */
                        mint_tx_hash_sender.send(mint_tx_hash);
                        break;
                    }

                }

            });

            let mint_tx_hash = mint_tx_hash_receiver.try_recv().unwrap();

            resp!{
                String, // the data type
                mint_tx_hash, // response data
                MINTED_SUCCESSFULLY, // response message
                StatusCode::CREATED, // status code
                None::<Cookie<'_>>, // cookie
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




pub mod exports{
    pub use super::mint_mock;
}