





use mongodb::options::FindOneAndUpdateOptions;
use mongodb::options::ReturnDocument;
use routerify::prelude::*;
use crate::misc;
use crate::schemas;
use crate::resp; // this has been imported from the misc inside the app.rs and we can simply import it in here using crate::resp
use crate::constants::*;
use crate::misc::otp::{Otp, OtpInput}; // based on orphan rule Otp trait must be imported here to use its methods on an instance of OTPAuth which returns impl Otp
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; // futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; // it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use log::info;
use mongodb::bson::doc;
use mongodb::Client;
use chrono::Utc;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex; // async Mutex will be used inside async methods since the trait Send is not implement for std::sync::Mutex 
use uuid::Uuid;







// -------------------------------- check OTP controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------

pub async fn main(req: Request<Body>) -> MafiaResult<hyper::Response<Body>, hyper::Error>{

     

    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    let whole_body_bytes = hyper::body::to_bytes(req.into_body()).await?; // to read the full body we have to use body::to_bytes or body::aggregate to collect all tcp IO stream of future chunk bytes or chunks which is of type utf8 bytes to concatenate the buffers from a body into a single Bytes asynchronously
    match serde_json::from_reader(whole_body_bytes.reader()){ // read the bytes of the filled buffer with hyper incoming body from the client by calling the reader() method from the Buf trait
        Ok(value) => { // making a serde value from the buffer which is a future IO stream coming from the client
            let data: serde_json::Value = value;
            let json = serde_json::to_string(&data).unwrap(); // converting data into a json string
            match serde_json::from_str::<schemas::auth::CheckOTPRequest>(&json){ // the generic type of from_str() method is CheckOTPRequest struct - mapping (deserializing) the json string into the CheckOTPRequest struct
                Ok(otp_info) => { // we got the phone number of the user
                    


                    
                    let code = otp_info.code;
                    let phone = otp_info.phone;
                    let time = otp_info.time;




                    
                    ////////////////// DB Ops

                    let users = db.clone().database(&db_name).collection::<schemas::auth::RegisterRequest>("users");
                    let otp_info = db.clone().database(&db_name).collection::<schemas::auth::OTPInfo>("otp_info");
                    match otp_info.find_one(doc!{"phone": phone.clone(), "code": code}, None).await.unwrap(){ // NOTE - we've cloned the phone in order to prevent its ownership from moving
                        Some(otp_info_doc) => {

                            if time > otp_info_doc.exp_time{

                                resp!{
                                    misc::app::Nill, // the data type
                                    misc::app::Nill(&[]), // the data itself
                                    EXPIRED_OTP_CODE, // response message
                                    StatusCode::NOT_ACCEPTABLE, // status code
                                    "application/json" // the content type 
                                }

                            } else{ // no need to clone time cause time is of type i64 and it's saved inside the stack

                                let user_info_col = db.clone().database(&db_name).collection::<schemas::auth::UserInfo>("users");
                                match user_info_col.find_one(doc!{"phone": phone.clone()}, None).await.unwrap(){ // we're finding the user based on the incoming phone from the clinet - we've cloned the phone in order to prevent its ownership from moving
                                    Some(user_info) => {
                                        let update_option = FindOneAndUpdateOptions::builder().return_document(Some(ReturnDocument::After)).build();
                                        match otp_info.find_one_and_update(doc!{"_id": otp_info_doc._id}, doc!{"$set": {"updated_at": Some(Utc::now().timestamp())}}, Some(update_option)).await.unwrap(){ // updating the updated_at field for the current otp_info doc based on the current otp_info doc id 
                                            Some(updated_otp_info) => {
                                                let check_otp_response = schemas::auth::CheckOTPResponse{
                                                    user_id: user_info._id, // this is of tyoe mongodb bson ObjectId
                                                    otp_info_id: otp_info_doc._id, // this is of tyoe mongodb bson ObjectId
                                                    code: otp_info_doc.code,
                                                    phone: otp_info_doc.phone,
                                                    last_otp_login_update: updated_otp_info.updated_at, 
                                                };

                                                /* ---------------------------------------- */
                                                /* ----------- constructing jwt ----------- */
                                                /* ---------------------------------------- */
                                                let (now, exp) = misc::jwt::gen_times().await;
                                                let jwt_payload = misc::jwt::Claims{_id: user_info._id, access_level: user_info.access_level, iat: now, exp}; // building jwt if passwords are matched
                                                match misc::jwt::construct(jwt_payload).await{
                                                    Ok(token) => {
                                                        users.update_one(doc!{"username": user_info.clone().username}, doc!{"$set": {"last_login_time": Some(Utc::now().timestamp())}}, None).await.unwrap();
                                                        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec
                                                        let user_response = schemas::auth::LoginResponse{
                                                            _id: user_info._id,
                                                            access_token: token,
                                                            username: user_info.username,
                                                            phone: user_info.phone,
                                                            access_level: user_info.access_level,
                                                            status: user_info.status,
                                                            role_id: user_info.role_id,
                                                            side_id: user_info.side_id,
                                                            created_at: user_info.created_at,
                                                            updated_at: user_info.updated_at,
                                                            last_login_time: Some(now),
                                                            wallet_address: user_info.wallet_address,
                                                            balance: user_info.balance
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
                                                /* ---------------------------------------- */
                                                /* ---------------------------------------- */
                                                /* ---------------------------------------- */

                                                resp!{
                                                    schemas::auth::CheckOTPResponse, // the data type
                                                    check_otp_response, // the data itself
                                                    ACCESS_GRANTED, // response message
                                                    StatusCode::OK, // status code
                                                    "application/json" // the content type 
                                                }
                                            },
                                            None => {

                                                resp!{
                                                    misc::app::Nill, // the data type
                                                    misc::app::Nill(&[]), // the data itself
                                                    DO_SIGNUP, // response message
                                                    StatusCode::NOT_FOUND, // status code
                                                    "application/json" // the content type 
                                                }

                                            },
                                        }
                                    },
                                    /* we'll simply insert a new user with a default access and status */
                                    None => {
                                        
                                        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec 
                                        let user_doc = schemas::auth::RegisterRequest{
                                            username: otp_info_doc.phone.clone(),
                                            phone: otp_info_doc.phone.clone(),
                                            pwd: "".to_string(),
                                            access_level: Some(DEFAULT_USER_ACCESS), // default access is the user access
                                            status: DEFAULT_STATUS, // setting the user (player) status to default which is 0
                                            role_id: None,
                                            side_id: None,
                                            created_at: Some(now),
                                            updated_at: Some(now),
                                            last_login_time: Some(now),
                                            wallet_address: Some("0x0000000000000000000000000000000000000000".to_string()),
                                            balance: Some(0)
                                        };
                                        match users.insert_one(user_doc.clone(), None).await{ // serializing the user doc which is of type RegisterRequest into the BSON to insert into the mongodb
                                            Ok(insert_result) => {
        
                                                let update_option = FindOneAndUpdateOptions::builder().return_document(Some(ReturnDocument::After)).build();
                                                match otp_info.find_one_and_update(doc!{"_id": otp_info_doc._id}, doc!{"$set": {"updated_at": Some(Utc::now().timestamp())}}, Some(update_option)).await.unwrap(){ // updating the updated_at field for the current otp_info doc based on the current otp_info doc id 
                                                    Some(updated_otp_info) => {
                                                        let check_otp_response = schemas::auth::CheckOTPResponse{
                                                            user_id: insert_result.inserted_id.as_object_id(), // this is of tyoe mongodb bson ObjectId
                                                            otp_info_id: otp_info_doc._id, // this is of tyoe mongodb bson ObjectId
                                                            code: otp_info_doc.code,
                                                            phone: otp_info_doc.phone,
                                                            last_otp_login_update: updated_otp_info.updated_at, 
                                                        };
                                                        
                                                        /* ---------------------------------------- */
                                                        /* ----------- constructing jwt ----------- */
                                                        /* ---------------------------------------- */
                                                        let (now, exp) = misc::jwt::gen_times().await;
                                                        let jwt_payload = misc::jwt::Claims{_id: insert_result.inserted_id.as_object_id(), access_level: user_doc.access_level.unwrap(), iat: now, exp}; // building jwt if passwords are matched
                                                        match misc::jwt::construct(jwt_payload).await{
                                                            Ok(token) => {
                                                                users.update_one(doc!{"username": user_doc.clone().username}, doc!{"$set": {"last_login_time": Some(Utc::now().timestamp())}}, None).await.unwrap();
                                                                let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec
                                                                let user_response = schemas::auth::LoginResponse{
                                                                    _id: insert_result.inserted_id.as_object_id(),
                                                                    access_token: token,
                                                                    username: user_doc.username,
                                                                    phone: user_doc.phone,
                                                                    access_level: user_doc.access_level.unwrap(),
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
                                                        /* ---------------------------------------- */
                                                        /* ---------------------------------------- */
                                                        /* ---------------------------------------- */
        
                                                    },
                                                    None => {
        
                                                        resp!{
                                                            misc::app::Nill, // the data type
                                                            misc::app::Nill(&[]), // the data itself
                                                            DO_SIGNUP, // response message
                                                            StatusCode::NOT_FOUND, // status code
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
                                                    StatusCode::NOT_ACCEPTABLE, // status code
                                                    "application/json" // the content type 
                                                }
        
                                            }
                                        }

                                    },
                                        
                                }
                                  
                            }
                        },
                        None => { // means we didn't find any document related to this otp and we have to tell the user do a signup
           
                            resp!{
                                misc::app::Nill, // the data type
                                misc::app::Nill(&[]), // the data itself
                                DO_SIGNUP, // response message
                                StatusCode::NOT_FOUND, // status code
                                "application/json" // the content type 
                            }

                        },
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