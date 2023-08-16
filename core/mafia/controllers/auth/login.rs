




use crate::misc;
use crate::schemas;
use crate::constants::*;
use crate::resp; // this has been imported from the misc inside the app.rs and we can simply import it in here using crate::resp
use chrono::Utc;
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; // futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; // it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use log::info;
use mongodb::bson::doc;
use mongodb::Client;
use std::env;







// -------------------------------- login controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn main(req: Request<Body>) -> MafiaResult<hyper::Response<Body>, hyper::Error>{


     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    let whole_body_bytes = hyper::body::to_bytes(req.into_body()).await?; // to read the full body we have to use body::to_bytes or body::aggregate to collect all tcp IO stream of future chunk bytes or chunks which is of type utf8 bytes to concatenate the buffers from a body into a single Bytes asynchronously
    match serde_json::from_reader(whole_body_bytes.reader()){ // read the bytes of the filled buffer with hyper incoming body from the client by calling the reader() method from the Buf trait
        Ok(value) => { // making a serde value from the buffer which is a future IO stream coming from the client
            let data: serde_json::Value = value;
            let json = serde_json::to_string(&data).unwrap(); // converting data into a json string
            match serde_json::from_str::<schemas::auth::LoginRequest>(&json){ // the generic type of from_str() method is LoginRequest struct - mapping (deserializing) the json string into the LoginRequest struct
                Ok(user_info) => { // we got the username and password inside the login route



                    
                    ////////////////// DB Ops
                    
                    let users = db.database(&db_name).collection::<schemas::auth::UserInfo>("users"); // selecting users collection to fetch all user infos into the UserInfo struct
                    match users.find_one(doc!{"username": user_info.clone().username}, None).await.unwrap(){ // finding user based on username
                        Some(user_doc) => { // deserializing BSON into the UserInfo struct
                            match schemas::auth::LoginRequest::verify_pwd(user_doc.clone().pwd, user_info.clone().pwd).await{
                                Ok(is_correct) => {
                                    if is_correct{
                                        let (now, exp) = misc::jwt::gen_times().await;
                                        let jwt_payload = misc::jwt::Claims{_id: user_doc.clone()._id, access_level: user_doc.access_level, iat: now, exp}; // building jwt if passwords are matched
                                        match misc::jwt::construct(jwt_payload).await{
                                            Ok(token) => {
                                                users.update_one(doc!{"username": user_doc.clone().username}, doc!{"$set": {"last_login_time": Some(Utc::now().timestamp())}}, None).await.unwrap();
                                                let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec
                                                let user_response = schemas::auth::LoginResponse{
                                                    _id: user_doc._id,
                                                    access_token: token,
                                                    username: user_doc.username,
                                                    phone: user_doc.phone,
                                                    access_level: user_doc.access_level,
                                                    status: user_doc.status,
                                                    role_id: user_doc.role_id,
                                                    side_id: user_doc.side_id,
                                                    created_at: user_doc.created_at,
                                                    updated_at: user_doc.updated_at,
                                                    last_login_time: Some(now),
                                                    wallet_address: user_doc.wallet_address,
                                                    balance: user_doc.balance
                                                };
                                            
                                                resp!{
                                                    schemas::auth::LoginResponse, // the data type
                                                    user_response, // the data itself
                                                    ACCESS_GRANTED, // response message
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
                                    } else{ // if we're here means hash and raw are not match together and we have the unsuccessful login

                                        resp!{
                                            misc::app::Nill, // the data type
                                            misc::app::Nill(&[]), // the data itself
                                            WRONG_CREDENTIALS, // response message
                                            StatusCode::NOT_FOUND, // status code
                                            "application/json" // the content type 
                                        }
                                        
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
                        &e.to_string(), // response message
                        StatusCode::NOT_ACCEPTABLE, // status code
                        "application/json" // the content type 
                    }

                },
            }
        },
        Err(e) => {
            
            resp!{
                misc::app::Nill, // the data type
                misc::app::Nill(&[]), // the data itself
                &e.to_string(), // response message
                StatusCode::BAD_REQUEST, // status code
                "application/json" // the content type 
            }
        },
    } 
}
