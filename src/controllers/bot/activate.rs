




use crate::misc;
use crate::middlewares;
use crate::contexts as ctx;
use crate::passport; //// this has been imported from the misc inside the app.rs and we can simply import it in here using crate::passport
use crate::resp; //// this has been imported from the misc inside the app.rs and we can simply import it in here using crate::resp
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
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();
    let discord_bot_flag_sender = &req.data::<tokio::sync::mpsc::Sender<bool>>().unwrap().to_owned();


    match passport!{
        req,
        vec![DEV_ACCESS] //// vector of access levels
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

                //// sending the start bot flag to the downside of 
                //// the channel to start the discrod bot
                discord_bot_flag_sender.send(true).await.unwrap(); //// this api call event sets this to true so we once we received the true flag we'll start the bot
                            
                resp!{
                    ctx::app::Nill, //// the data type
                    ctx::app::Nill(&[]), //// the data itself
                    DISCORD_BOT_STARTED, //// response message
                    StatusCode::OK, //// status code
                    "application/json" //// the content type 
                }

                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------
            } else {
                return response.unwrap(); //// response is full and it contains one of these errors: wrong token, not registered or not found user
            }
        },
        None => { //// passport data not found response

            resp!{
                ctx::app::Nill, //// the data type
                ctx::app::Nill(&[]), //// the data itself
                PASSPORT_DATA_NOT_FOUND, //// response message
                StatusCode::NOT_ACCEPTABLE, //// status code
                "application/json" //// the content type
            }
        },
    }
}