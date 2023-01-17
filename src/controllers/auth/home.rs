




use crate::contexts as ctx;
use crate::schemas;
use crate::middlewares;
use crate::constants::*;
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; //// futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; //// it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use log::info;
use mongodb::bson::doc;
use mongodb::Client;
use std::env;







// -------------------------------- home controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn main(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { //// the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have borrow the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   
                            
            let _id = token_data.claims._id;
            let username = token_data.claims.username;



            ////////////////////////////////// DB Ops
                    
            let users = db.database("ayoub").collection::<schemas::auth::UserInfo>("users"); //// selecting users collection to fetch all user infos into the UserInfo struct
            match users.find_one(doc!{"username": username.clone(), "_id": _id.unwrap()}, None).await.unwrap(){ //// finding user based on username
                Some(user_doc) => { //// deserializing BSON into the UserInfo struct
                    let user_response = schemas::auth::CheckTokenResponse{
                        _id: user_doc._id,
                        username: user_doc.username,
                        phone: user_doc.phone,
                        access_level: user_doc.access_level,
                        status: user_doc.status,
                        role_id: user_doc.role_id,
                        side_id: user_doc.side_id,
                        created_at: user_doc.created_at,
                        updated_at: user_doc.updated_at,
                        last_login_time: user_doc.last_login_time,
                        wallet_address: user_doc.wallet_address,
                        balance: user_doc.balance
                    };
                    let response_body = ctx::app::Response::<schemas::auth::CheckTokenResponse>{ //// we have to specify a generic type for data field in Response struct which in our case is CheckTokenResponse struct
                        data: Some(user_response), 
                        message: ACCESS_GRANTED,
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
                }, 
                None => { //// means we didn't find any document related to this username and we have to tell the user do a signup
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

            //////////////////////////////////



        },
        Err(e) => {
            let response_body = ctx::app::Response::<ctx::app::Nill>{
                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                message: &e, //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
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
}