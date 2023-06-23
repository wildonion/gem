




use crate::misc;
use crate::schemas;
use crate::middlewares;
use crate::constants::*;
use crate::resp; // this has been imported from the misc inside the app.rs and we can simply import it in here using crate::resp
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; // futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; // it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use log::info;
use mongodb::bson::doc;
use mongodb::Client;
use std::env;







// -------------------------------- home controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn main(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { // the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have the borrowed form of the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   
                            
            let _id = token_data.claims._id;
            let username = token_data.claims.username;



            ////////////////// DB Ops
                    
            let users = db.database("conse").collection::<schemas::auth::UserInfo>("users"); // selecting users collection to fetch all user infos into the UserInfo struct
            match users.find_one(doc!{"username": username.clone(), "_id": _id.unwrap()}, None).await.unwrap(){ // finding user based on username
                Some(user_doc) => { // deserializing BSON into the UserInfo struct
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

                    resp!{
                        schemas::auth::CheckTokenResponse, // the data type
                        user_response, // the data itself
                        ACCESS_GRANTED, // response message
                        StatusCode::OK, // status code
                        "application/json" // the content type 
                    }


                }, 
                None => { // means we didn't find any document related to this username and we have to tell the user do a signup

                    resp!{
                        misc::app::Nill, // the data type
                        misc::app::Nill(&[]), // the data itself
                        DO_SIGNUP, // response message
                        StatusCode::NOT_FOUND, // status code
                        "application/json" // the content type 
                    }
                }
            }

            //////////////////



        },
        Err(e) => {

            resp!{
                misc::app::Nill, // the data type
                misc::app::Nill(&[]), // the data itself
                &e, // response message
                StatusCode::INTERNAL_SERVER_ERROR, // status code
                "application/json" // the content type 
            }
        },
    }
}