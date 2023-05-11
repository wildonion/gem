

use crate::constants::*;
use crate::misc;
use crate::constants::*;
use crate::passport; //// this has been imported from the misc inside the app.rs and we can simply import it in here using crate::passport
use crate::resp;
use crate::schemas; //// this has been imported from the misc inside the app.rs and we can simply import it in here using crate::resp
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; //// futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; //// it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response};
use mongodb::bson::doc;
use routerify::prelude::RequestExt; //// this trait is needed to call the data() method on the request object  
use redis::FromRedisValue;
use redis::JsonAsyncCommands;
use redis::cluster::ClusterClient;
use redis::AsyncCommands; //// this trait is required to be imported in here to call set() methods on the cluster connection
use redis::RedisResult;
use mongodb::Client;
use std::env;








// -------------------------------- not found controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------

pub async fn register_notif(req: hyper::Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{ //// the return type is hyper response

    let redis_conn = &req.data::<redis::Connection>().unwrap().to_owned();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    /*
        @params: 
            - @request       → hyper request object; since this struct doesn't implement Clone trait and we must pass it then return it
            - @storage       → instance inside the request object
            - @access levels → vector of access levels
    */
    match passport!{
        req,
        db.clone(),
        vec![DEV_ACCESS, ADMIN_ACCESS]
    } {
    
        Some(passport_data) => {

            let token_data = passport_data.0;
            let request = passport_data.1; //// the request object that is returned from the passport!{}
            let response = passport_data.2; //// the response object that might be fulfilled if anything went wrong

            if token_data.is_some() && response.is_none() && request.is_some(){ //// if the response was empty means we have the passport data since the response must be fulfilled in this route

                //// -------------------------------------------------------------------------------------
                //// ------------------------------- ACCESS GRANTED REGION -------------------------------
                //// -------------------------------------------------------------------------------------
                
                let token_data = token_data.unwrap();
                let _id = token_data.claims._id;
                let username = token_data.claims.username;
                let access_level = token_data.claims.access_level;
                let req = request.unwrap();

                let mut god_has_a_group = false;
                let group_filter = doc!{"god_id": _id.unwrap().to_string()};
                let groups = db.clone().database(&db_name).collection::<schemas::game::GroupInfo>("groups");
                match groups.find_one(group_filter, None).await.unwrap(){ //// first we have to check that the caller owns a group
                    Some(group_doc) => god_has_a_group = true, //// if we're here means the caller of this api has a already and owned group
                    None => {},
                }

                
                if god_has_a_group || access_level == DEV_ACCESS{
                    
                    // TODO - parse request object to get request data
                    // ...
                    // TODO - fetch and store redis data from its server 
                    // ...

                    todo!()

                } else{

                    resp!{
                        misc::app::Nill, //// the data type
                        misc::app::Nill(&[]), //// the data itself
                        ACCESS_DENIED, //// response message
                        StatusCode::FORBIDDEN, //// status code
                        "application/json" //// the content type 
                    }
                }

                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------

            } else {
                return response.unwrap(); //// response is full and it contains one of these errors: wrong token, not registered or not found user
            }

        },
        None => {//// passport data not found response

            resp!{
                misc::app::Nill, //// the data type
                misc::app::Nill(&[]), //// the data itself
                PASSPORT_DATA_NOT_FOUND, //// response message
                StatusCode::NOT_ACCEPTABLE, //// status code
                "application/json" //// the content type
            }
        }
    
    }

}