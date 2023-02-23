







use mongodb::options::FindOneAndUpdateOptions;
use mongodb::options::ReturnDocument;
use routerify::prelude::*;
use crate::middlewares;
use crate::utils;
use crate::schemas;
use crate::contexts as ctx;
use crate::constants::*;
use chrono::Utc;
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; //// futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; //// it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response, Request};
use log::info;
use mongodb::Client;
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use std::env;

























// -------------------------------- mock reservation controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn mock_reservation(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{

    
     

    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    match middlewares::auth::pass(req).await{
        Ok((token_data, req)) => { //// the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have borrow the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function   
                            
            

            let _id = token_data.claims._id;
            let username = token_data.claims.username;
            let access_level = token_data.claims.access_level;
    
            
            
            let db_to_pass = db.clone();
            if middlewares::auth::user::exists(Some(&db_to_pass), _id, username.clone(), access_level).await{ //// finding the user with these info extracted from jwt
                if access_level == DEV_ACCESS || access_level == DEFAULT_USER_ACCESS || access_level == ADMIN_ACCESS{ // NOTE - only dev, God and player can handle this route since God can reserve another God's event
                    let whole_body_bytes = hyper::body::to_bytes(req.into_body()).await?; //// to read the full body we have to use body::to_bytes or body::aggregate to collect all tcp IO stream of future chunk bytes or chunks which is of type utf8 bytes to concatenate the buffers from a body into a single Bytes asynchronously
                    match serde_json::from_reader(whole_body_bytes.reader()){ //// read the bytes of the filled buffer with hyper incoming body from the client by calling the reader() method from the Buf trait
                        Ok(value) => { //// making a serde value from the buffer which is a future IO stream coming from the client
                            let data: serde_json::Value = value;
                            let json = serde_json::to_string(&data).unwrap(); //// converting data into a json string
                            match serde_json::from_str::<schemas::event::MockReservationRequest>(&json){ //// the generic type of from_str() method is MockReservationRequest struct - mapping (deserializing) the json string into the MockReservationRequest struct
                                Ok(mock_reservation_info) => { //// we got the username and password inside the login route

                                    ////////////////////////////////// DB ops

                                    let update_option = FindOneAndUpdateOptions::builder().return_document(Some(ReturnDocument::After)).build();
                                    let event_id = ObjectId::parse_str(mock_reservation_info.event_id.as_str()).unwrap(); //// generating mongodb object id from the id string - mock_reservation_info.event_id is the mongodb object id of the event that the caller of this method is trying to reserve it
                                    let events = db.database(&db_name).collection::<schemas::event::EventInfo>("events"); //// selecting events collection to fetch and deserialize all event infos or documents from BSON into the EventInfo struct which contains the whole fields
                                    match events.find_one(doc! { "_id": event_id }, None).await.unwrap(){
                                        Some(event_doc) => {
                                            let init_player_info = schemas::game::ReservePlayerInfoResponseWithRoleName{
                                                _id: _id.unwrap(),
                                                username,
                                                status: DEFAULT_STATUS,
                                                role_name: None,
                                                role_id: None,
                                                side_id: None,
                                            };
                                            let updated_players = event_doc.add_player(init_player_info).await; //// add new player info into the existing players vector of the passed in event_id
                                            let serialized_updated_players = bson::to_bson(&updated_players).unwrap(); //// we have to serialize the updated_players to BSON Document object in order to update the players field inside the collection
                                            let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec 
                                            match events.find_one_and_update(doc!{"_id": event_id}, doc!{"$set": {"players": serialized_updated_players, "updated_at": Some(now)}}, Some(update_option)).await.unwrap(){
                                                Some(event_doc) => {
                                                    let event_doc = schemas::event::ReserveEventResponse{
                                                        _id: event_doc._id,
                                                        title: event_doc.title,
                                                        content: event_doc.content,
                                                        deck_id: event_doc.deck_id,
                                                        phases: event_doc.phases,
                                                        max_players: event_doc.max_players,
                                                        players: event_doc.players,
                                                        is_expired: event_doc.is_expired,
                                                        is_locked: event_doc.is_locked,
                                                        expire_at: event_doc.expire_at,
                                                        created_at: event_doc.created_at,
                                                        updated_at: event_doc.updated_at,
                                                    };
                                                    let response_body = ctx::app::Response::<schemas::event::ReserveEventResponse>{ //// we have to specify a generic type for data field in Response struct which in our case is ReserveEventResponse struct
                                                        data: Some(event_doc),
                                                        message: UPDATED,
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
                                                None => {
                                                    let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                                                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                                        message: NOT_FOUND_DOCUMENT, //// document not found in database and the user must do a signup
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
                                        },
                                        None => { //// means we didn't find any document related to this user_id and we have to tell the user do a signup
                                            let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                                                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                                message: NOT_FOUND_DOCUMENT, //// document not found in database and the user must do a signup
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

                                },
                                Err(e) => {
                                    let response_body = ctx::app::Response::<ctx::app::Nill>{
                                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                        message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                        status: 406,
                                    };
                                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                                    Ok(
                                        res
                                            .status(StatusCode::NOT_ACCEPTABLE)
                                            .header(header::CONTENT_TYPE, "application/json")
                                            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                            .unwrap_or(hyper::Response::default()) 
                                    )
                                },
                            }
                        },
                        Err(e) => {
                            let response_body = ctx::app::Response::<ctx::app::Nill>{
                                data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                                message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                                status: 400,
                            };
                            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                            Ok(
                                res
                                    .status(StatusCode::BAD_REQUEST)
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                    .unwrap() 
                            )
                        },
                    }
                } else{ //// access denied for this user with none admin and dev access level
                    let response_body = ctx::app::Response::<ctx::app::Nill>{
                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
                        message: ACCESS_DENIED,
                        status: 403,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                    Ok(
                        res
                            .status(StatusCode::BAD_REQUEST)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                            .unwrap() 
                    )
                }
            } else{ //// user doesn't exist :(
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














// -------------------------------- process payment controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------
pub async fn process_payment_request(req: Request<Body>) -> GenericResult<hyper::Response<Body>, hyper::Error>{

    
     

    use routerify::prelude::*;
    let res = Response::builder();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();


    
    // TODO - 
    // get all (un)successful payments for an event with admin or God access
    // get all (un)successful payments for a user with user access



    let response_body = ctx::app::Response::<ctx::app::Nill>{
        message: NOT_IMPLEMENTED,
        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
        status: 501,
    };
    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
    Ok(
        res
            .status(StatusCode::NOT_IMPLEMENTED) //// not found route or method not allowed
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket
            .unwrap()
    )


}














