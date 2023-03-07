





use crate::middlewares;
use crate::contexts as ctx;
use crate::schemas;
use crate::constants::*;
use crate::utils;
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; //// futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; //// it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use log::info;
use mongodb::Client;
use mongodb::bson::Regex;
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use hyper::http::Uri;
use mongodb::options::FindOptions;
use std::env;














// -------------------------------- get all whitelists controller
// ➝ Return : Hyper Response Body or Hyper Error
// --------------------------------------------------------------------------------------

pub async fn all_whitelists(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{ //// get all user infos


    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();


    ////////////////////////////////// DB Ops

    let whitelist = db.database(&db_name).collection::<schemas::whitelist::WhitelistInfo>("whitelist"); //// selecting whitelist collection to fetch and deserialize all whitelist infos or documents from BSON into the WhitelistInfo struct
    let mut available_whitelist = Vec::<schemas::whitelist::WhitelistInfo>::new();
    match whitelist.find(None, None).await{
        Ok(mut cursor) => {
            while let Some(wl) = cursor.try_next().await.unwrap(){ //// calling try_next() method on cursor needs the cursor to be mutable - reading while awaiting on try_next() method doesn't return None
                available_whitelist.push(wl);
            }
            let res = Response::builder(); //// creating a new response cause we didn't find any available route
            let response_body = ctx::app::Response::<Vec<schemas::whitelist::WhitelistInfo>>{
                message: FETCHED,
                data: Some(available_whitelist),
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK) //// not found route or method not allowed
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }
    
    //////////////////////////////////
                

                

}









// -------------------------------- get whitelist controller
// ➝ Return : Hyper Response Body or Hyper Error
// --------------------------------------------------------------------------------------

pub async fn whitelist(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{ //// get all user infos


    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

                    
    let name = format!("{}", req.param("name").unwrap()); //// we must create the name param using format!() since this macro will borrow the req object and doesn't move it so we can access the req object later to handle other incoming data 
    
    ////////////////////////////////// DB Ops
    
    let filter = doc! {"name": name};
    let whitelist = db.database(&db_name).collection::<schemas::whitelist::WhitelistInfo>("whitelist"); //// selecting whitelist collection to fetch and deserialize all whitelist infos or documents from BSON into the WhitelistInfo struct
    match whitelist.find_one(filter, None).await.unwrap(){
        Some(whitelist_doc) => {
            let res = Response::builder(); //// creating a new response cause we didn't find any available route
            let response_body = ctx::app::Response::<schemas::whitelist::WhitelistInfo>{
                message: FETCHED,
                data: Some(whitelist_doc),
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK) //// not found route or method not allowed
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        },
       None => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: NOT_FOUND_DOCUMENT,
                status: 404,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }
    
    //////////////////////////////////
                

                

}


