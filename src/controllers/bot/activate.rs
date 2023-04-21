




use crate::misc;
use crate::middlewares;
use crate::contexts as ctx;
use crate::passport;
use crate::schemas;
use crate::constants::*;
use chrono::Utc;
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; //// futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; //// it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use mongodb::Client;
use log::info;
use mongodb::options::FindOneAndUpdateOptions;
use mongodb::options::ReturnDocument;
use std::env;






pub async fn start(req: hyper::Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();
    let discord_bot_flag_sender = &req.data::<tokio::sync::mpsc::Sender<bool>>().unwrap().to_owned();
    let db_to_pass_to_macro = db.clone();


    match passport!{ //// this is inside the misc
        req,
        db_to_pass_to_macro
    } {
        Some(passport_data) => {   
            let _id = passport_data.0;
            let username = passport_data.1;
            let access_level = passport_data.2;
            let req = passport_data.3;

            //// -------------------------------------------------------------------------------------
            //// ------------------------------- PASSPORT DATA REGION --------------------------------
            //// -------------------------------------------------------------------------------------
            if access_level == DEV_ACCESS{ //// only dev can start the bot
                    
                //// sending the start bot flag to the downside of 
                //// the channel to start the discrod bot
                // ------------------------
                discord_bot_flag_sender.send(true).await.unwrap(); //// this api call event sets this to true so we once we received the true flag we'll start the bot
                // ------------------------
                
                let response_body = ctx::app::Response::<ctx::app::Nill>{
                    data: Some(ctx::app::Nill(&[])),
                    message: DISCORD_BOT_STARTED,
                    status: 200,
                };
                let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                Ok(
                    res
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                        .unwrap() 
                )
            } else{ //// access denied for this user with none admin and dev access level
                let response_body = ctx::app::Response::<ctx::app::Nill>{
                    data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                    message: ACCESS_DENIED,
                    status: 403,
                };
                let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                Ok(
                    res
                        .status(StatusCode::FORBIDDEN)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                        .unwrap() 
                )
            }
            //// -------------------------------------------------------------------------------------
            //// -------------------------------------------------------------------------------------
            //// -------------------------------------------------------------------------------------
        },
        None => { //// if we're here it might be because of the JWT parsing (wrong token) or not found user error 
            let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: DO_SIGNUP, //// document not found in database and the user must do a signup
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
        }

    }

}