







use futures_util::TryStreamExt; /* TryStreamExt can be used to call try_next() on future object */
use mongodb::bson::oid::ObjectId;
use ring::signature::KeyPair;
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
use models::public::Id as IdGenerator;
use models::public::{WithdrawMetadata, DepositMetadata};





#[post("/id/build")]
async fn make_id(
    req: HttpRequest,
    mut id_: web::Json<IdGenerator>,
    storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();
    let get_redis_conn = redis_client.get_async_connection().await;
    let Ok(mut redis_conn) = get_redis_conn else{

        let redis_get_conn_error = get_redis_conn.err().unwrap();
        let redis_get_conn_error_string = redis_get_conn_error.to_string();
        use error::{ErrorKind, StorageError::Redis, PanelError};
        let error_content = redis_get_conn_error_string.as_bytes().to_vec(); /* extend the empty msg_content from the error utf8 slice */
        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Redis(redis_get_conn_error)));
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

        resp!{
            &[u8], // the date type
            &[], // the data itself
            &redis_get_conn_error_string, // response message
            StatusCode::INTERNAL_SERVER_ERROR, // status code
            None::<Cookie<'_>>, // cookie
        }

    };


    /* ECDSA keypair */
    let key_pair = gen_ec_key_pair(); // generates a pair of Elliptic Curve (ECDSA) keys
    let (private, public) = key_pair.clone().split();
    let ec_signer = SecureSign::new(private.clone());
    let ec_verifier = SecureVerify::new(public.clone());
    id_.0.unique_id = Some(hex::encode(public.as_ref()));
    id_.0.signer = Some(hex::encode(private.as_ref()));


    /* generating snowflake id */
    let machine_id = std::env::var("MACHINE_ID").unwrap_or("1".to_string()).parse::<i32>().unwrap();
    let node_id = std::env::var("NODE_ID").unwrap_or("1".to_string()).parse::<i32>().unwrap();
    let mut id_generator_generator = SnowflakeIdGenerator::new(machine_id, node_id);
    let snowflake_id = id_generator_generator.real_time_generate();
    id_.snowflake_id = Some(snowflake_id);


    /* stringifying the id_ instance to generate the signature */
    let json_input = serde_json::json!({
        "paypal_id": id_.paypal_id,
        "account_number": id_.account_number,
        "social_id": id_.social_id,
        "username": id_.username,
        "snowflake_id": snowflake_id,
        "unique_id": id_.0.unique_id,
    });
    let inputs_to_sign = serde_json::to_string(&json_input).unwrap(); /* json stringifying the json_input value */


    /* generating signature from the input data */
    let ec_sig = ec_signer.sign(inputs_to_sign.as_bytes()).unwrap();
    id_.0.signature = Some(hex::encode(&ec_sig));
    let encoded_id = ec_verifier.verify(ec_sig).unwrap();
    let decoded_id = serde_json::from_slice::<IdGenerator>(&encoded_id).unwrap();
    

    /* storing the generated unique id inside the redis ram */
    let redis_stringified_inputs = serde_json::to_string(&id_).unwrap();
    let _: () = redis_conn.set(id_.username.as_str(), redis_stringified_inputs.as_str()).await.unwrap();
;
    resp!{
        IdGenerator, // the data type
        id_.0, // response data
        ID_BUILT, // response message
        StatusCode::CREATED, // status code
        None::<Cookie<'_>>, // cookie
    }


}

#[post("/deposit")]
async fn deposit(
    req: HttpRequest,
    metadata: web::Json<DepositMetadata>,
    storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();
            let mut interval = tokio::time::interval(TokioDuration::from_secs(10));
            
            /* 
                since we need to access the tx mint hash outside of the tokio::spawn()
                thus we have to use tokio jobq channel to fill it inside the tokio green
                threadpool then use it outside of it by receiving from the channel
            */
            let (deposit_tx_hash_sender, 
                mut deposit_tx_hash_receiver) = 
                tokio::sync::oneshot::channel::<String>();

            /* spawning an async task in the background to do the payment and minting logics */
            tokio::spawn(async move{
                
                let mut contract_mint_call = false;

                loop{
                    
                    interval.tick().await;

                    /* 
                        ------------------------------------
                           THE DEPOSIT API (Sender Only)
                        ------------------------------------
                        
                        0 => sender pay the exchange with the amounts 
                        1 => exchange sends the paid amount to the coinbase usdc/usdt server wallet 
                        2 => send successful response to the sender contains tx hash of depositting into the coinbase
                        
                    */ 

                    if contract_mint_call == true{
                        let deposit_tx_hash = String::from("card minted this is tx hash");
                        /* 
                            since the send method is not async, it can be used anywhere
                            which means we can call it once in each scope cause it has 
                            no clone() method, the clone() method must be implemented for
                            future objects because we dont't know when they gets solved 
                            and we might move them between other scopes to await on them.
                        */
                        deposit_tx_hash_sender.send(deposit_tx_hash);
                        break;
                    }

                }

            });

            let deposit_tx_hash = deposit_tx_hash_receiver.try_recv().unwrap();

            resp!{
                String, // the data type
                deposit_tx_hash, // response data
                DEPOSITED_SUCCESSFULLY, // response message
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


#[post("/withdraw")]
async fn withdraw(
    req: HttpRequest,
    metadata: web::Json<WithdrawMetadata>,
    storage: web::Data<Option<Arc<Storage>>> // shared storage (none async redis, redis async pubsub conn, postgres and mongodb)
) -> PanelHttpResponse{

    
    let storage = storage.as_ref().to_owned(); /* as_ref() returns shared reference */
    let redis_client = storage.as_ref().clone().unwrap().get_redis().await.unwrap();


    match storage.clone().unwrap().as_ref().get_pgdb().await{

        Some(pg_pool) => {

            let connection = &mut pg_pool.get().unwrap();


            /* 

                -----------------------------------------
                    THE WITHDRAW API (Receiver Only)
                -----------------------------------------
                        
                0 => call coinbase trade api to exchange usdt/usdc to the passed in currency type 
                1 => send the traded to paypal wallet of the server  
                2 => send the amount from the server paypal to the receiver paypal
                
            */ 


            resp!{
                &[u8], // the data type
                &[], // response data
                CLAIMED_SUCCESSFULLY, // response message
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
    pub use super::deposit;
    pub use super::withdraw;
    pub use super::make_id;
}