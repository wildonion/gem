





use crate::middlewares;
use crate::misc;
use crate::schemas;
use crate::constants::*;
use crate::resp; // this has been imported from the misc inside the app.rs and we can simply import it in here using crate::resp
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; // futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; // it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use log::info;
use mongodb::Client;
use mongodb::bson::Regex;
use mongodb::bson::{self, oid::ObjectId, doc}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use hyper::http::Uri;
use mongodb::options::FindOptions;
use std::env;















// -------------------------------- get NFT mint addrs controller
// ➝ Return : Hyper Response Body or Hyper Error
// --------------------------------------------------------------------------------------

pub async fn mint_addrs(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{ // get a whitelist infos


    // ==============================================================================
    //                              LOAD NFT MINT ADDRESSES
    // ==============================================================================
    // we can pass a buffer of file data (path buffer) 
    // or tcp stream bytes into the from_reader()
    let file = std::fs::File::open("addrs.json").expect("file should open read only"); // the file must be inside where we run the `cargo run` command or the root dir
    let nfts_value: serde_json::Value = serde_json::from_reader(file).expect("file should be proper JSON");
    let nfts_json_string = serde_json::to_string(&nfts_value).unwrap();
    let nft = serde_json::from_str::<schemas::whitelist::Nft>(&nfts_json_string).unwrap(); // we must map the json string into the Nft struct to fill its vector
    let snapshot_nfts = nft.mint_addrs;
    

    resp!{
        Vec<String>, // the data type
        snapshot_nfts, // the data itself
        FOUND_DOCUMENT, // response message
        StatusCode::FOUND, // status code
        "application/json" // the content type 
    }

                                

}












// -------------------------------- get all whitelists controller
// ➝ Return : Hyper Response Body or Hyper Error
// --------------------------------------------------------------------------------------

pub async fn all_whitelists(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{ // get all whitelist infos


    use routerify::prelude::*;
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();


    ////////////////// DB Ops

    let whitelist = db.database(&db_name).collection::<schemas::whitelist::WhitelistInfo>("whitelist"); // selecting whitelist collection to fetch and deserialize all whitelist infos or documents from BSON into the WhitelistInfo struct
    let mut available_whitelist = Vec::<schemas::whitelist::WhitelistInfo>::new();
    match whitelist.find(None, None).await{
        Ok(mut cursor) => {
            while let Ok(Some(wl)) = cursor.try_next().await{ // calling try_next() method on cursor needs the cursor to be mutable - reading while awaiting on try_next() method doesn't return None
                available_whitelist.push(wl);
            }

            resp!{
                Vec<schemas::whitelist::WhitelistInfo>, // the data type
                available_whitelist, // the data itself
                FETCHED, // response message
                StatusCode::OK, // status code
                "application/json" // the content type 
            }
        },
        Err(e) => {

            resp!{
                misc::app::Nill, // the data type
                misc::app::Nill(&[]), // the data itself
                &e.to_string(), // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                "application/json" // the content type 
            }
        },
    }
    
    //////////////////
                

                

}









// -------------------------------- get whitelist controller
// ➝ Return : Hyper Response Body or Hyper Error
// --------------------------------------------------------------------------------------

pub async fn whitelist(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{ // get a whitelist infos


    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();                    
    let name = format!("{}", req.param("name").unwrap()); // we must create the name param using format!() since this macro will borrow the req object and doesn't move it so we can access the req object later to handle other incoming data 
    


    ////////////////// DB Ops
    
    let filter = doc! {"name": name};
    let whitelist = db.database(&db_name).collection::<schemas::whitelist::WhitelistInfo>("whitelist"); // selecting whitelist collection to fetch and deserialize all whitelist infos or documents from BSON into the WhitelistInfo struct
    match whitelist.find_one(filter, None).await.unwrap(){
        Some(whitelist_doc) => {
            let res = Response::builder(); // creating a new response cause we didn't find any available route
            let response_body = misc::app::Response::<schemas::whitelist::WhitelistInfo>{
                message: FETCHED,
                data: Some(whitelist_doc),
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK) // not found route or method not allowed
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        },
       None => {
            let response_body = misc::app::Response::<misc::app::Nill>{
                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                message: NOT_FOUND_DOCUMENT,
                status: 404,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }
    
    //////////////////
                

                

}










// -------------------------------- get whitelist owner score controller
// ➝ Return : Hyper Response Body or Hyper Error
// --------------------------------------------------------------------------------------

pub async fn whitelist_owner_score(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{ // get a whitelist infos


    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();                    
    let name = format!("{}", req.param("name").unwrap()); // we must create the name param using format!() since this macro will borrow the req object and doesn't move it so we can access the req object later to handle other incoming data 
    let owner = format!("{}", req.param("owner").unwrap()); // we must create the name param using format!() since this macro will borrow the req object and doesn't move it so we can access the req object later to handle other incoming data 



    ////////////////// DB Ops
    
    let filter = doc! {"name": name, "owners.owner": owner};
    let whitelist = db.database(&db_name).collection::<schemas::whitelist::WhitelistInfo>("whitelist"); // selecting whitelist collection to fetch and deserialize all whitelist infos or documents from BSON into the WhitelistInfo struct
    match whitelist.find_one(filter, None).await.unwrap(){
        Some(whitelist_doc) => {
            let res = Response::builder(); // creating a new response cause we didn't find any available route
            let response_body = misc::app::Response::<schemas::whitelist::WhitelistInfo>{
                message: FETCHED,
                data: Some(whitelist_doc),
                status: 200,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::OK) // not found route or method not allowed
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket
                    .unwrap()
            )
        },
       None => {
            let response_body = misc::app::Response::<misc::app::Nill>{
                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                message: NOT_FOUND_DOCUMENT,
                status: 404,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }
    
    //////////////////
                

                

}

