






use crate::middlewares;
use crate::misc;
use crate::schemas;
use crate::constants::*;
use chrono::Utc;
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; // futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; // it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use mongodb::Client;
use log::info;
use mongodb::bson::{self, oid::ObjectId, doc};
use mongodb::options::FindOneAndUpdateOptions;
use mongodb::options::ReturnDocument; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use std::env;














// -------------------------------- add last move controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------

pub async fn add(req: Request<Body>) -> RendezvousResult<hyper::Response<Body>, hyper::Error>{
 
    
    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => {




            let _id = token_data.claims._id;
            let access_level = token_data.claims.access_level;



            let db_to_pass = db.clone();
            if middlewares::auth::user::exists(Some(&db_to_pass), _id, access_level).await{ // finding the user with these info extracted from jwt
                if access_level == DEV_ACCESS{ // NOTE - only dev can handle this route
                    let whole_body_bytes = hyper::body::to_bytes(req.into_body()).await?; // to read the full body we have to use body::to_bytes or body::aggregate to collect all tcp IO stream of future chunk bytes or chunks which is of type utf8 bytes to concatenate the buffers from a body into a single Bytes asynchronously
                    match serde_json::from_reader(whole_body_bytes.reader()){ // read the bytes of the filled buffer with hyper incoming body from the client by calling the reader() method from the Buf trait
                        Ok(value) => { // making a serde value from the buffer which is a future IO stream coming from the client
                            let data: serde_json::Value = value;
                            let json = serde_json::to_string(&data).unwrap(); // converting data into a json string
                            match serde_json::from_str::<schemas::game::AddLastMoveRequest>(&json){ // the generic type of from_str() method is AddLastMoveRequest struct - mapping (deserializing) the json string into the AddLastMoveRequest struct
                                Ok(last_move_info) => {


                                    let name = last_move_info.clone().name; // cloning to prevent ownership moving
                                    let rate = last_move_info.rate;
                                    let desc = last_move_info.clone().desc; // cloning to prevent ownership moving



                                    ////////////////// DB Ops
                                    
                                    let last_moves = db.clone().database(&db_name).collection::<schemas::game::LastMoveInfo>("last_moves");
                                    match last_moves.find_one(doc!{"name": last_move_info.clone().name}, None).await.unwrap(){
                                        Some(last_mvoe_doc) => { 
                                            let response_body = misc::app::Response::<schemas::game::LastMoveInfo>{ // we have to specify a generic type for data field in Response struct which in our case is LastMoveInfo struct
                                                data: Some(last_mvoe_doc),
                                                message: FOUND_DOCUMENT,
                                                status: 302,
                                            };
                                            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                                            Ok(
                                                res
                                                    .status(StatusCode::FOUND)
                                                    .header(header::CONTENT_TYPE, "application/json")
                                                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                    .unwrap() 
                                            )        
                                        }, 
                                        None => { // no document found with this name thus we must insert a new one into the databse
                                            let now = Utc::now().timestamp_nanos_opt().unwrap() / 1_000_000_000; // nano to sec 
                                            let last_moves = db.clone().database(&db_name).collection::<schemas::game::AddLastMoveRequest>("last_moves"); // using AddLastMoveRequest struct to insert a last move info into last_moves collection 
                                            let last_move_doc = schemas::game::AddLastMoveRequest{
                                                name,
                                                rate,
                                                desc,
                                                is_disabled: Some(false),
                                                created_at: Some(now),
                                                updated_at: Some(now),
                                            };
                                            match last_moves.insert_one(last_move_doc, None).await{
                                                Ok(insert_result) => {
                                                    let response_body = misc::app::Response::<ObjectId>{ // we have to specify a generic type for data field in Response struct which in our case is ObjectId struct
                                                        data: Some(insert_result.inserted_id.as_object_id().unwrap()),
                                                        message: INSERTED,
                                                        status: 201,
                                                    };
                                                    let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                                                    Ok(
                                                        res
                                                            .status(StatusCode::CREATED)
                                                            .header(header::CONTENT_TYPE, "application/json")
                                                            .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                            .unwrap() 
                                                    )
                                                },
                                                Err(e) => {
                                                    let response_body = misc::app::Response::<misc::app::Nill>{
                                                        data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                                                        message: &e.to_string(), // e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                                        status: 406,
                                                    };
                                                    let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                                                    Ok(
                                                        res
                                                            .status(StatusCode::NOT_ACCEPTABLE)
                                                            .header(header::CONTENT_TYPE, "application/json")
                                                            .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                            .unwrap() 
                                                    )
                                                },
                                            }
                                        },                            
                                    }
                                    
                                    //////////////////
                                        
                                    
                                },
                                Err(e) => {
                                    let response_body = misc::app::Response::<misc::app::Nill>{
                                        data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                                        message: &e.to_string(), // e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                        status: 406,
                                    };
                                    let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                                    Ok(
                                        res
                                            .status(StatusCode::NOT_ACCEPTABLE)
                                            .header(header::CONTENT_TYPE, "application/json")
                                            .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                            .unwrap_or(hyper::Response::default()) 
                                    )
                                },
                            }
                        },
                        Err(e) => {
                            let response_body = misc::app::Response::<misc::app::Nill>{
                                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                                message: &e.to_string(), // e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                status: 400,
                            };
                            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                            Ok(
                                res
                                    .status(StatusCode::BAD_REQUEST)
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                    .unwrap() 
                            )
                        },
                    }
                
                
                } else{ // access denied for this user with none admin and dev access level
                    let response_body = misc::app::Response::<misc::app::Nill>{
                        data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                        message: ACCESS_DENIED,
                        status: 403,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::FORBIDDEN)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                }
            } else{ // user doesn't exist :(
                let response_body = misc::app::Response::<misc::app::Nill>{ // we have to specify a generic type for data field in Response struct which in our case is Nill struct
                    data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                    message: DO_SIGNUP, // document not found in database and the user must do a signup
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
            }
        },
        Err(e) => {
            let response_body = misc::app::Response::<misc::app::Nill>{
                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                message: &e, // e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }

}










// -------------------------------- get all last moves controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn all(req: Request<Body>) -> RendezvousResult<hyper::Response<Body>, hyper::Error>{
    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();


    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { // the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have the borrowed form of the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   
                            
            
    
            let _id = token_data.claims._id;
            let access_level = token_data.claims.access_level;
    
            
            let db_to_pass = db.clone();
            if middlewares::auth::user::exists(Some(&db_to_pass), _id, access_level).await{ // finding the user with these info extracted from jwt
                if access_level == ADMIN_ACCESS || access_level == DEV_ACCESS || access_level == DEFAULT_USER_ACCESS{ // NOTE - only dev, admin (God) and player can handle this route

                    
                    ////////////////// DB Ops
                    
                    let filter = doc! { "is_disabled": false }; // filtering all none disabled last_moves
                    let last_moves = db.clone().database(&db_name).collection::<schemas::game::LastMoveInfo>("last_moves"); // selecting last_moves collection to fetch and deserialize all last_moves infos or documents from BSON into the LastMoveInfo struct
                    let mut available_last_moves = Vec::<schemas::game::LastMoveInfo>::new();

                    match last_moves.find(filter, None).await{
                        Ok(mut cursor) => {
                            while let Some(last_move) = cursor.try_next().await.unwrap(){ // calling try_next() method on cursor needs the cursor to be mutable - reading while awaiting on try_next() method doesn't return None
                                available_last_moves.push(last_move);
                            }
                            let res = Response::builder(); // creating a new response cause we didn't find any available route
                            let response_body = misc::app::Response::<Vec<schemas::game::LastMoveInfo>>{
                                message: FETCHED,
                                data: Some(available_last_moves),
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
                        Err(e) => {
                            let response_body = misc::app::Response::<misc::app::Nill>{
                                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                                message: &e.to_string(), // e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                status: 500,
                            };
                            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                            Ok(
                                res
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                    .unwrap() 
                            )
                        },
                    }

                    //////////////////
                
                
                } else{ // access denied for this user with none admin and dev access level
                    let response_body = misc::app::Response::<misc::app::Nill>{
                        data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                        message: ACCESS_DENIED,
                        status: 403,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::FORBIDDEN)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                }
            } else{ // user doesn't exist :(
                let response_body = misc::app::Response::<misc::app::Nill>{ // we have to specify a generic type for data field in Response struct which in our case is Nill struct
                    data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                    message: DO_SIGNUP, // document not found in database and the user must do a signup
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
            }
        },
        Err(e) => {
            let response_body = misc::app::Response::<misc::app::Nill>{
                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                message: &e, // e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }

}








// -------------------------------- disable last move controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn disable(req: Request<Body>) -> RendezvousResult<hyper::Response<Body>, hyper::Error>{

     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { // the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have the borrowed form of the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   
                            
            
    
            let _id = token_data.claims._id;
            let access_level = token_data.claims.access_level;
    
            
            let db_to_pass = db.clone();
            if middlewares::auth::user::exists(Some(&db_to_pass), _id, access_level).await{ // finding the user with these info extracted from jwt
                if access_level == DEV_ACCESS{ // NOTE - only dev can handle this route
                    let whole_body_bytes = hyper::body::to_bytes(req.into_body()).await?; // to read the full body we have to use body::to_bytes or body::aggregate to collect all tcp IO stream of future chunk bytes or chunks which is of type utf8 bytes to concatenate the buffers from a body into a single Bytes asynchronously
                    match serde_json::from_reader(whole_body_bytes.reader()){ // read the bytes of the filled buffer with hyper incoming body from the client by calling the reader() method from the Buf trait
                        Ok(value) => { // making a serde value from the buffer which is a future IO stream coming from the client
                            let data: serde_json::Value = value;
                            let json = serde_json::to_string(&data).unwrap(); // converting data into a json string
                            match serde_json::from_str::<schemas::game::DisableLastMoveRequest>(&json){ // the generic type of from_str() method is DisableLastMoveRequest struct - mapping (deserializing) the json string into the DisableLastMoveRequest struct
                                Ok(dis_info) => {

                                    
                                    ////////////////// DB Ops
                                    
                                    let update_option = FindOneAndUpdateOptions::builder().return_document(Some(ReturnDocument::After)).build();
                                    let last_move_id = ObjectId::parse_str(dis_info._id.as_str()).unwrap(); // generating mongodb object id from the id string
                                    let last_moves = db.clone().database(&db_name).collection::<schemas::game::LastMoveInfo>("last_moves"); // selecting last_moves collection to fetch all last move infos into the LastMoveInfo struct
                                    match last_moves.find_one_and_update(doc!{"_id": last_move_id}, doc!{"$set": {"is_disabled": true, "updated_at": Some(Utc::now().timestamp())}}, Some(update_option)).await.unwrap(){ // finding last move based on last move id
                                        Some(last_mvoe_doc) => { // deserializing BSON into the LastMoveInfo struct
                                            let response_body = misc::app::Response::<schemas::game::LastMoveInfo>{ // we have to specify a generic type for data field in Response struct which in our case is LastMoveInfo struct
                                                data: Some(last_mvoe_doc),
                                                message: UPDATED, // collection found in conse database
                                                status: 200,
                                            };
                                            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                                            Ok(
                                                res
                                                    .status(StatusCode::OK)
                                                    .header(header::CONTENT_TYPE, "application/json")
                                                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                                    .unwrap() 
                                            )
                                        }, 
                                        None => { // means we didn't find any document related to this title and we have to tell the user to create a new last move
                                            let response_body = misc::app::Response::<misc::app::Nill>{ // we have to specify a generic type for data field in Response struct which in our case is Nill struct
                                                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                                                message: NOT_FOUND_DOCUMENT, // document not found in database and the user must do a signup
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


                                },
                                Err(e) => {
                                    let response_body = misc::app::Response::<misc::app::Nill>{
                                        data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                                        message: &e.to_string(), // e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                        status: 406,
                                    };
                                    let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                                    Ok(
                                        res
                                            .status(StatusCode::NOT_ACCEPTABLE)
                                            .header(header::CONTENT_TYPE, "application/json")
                                            .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                            .unwrap_or(hyper::Response::default()) 
                                    )
                                },
                            }
                        },
                        Err(e) => {
                            let response_body = misc::app::Response::<misc::app::Nill>{
                                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                                message: &e.to_string(), // e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                status: 400,
                            };
                            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                            Ok(
                                res
                                    .status(StatusCode::BAD_REQUEST)
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                    .unwrap() 
                            )
                        },
                    }
                
                
                } else{ // access denied for this user with none admin and dev access level
                    let response_body = misc::app::Response::<misc::app::Nill>{
                        data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                        message: ACCESS_DENIED,
                        status: 403,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::FORBIDDEN)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                }
            } else{ // user doesn't exist :(
                let response_body = misc::app::Response::<misc::app::Nill>{ // we have to specify a generic type for data field in Response struct which in our case is Nill struct
                    data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                    message: DO_SIGNUP, // document not found in database and the user must do a signup
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
            }
        },
        Err(e) => {
            let response_body = misc::app::Response::<misc::app::Nill>{
                data: Some(misc::app::Nill(&[])), // data is an empty &[u8] array
                message: &e, // e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                status: 500,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); // converting the response body object into json stringify to send using hyper body
            Ok(
                res
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(response_body_json)) // the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            )
        },
    }

}