




use crate::misc;
use crate::middlewares;
use crate::contexts as ctx;
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

    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { //// the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have the borrowed form of the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   

            //// in this route we don't need to use the req object 
            //// since we're only activating the bot
            //// ...

            let _id = token_data.claims._id;
            let username = token_data.claims.username;
            let access_level = token_data.claims.access_level;


            let db_to_pass = db.clone();
            if middlewares::auth::user::exists(Some(&db_to_pass), _id, username, access_level).await{ //// finding the user with these info extracted from jwt
                if access_level == DEV_ACCESS{
                    //// sending the start bot flag to the downside of the channel
                    //// to start the discrod bot
                    discord_bot_flag_sender.send(true).await.unwrap(); //// this api call event sets this to true so we once we received the true flag we'll start the bot
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
            } else{ //// user doesn't exist with the extracted info from the JWT
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
        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e, //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap();
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json))
                    .unwrap()
            )
        }
    }




}