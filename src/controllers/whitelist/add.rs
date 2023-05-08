




use crate::middlewares;
use crate::misc;
use crate::passport;
use crate::schemas;
use crate::constants::*;
use crate::resp; //// this has been imported from the misc inside the app.rs and we can simply import it in here using crate::resp
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








// -------------------------------- add user to whitelist controller
// ➝ Return : Hyper Response Body or Hyper Error
// ----------------------------------------------------------------------------------

pub async fn upsert(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{

    use routerify::prelude::*; //// to build the response object
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let db = &req.data::<Client>().unwrap().to_owned();

    ///// ==============================================================================
    ////                              LOAD NFT MINT ADDRESSES
    ///// ==============================================================================
    //// from_reader() accepts a buffer path or tcp stream
    //// then it returns a serde Value which can be converted
    //// into the json string that can be mapped into a structure.
    let file = std::fs::File::open("nfts.json").unwrap(); //// the file must be inside where we run the `cargo run` command or the root dir
    let nfts_value: serde_json::Value = serde_json::from_reader(file).unwrap();
    let nfts_json_string = serde_json::to_string(&nfts_value).unwrap(); //// reader in serde_json::from_reader can be a tokio tcp stream, a file or a buffer that contains the u8 bytes
    let nft = serde_json::from_str::<schemas::whitelist::Nft>(&nfts_json_string).unwrap(); 
    let snapshot_nfts = nft.mint_addrs;




    //// ============ NOTE ============
    //// frontend must have the following
    //// hash in a config file since this
    //// hash will be decoded that must be
    //// the one inside the .env file.
    // let api_key_hardcoded = "$argon2i$v=19$m=4096,t=3,p=1$Y29uc2UtaW5zZWN1cmUtOTgwbzM3XiEzZnUpa3pibzV6KGtybTJzXl5ibzFuKi1udnkoNis4MiklNjB5cGRtLXU$xyuPmb2pZQ4P2atgLPwc3ocE5VrEamWBkOxE9SBrdrE";
    
    ///// ==============================================================================
    ////                                API KEY VALIDATION
    ///// ==============================================================================
    let Some(header_value_api_key) = req.headers().get("API_KEY") else{

        resp!{
            misc::app::Nill, //// the data type
            misc::app::Nill(&[]), //// the data itself
            HTTP_HEADER_ERR, //// response message
            StatusCode::INTERNAL_SERVER_ERROR, //// status code
            "application/json" //// the content type 
        }
    };

    let Ok(api_key) = header_value_api_key.to_str() else{ //// hased api key from the client

        resp!{
            misc::app::Nill, //// the data type
            misc::app::Nill(&[]), //// the data itself
            NO_API_KEY, //// response message
            StatusCode::FORBIDDEN, //// status code
            "application/json" //// the content type 
        }
    }; 


    let whitelist_secret_key = env::var("WHITELIST_SECRET_KEY").expect("⚠️ no whitelist secret key variable set");
    let whitelist_secret_key_bytes = whitelist_secret_key.as_bytes();
    let dev = match argon2::verify_encoded(api_key, whitelist_secret_key_bytes){
        Ok(is_dev) => {
            is_dev
        }, 
        Err(e) => {
            resp!{
                misc::app::Nill, //// the data type
                misc::app::Nill(&[]), //// the data itself
                &e.to_string(), //// response message
                StatusCode::INTERNAL_SERVER_ERROR, //// status code
                "application/json" //// the content type 
            }
        }
    };

    if !dev{

        resp!{
            misc::app::Nill, //// the data type
            misc::app::Nill(&[]), //// the data itself
            ACCESS_DENIED, //// response message
            StatusCode::FORBIDDEN, //// status code
            "application/json" //// the content type 
        }
    }
    ///// ==============================================================================

    /*
        @params: 
            - @request       → hyper request object; since this struct doesn't implement Clone trait
            - @storage       → instance inside the request object
            - @access levels → vector of access levels
    */
    match passport!{
        req,
        db.clone(),
        vec![DEV_ACCESS, DEFAULT_USER_ACCESS, ADMIN_ACCESS] //// vector of access levels
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

                let whole_body_bytes = hyper::body::to_bytes(req.into_body()).await?; //// to read the full body we have to use body::to_bytes or body::aggregate to collect all tcp IO stream of future chunk bytes or chunks which is of type utf8 bytes to concatenate the buffers from a body into a single Bytes asynchronously
                match serde_json::from_reader(whole_body_bytes.reader()){ //// read the bytes of the filled buffer with hyper incoming body from the client by calling the reader() method from the Buf trait
                    Ok(value) => { //// making a serde value from the buffer which is a future IO stream coming from the client
                        let data: serde_json::Value = value;
                        let json = serde_json::to_string(&data).unwrap(); //// converting data into a json string
                        match serde_json::from_str::<schemas::whitelist::InsertWhitelistRequest>(&json){ //// the generic type of from_str() method is InsertWhitelistRequest struct - mapping (deserializing) the json string into the InsertWhitelistRequest struct
                            Ok(wl_info) => {


                                let owner = wl_info.owner.clone(); //// cloning to prevent ownership moving
                                let mint_addrs = wl_info.mint_addrs.clone(); //// cloning to prevent ownership moving - the pda calculated from the nft burn tx hash address and the nft owner after burning
                                let name = wl_info.name.clone(); //// cloning to prevent ownership moving

                                // ================= NFT VERIFICATION ===================
                                //// one of the NFT mint addr must be inside the snapshot
                                //// otherwise the NFT is not verified and is not owned
                                //// by the owner
                                // ======================================================
                                let vefified_nft_mint_addresses = mint_addrs.iter().all(|mint_addr| snapshot_nfts.contains(mint_addr));
                                if !vefified_nft_mint_addresses{

                                    resp!{
                                        misc::app::Nill, //// the data type
                                        misc::app::Nill(&[]), //// the data itself
                                        NOT_VERIFIED_NFT, //// response message
                                        StatusCode::NOT_ACCEPTABLE, //// status code
                                        "application/json" //// the content type 
                                    }
                                }



                                ////////////////////////////////// DB Ops

                                let update_option = FindOneAndUpdateOptions::builder().return_document(Some(ReturnDocument::After)).build();
                                let whitelist = db.clone().database(&db_name).collection::<schemas::whitelist::WhitelistInfo>("whitelist");
                                ///// ==============================================================================
                                ////                         UNIQUE OWNER MINT ADDRESS VALIDATION
                                ///// ==============================================================================
                                match whitelist.find_one(doc!{"name": name.clone(), "owners.mint_addrs": mint_addrs.clone()}, None).await.unwrap(){
                                    Some(wl_doc) => {

                                        resp!{
                                            schemas::whitelist::WhitelistInfo, //// the data type
                                            wl_doc, //// the data itself
                                            ALREADY_BURNED, //// response message
                                            StatusCode::FOUND, //// status code
                                            "application/json" //// the content type 
                                        }
                                    }
                                    None => {}
                                }
                                ///// ==============================================================================
                                match whitelist.find_one(doc!{"name": name.clone()}, None).await.unwrap(){
                                    Some(mut wl_doc_name) => { //// we must declare the wl_doc as mutable since we want to mutate it later
                                        match whitelist.find_one(doc!{"owners.owner": owner.clone()}, None).await.unwrap(){
                                            Some(mut wl_doc_owners) => {
                                                let is_owner_exists = wl_doc_owners.owners.clone().into_iter().position(|od| od.owner == owner.clone());
                                                let owner_index = is_owner_exists.unwrap(); //// we're sure that we have an owner definitely since the find_one() query has executed correctly if we're here :)
                                                //// we found the passed in owner inside the whitelist
                                                //// then we have to update the list with the passed in pda
                                                if let Some(mint_addrs) = wl_doc_owners.add_mint_addrs(mint_addrs.clone(), owner_index).await{
                                                    //// means we have an updated pda 
                                                    //// then we need to update the collection
                                                    wl_doc_owners.owners[owner_index].mint_addrs = mint_addrs;
                                                    let serialized_owners = bson::to_bson(&wl_doc_owners.owners).unwrap(); //// we have to serialize the owners to BSON Document object in order to update the owners field inside the collection
                                                    match whitelist.find_one_and_update(doc!{"name": name}, doc!{"$set": {"owners": serialized_owners, "updated_at": Some(Utc::now().timestamp())}}, Some(update_option)).await.unwrap(){
                                                        Some(wl_info) => { //// deserializing BSON into the EventInfo struct
                                                            resp!{
                                                                schemas::whitelist::WhitelistInfo, //// the data type
                                                                wl_info, //// the data itself
                                                                UPDATED, //// response message
                                                                StatusCode::OK, //// status code
                                                                "application/json" //// the content type 
                                                            }
                                                        }, 
                                                        None => { //// means we didn't find any document related to this title and we have to tell the user to create a new event
                                                            resp!{
                                                                misc::app::Nill, //// the data type
                                                                misc::app::Nill(&[]), //// the data itself
                                                                NOT_FOUND_DOCUMENT, //// response message
                                                                StatusCode::NOT_FOUND, //// status code
                                                                "application/json" //// the content type 
                                                            }
                                                        },
                                                    } 
                                                } else{
                                                    resp!{
                                                        schemas::whitelist::WhitelistInfo, //// the data type
                                                        wl_doc_owners, //// the data itself
                                                        ALREADY_BURNED, //// response message
                                                        StatusCode::FOUND, //// status code
                                                        "application/json" //// the content type 
                                                    }
                                                } 
                                            },
                                            None => { //// insert new owners field for the passed in whitelist name 
                                                let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec 
                                                let mut owners = wl_doc_name.owners;
                                                let new_owner_data = schemas::whitelist::OwnerData{
                                                    mint_addrs: mint_addrs.clone(),
                                                    owner: owner.clone(),
                                                    requested_at: Some(now)
                                                };
                                                owners.push(new_owner_data); //// update the owners field 
                                                let serialized_updated_owners = bson::to_bson(&owners).unwrap();
                                                match whitelist.find_one_and_update(doc!{"name": name.clone()}, doc!{"$set": {"owners": serialized_updated_owners, "updated_at": Some(Utc::now().timestamp())}}, Some(update_option)).await.unwrap(){
                                                    Some(wl_info) => { //// deserializing BSON into the EventInfo struct
                                                        resp!{
                                                            schemas::whitelist::WhitelistInfo, //// the data type
                                                            wl_info, //// the data itself
                                                            UPDATED, //// response message
                                                            StatusCode::OK, //// status code
                                                            "application/json" //// the content type 
                                                        }
                                                    }, 
                                                    None => { //// means we didn't find any document related to this title and we have to tell the user to create a new event
                                                        resp!{
                                                            misc::app::Nill, //// the data type
                                                            misc::app::Nill(&[]), //// the data itself
                                                            NOT_FOUND_DOCUMENT, //// response message
                                                            StatusCode::NOT_FOUND, //// status code
                                                            "application/json" //// the content type 
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                    }, 
                                    None => { //// no document found with this name thus we must insert a new whitelist into the databse with the passed in owner as default and first owner
                                        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec 
                                        let whitelist = db.clone().database(&db_name).collection::<schemas::whitelist::AddWhitelistInfo>("whitelist"); //// using AddWhitelistInfo struct to insert a whitelist info into whitelist collection 
                                        let whitelist_doc = schemas::whitelist::AddWhitelistInfo{
                                            name,
                                            owners: vec![schemas::whitelist::OwnerData{
                                                mint_addrs, //// vector of the passed in mint_addrs from the client
                                                owner, //// owner of the burned nft
                                                requested_at: Some(now)
                                            }],
                                            created_at: Some(now),
                                            updated_at: Some(now),
                                        };
                                        match whitelist.insert_one(whitelist_doc, None).await{
                                            Ok(insert_result) => {

                                                resp!{
                                                    ObjectId, //// the data type
                                                    insert_result.inserted_id.as_object_id().unwrap(), //// the data itself
                                                    INSERTED, //// response message
                                                    StatusCode::CREATED, //// status code
                                                    "application/json" //// the content type 
                                                }
                                            },
                                            Err(e) => {

                                                resp!{
                                                    misc::app::Nill, //// the data type
                                                    misc::app::Nill(&[]), //// the data itself
                                                    &e.to_string(), //// response message
                                                    StatusCode::NOT_ACCEPTABLE, //// status code
                                                    "application/json" //// the content type 
                                                }
                                            },
                                        }
                                    },                            
                                }
                                
                                //////////////////////////////////
                            },
                            Err(e) => {

                                resp!{
                                    misc::app::Nill, //// the data type
                                    misc::app::Nill(&[]), //// the data itself
                                    &e.to_string(), //// response message
                                    StatusCode::NOT_ACCEPTABLE, //// status code
                                    "application/json" //// the content type 
                                }
                            },
                        }
                    },
                    Err(e) => {

                        resp!{
                            misc::app::Nill, //// the data type
                            misc::app::Nill(&[]), //// the data itself
                            &e.to_string(), //// response message
                            StatusCode::BAD_REQUEST, //// status code
                            "application/json" //// the content type 
                        }
                    },
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
                misc::app::Nill, //// the data type
                misc::app::Nill(&[]), //// the data itself
                PASSPORT_DATA_NOT_FOUND, //// response message
                StatusCode::NOT_ACCEPTABLE, //// status code
                "application/json" //// the content type
            }
        }
    }

}