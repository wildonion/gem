


 

use actix::Addr;

use crate::*;
use crate::constants::{NO_FANS_FOUND, STORAGE_IO_ERROR_CODE, INVALID_QUERY_LIMIT, NO_FRIEND_FOUND, NO_USER_FANS, USER_SCREEN_CID_NOT_FOUND, INVALID_GALLERY_PRICE};
use crate::misc::{Response, Limit};
use crate::schema::users_fans::dsl::*;
use crate::schema::users_fans;
use super::users::{User, UserWalletInfoResponse, UserData};
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest};



/* 

    diesel migration generate users_fans ---> create users_fans migration sql files
    diesel migration run                 ---> apply sql files to db 
    diesel migration redo                ---> drop tables 

    friends                              ---> those one who have sent requests to user_screen_cid
    invitation_requests                  ---> those one who have sent invitation requests of their own gallery to user_screen_cid
    
    >_ user_screen_cid can accept each request he wants inside friends field
    >_ friends are the ones inside `friends` field who have sent requests to each other and both of them accepted each other's request
    >_ followers are the ones inside `friends` field who their requests are accepted by the user_screen_cid
    >_ followings are the ones inside `friends` field who you've send request to them and they've accepted your request 
    
*/
#[derive(Queryable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users_fans)]
pub struct UserFan{
    pub id: i32,
    pub user_screen_cid: String, /* this is unique */
    pub friends: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub invitation_requests: Option<serde_json::Value>, /* pg key, value based json binary object */
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(PartialEq)]
pub struct FriendData{
    pub username: String, // request sender username
    pub user_avatar: Option<String>, // request sender avatar
    pub screen_cid: String, // request sender screen_cid
    pub requested_at: i64, 
    pub is_accepted: bool, /* owner or user_screen_cid field in UserFan struct must accept or reject the request */
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(PartialEq)]
pub struct UserRelations{
    pub user_info: UserWalletInfoResponse,
    pub followers: UserFanData,
    pub friends: UserFanData,
    pub followings: Vec<UserFanData>
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(PartialEq)]
pub struct SendFriendRequest{
    pub owner_cid: String,
    pub to_screen_cid: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(PartialEq)]
pub struct InvitationRequestData{
    pub username: String,
    pub user_avatar: Option<String>,
    pub from_screen_cid: String,
    pub requested_at: i64,
    pub gallery_id: i32,
    pub is_accepted: bool
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InvitationRequestDataResponse{
    pub to_screen_cid: String,
    pub from_screen_cid: String,
    pub requested_at: i64,
    pub gallery_id: i32,
    pub is_accepted: bool,
    pub username: String,
    pub user_avatar: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[derive(PartialEq)]
pub struct UserFanData{
    pub id: i32,
    pub user_screen_cid: String,
    pub friends: Option<serde_json::Value>,
    pub invitation_requests: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AcceptInvitationRequest{
    pub owner_cid: String, 
    pub from_screen_cid: String, 
    pub gal_id: i32,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EnterPrivateGalleryRequest{
    pub caller_cid: String, 
    pub owner_screen_cid: String, 
    pub gal_id: i32,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AcceptFriendRequest{
    pub owner_cid: String, 
    pub friend_screen_cid: String, 
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RemoveFollower{
    pub owner_cid: String, 
    pub follower_screen_cid: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RemoveFriend{
    pub owner_cid: String, 
    pub friend_screen_cid: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, AsChangeset)]
#[diesel(table_name=users_fans)]
pub struct UpdateUserFanData{
    pub friends: Option<serde_json::Value>,
    pub invitation_requests: Option<serde_json::Value>,
}

#[derive(Insertable)]
#[diesel(table_name=users_fans)]
pub struct InsertNewUserFanRequest{
    pub user_screen_cid: String,
    pub friends: Option<serde_json::Value>,
    pub invitation_requests: Option<serde_json::Value>,
}

impl UserFan{

    pub async fn update_user_fans_data_with_this_user(latest_user_info: UserData,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserFan>, String>{

            match users_fans
            .order(users_fans::created_at.desc())
            .load::<UserFan>(connection)
            {
                Ok(users_fans_) => {

                    let mut updated_users_fans = vec![];
                    for user_fan in users_fans_{

                        let user_friends_data = user_fan.friends;
                        let mut decoded_friends_data = if user_friends_data.is_some(){
                            serde_json::from_value::<Vec<FriendData>>(user_friends_data.unwrap()).unwrap()
                        } else{
                            vec![]
                        }; 

                        let user_invitation_request_data = user_fan.invitation_requests;
                        let mut decoded_invitation_request_data = if user_invitation_request_data.is_some(){
                            serde_json::from_value::<Vec<InvitationRequestData>>(user_invitation_request_data.unwrap()).unwrap()
                        } else{
                            vec![]
                        };
                        
                        /* 
                            since we're taking a mutable pointer to decoded_friends_data
                            so by mutating an element of &mut decoded_friends_data the
                            decoded_friends_data itself will be mutated too
                        */
                        for friend in &mut decoded_friends_data{

                            if friend.screen_cid == latest_user_info.clone().screen_cid.unwrap_or(String::from("")){

                                friend.user_avatar = latest_user_info.clone().avatar;
                                friend.username = latest_user_info.clone().username;
                            }
                        }

                        /* 
                            since we're taking a mutable pointer to decoded_invitation_request_data
                            so by mutating an element of &mut decoded_invitation_request_data the
                            decoded_invitation_request_data itself will be mutated too
                        */
                        for inv in &mut decoded_invitation_request_data{

                            if inv.from_screen_cid == latest_user_info.clone().screen_cid.unwrap_or(String::from("")){

                                inv.user_avatar = latest_user_info.clone().avatar;
                                inv.username = latest_user_info.clone().username;
                            }
                            
                        }

                        // update user_fan 
                        let _ = match diesel::update(users_fans.find((user_fan.id, user_fan.user_screen_cid)))
                            .set((
                                /* store as a new json value in db */
                                friends.eq(
                                    serde_json::to_value(decoded_friends_data).unwrap()
                                ), 
                                invitation_requests.eq(
                                    serde_json::to_value(decoded_invitation_request_data).unwrap()
                                )
                            ))
                            .returning(UserFan::as_returning())
                            .get_result::<UserFan>(connection)
                            {
                                Ok(fetched_uf_data) => {
                                    updated_users_fans.push(fetched_uf_data);
                                },
                                Err(e) => {

                                    let resp_err = &e.to_string();

                                    /* custom error handler */
                                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                    
                                    let error_content = &e.to_string();
                                    let error_content = error_content.as_bytes().to_vec();  
                                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserFan::update_user_fans_data_with_this_user");
                                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                                }
                            };


                    }

                    Ok(
                        updated_users_fans
                    )

                },
                Err(e) => {
    
                    let resp_err = &e.to_string();
    
    
                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserFan::update_user_fans_data_with_this_user");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
    

                    Err(resp_err.to_owned())
                    
                }
            }


    }

    pub async fn accept_friend_request(accept_friend_request: AcceptFriendRequest,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<UserFanData, PanelHttpResponse>{

        let AcceptFriendRequest { owner_cid, friend_screen_cid, tx_signature, hash_data } 
            = accept_friend_request;

        let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid.clone());
        let get_user_fan = Self::get_user_fans_data_for(&owner_screen_cid, connection).await;
        let Ok(user_fan_data) = get_user_fan else{
            let resp_error = get_user_fan.unwrap_err();
            return Err(resp_error);
        };

        let user_friends_data = user_fan_data.friends;
        let mut decoded_friends_data = if user_friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(user_friends_data.unwrap()).unwrap()
        } else{
            vec![]
        };
        

        /* mutating a structure inside a vector of FriendData structs using &mut pointer */
        'updatefrienddatablock: for frn_req in &mut decoded_friends_data{

            if frn_req.is_accepted == false && 
                frn_req.screen_cid == friend_screen_cid{
                    
                frn_req.is_accepted = true;
                break 'updatefrienddatablock;

            }
        }

        Self::update(&owner_screen_cid, UpdateUserFanData{ 
            friends: Some(serde_json::to_value(decoded_friends_data).unwrap()), /* encoding the updated decoded_friends_data back to serde json value */
            invitation_requests: user_fan_data.invitation_requests
        }, connection).await
        

    }

    /*  
        this will be used to fetch the user unaccepted invitation requests
        inside the user fan data from any gallery owner
    */
    pub async fn get_user_unaccpeted_invitation_requests(owner_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
    -> Result<Vec<Option<InvitationRequestData>>, PanelHttpResponse>{
        
        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }
        
        /* 
            owner_screen_cid is taken from the _id inside the JWT so the caller 
            can only call this method for his own
        */
        let get_user_fan_data = Self::get_user_fans_data_for(owner_screen_cid, connection).await;
        let Ok(user_fan_data) = get_user_fan_data else{

            let resp_err = get_user_fan_data.unwrap_err();
            return Err(resp_err);
        };


        let user_invitation_request_data = user_fan_data.invitation_requests;
        let decoded_invitation_request_data = if user_invitation_request_data.is_some(){
            serde_json::from_value::<Vec<InvitationRequestData>>(user_invitation_request_data.unwrap()).unwrap()
        } else{
            vec![]
        };        


        let mut unaccepted_ones = decoded_invitation_request_data
            .into_iter()
            .map(|inv|{

                if inv.is_accepted == false{
                    Some(inv)
                } else{
                    None
                }

            })
            .collect::<Vec<Option<InvitationRequestData>>>();

        unaccepted_ones.retain(|inv| inv.is_some());

        /* 
            sorting invitation requests in desc order, sort_by accepts a closure which returns 
            the Ordering and order the elements based on that 
        */
        unaccepted_ones.sort_by(|inv1, inv2|{
            /* 
                cannot move out of `*inv1` which is behind a shared reference
                move occurs because `*inv1` has type `std::option::Option<InvitationRequestData>`, 
                which does not implement the `Copy` trait and unwrap() takes the 
                ownership of the instance.
                also we must create a longer lifetime for `InvitationRequestData::default()` by 
                putting it inside a type so we can take a reference to it and pass the 
                reference to the `unwrap_or()`, cause &InvitationRequestData::default() will be dropped 
                at the end of the `unwrap_or()` statement while we're borrowing it.
            */
            let inv1_default = InvitationRequestData::default();
            let inv2_default = InvitationRequestData::default();
            let inv1 = inv1.as_ref().unwrap_or(&inv1_default);
            let inv2 = inv2.as_ref().unwrap_or(&inv2_default);

            let inv1_requested_at = inv1.requested_at;
            let inv2_requested_at = inv2.requested_at;

            inv2_requested_at.cmp(&inv1_requested_at)

        });

        /*  
            first we need to slice the current vector convert that type into 
            another vector, the reason behind doing this is becasue we can't
            call to_vec() on the slice directly since the lifetime fo the slice
            will be dropped while is getting used we have to create a longer 
            lifetime then call to_vec() on that type
        */
        let sliced = if unaccepted_ones.len() > to{
            let data = &unaccepted_ones[from..to+1];
            data.to_vec()
        } else{
            let data = &unaccepted_ones[from..unaccepted_ones.len()];
            data.to_vec()
        };

        Ok(
            if sliced.contains(&None){
                vec![]
            } else{
                sliced.to_owned()
            }
        )

    }

    pub async fn get_user_unaccpeted_friend_requests(owner_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
    -> Result<Vec<Option<FriendData>>, PanelHttpResponse>{
        
        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }
        
        let get_user_fan_data = Self::get_user_fans_data_for(owner_screen_cid, connection).await;
        let Ok(user_fan_data) = get_user_fan_data else{

            let resp_err = get_user_fan_data.unwrap_err();
            return Err(resp_err);
        };


        let user_friends_data = user_fan_data.friends;
        let decoded_friends_data = if user_friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(user_friends_data.unwrap()).unwrap()
        } else{
            vec![]
        };        

        let mut unaccepted_ones = decoded_friends_data
            .into_iter()
            .map(|frd|{

                if frd.is_accepted == false{
                    Some(frd)
                } else{
                    None
                }

            })
            .collect::<Vec<Option<FriendData>>>();

        unaccepted_ones.retain(|frd| frd.is_some());

        /* sorting friend requests in desc order */
        unaccepted_ones.sort_by(|frd1, frd2|{
            /* 
                cannot move out of `*frd1` which is behind a shared reference
                move occurs because `*frd1` has type `std::option::Option<FriendData>`, 
                which does not implement the `Copy` trait and unwrap() takes the 
                ownership of the instance.
                also we must create a longer lifetime for `FriendData::default()` by 
                putting it inside a type so we can take a reference to it and pass the 
                reference to the `unwrap_or()`, cause &FriendData::default() will be dropped 
                at the end of the `unwrap_or()` statement while we're borrowing it.
            */
            let frd1_default = FriendData::default();
            let frd2_default = FriendData::default();
            let frd1 = frd1.as_ref().unwrap_or(&frd1_default);
            let frd2 = frd2.as_ref().unwrap_or(&frd2_default);

            let frd1_requested_at = frd1.requested_at;
            let frd2_requested_at = frd2.requested_at;

            frd2_requested_at.cmp(&frd1_requested_at)

        });
        
        /*  
            first we need to slice the current vector convert that type into 
            another vector, the reason behind doing this is becasue we can't
            call to_vec() on the slice directly since the lifetime fo the slice
            will be dropped while is getting used we have to create a longer 
            lifetime then call to_vec() on that type
        */
        let sliced = if unaccepted_ones.len() > to{
            let data = &unaccepted_ones[from..to+1];
            data.to_vec()
        } else{
            let data = &unaccepted_ones[from..unaccepted_ones.len()];
            data.to_vec()
        };

        Ok(
            if sliced.contains(&None){
                vec![]
            } else{
                sliced.to_owned()
            }
        )

    }

    pub async fn get_user_fans_data_for(owner_screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
    -> Result<UserFanData, PanelHttpResponse>{


        let user_fan_data = users_fans
            .filter(users_fans::user_screen_cid.eq(owner_screen_cid))
            .first::<UserFan>(connection);

        let Ok(fan_data) = user_fan_data else{

            let resp = Response{
                data: Some(owner_screen_cid),
                message: NO_FANS_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        Ok(
            UserFanData{
                id: fan_data.id,
                user_screen_cid: fan_data.user_screen_cid,
                friends: fan_data.friends,
                invitation_requests: fan_data.invitation_requests,
                created_at: fan_data.created_at.to_string(),
                updated_at: fan_data.updated_at.to_string(),
            }
        )

    }


    /* -------------------- 
    // both screen cids must have each other in their friends data 
    // and they must have accepted each other's request
    -------------------- */
    pub async fn are_we_friends(first_screen_cid: &str, second_screen_cid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<bool, PanelHttpResponse>{

        /* ----- get all fan data of the first one ------ */
        let user_fan_data = users_fans
            .filter(users_fans::user_screen_cid.eq(first_screen_cid))
            .first::<UserFan>(connection);

        let Ok(first_fan_data) = user_fan_data else{

            let resp = Response{
                data: Some(first_screen_cid),
                message: NO_FANS_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        /* ----- get all fan data of the second one ------ */
        let user_fan_data = users_fans
            .filter(users_fans::user_screen_cid.eq(second_screen_cid))
            .first::<UserFan>(connection);

        let Ok(second_fan_data) = user_fan_data else{

            let resp = Response{
                data: Some(first_screen_cid),
                message: NO_FANS_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        /* both of them must have each other in their friend data and the request be accepted */
        let first_friends_data = first_fan_data.clone().friends;
        let decoded_first_friends_data = if first_friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(first_friends_data.clone().unwrap()).unwrap()
        } else{
            vec![]
        };

        let second_friends_data = second_fan_data.clone().friends;
        let decoded_second_friends_data = if second_friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(second_friends_data.clone().unwrap()).unwrap()
        } else{
            vec![]
        };

        if 
            decoded_first_friends_data
                .into_iter()
                .any(|fd| fd.screen_cid == second_screen_cid && fd.is_accepted == true) 
        
        && 
            decoded_second_friends_data
                .into_iter()
                .any(|fd| fd.screen_cid == first_screen_cid && fd.is_accepted == true)
        
        {
            Ok(true)
        } else{
            Ok(false)
        }


    }
    
    /* -------------------- 
    // get those ones inside the owner friend data who
    // are friend with the owner
    -------------------- */
    pub async fn get_all_my_friends(owner_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
    -> Result<UserFanData, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let user_fan_data = users_fans
            .filter(users_fans::user_screen_cid.eq(owner_screen_cid))
            .first::<UserFan>(connection);

        let Ok(fan_data) = user_fan_data else{

            let resp = Response{
                data: Some(owner_screen_cid),
                message: NO_FANS_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        let friends_data = fan_data.clone().friends;
        let decoded_friends_data = if friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
        } else{
            vec![]
        };

        let mut both_friend_data_arr = vec![];
        for fd in decoded_friends_data{
            
            let check_we_are_friend = Self::are_we_friends(
                &fd.screen_cid, 
                owner_screen_cid, connection).await;
            
            let are_we_friend = check_we_are_friend.unwrap_or(false);
            
            if are_we_friend{

                both_friend_data_arr.push(fd);
            } 
        }

        Ok(
            UserFanData{
                id: fan_data.id,
                user_screen_cid: fan_data.user_screen_cid,
                friends: {

                    if both_friend_data_arr.is_empty(){
                        Some(serde_json::to_value(both_friend_data_arr).unwrap())
                    } else{

                        /* sorting friend requests in desc order */
                        both_friend_data_arr.sort_by(|frd1, frd2|{
    
                            let frd1_requested_at = frd1.requested_at;
                            let frd2_requested_at = frd2.requested_at;
                            frd2_requested_at.cmp(&frd1_requested_at)
    
                        });
                        
                        /*  
                            first we need to slice the current vector convert that type into 
                            another vector, the reason behind doing this is becasue we can't
                            call to_vec() on the slice directly since the lifetime fo the slice
                            will be dropped while is getting used we have to create a longer 
                            lifetime then call to_vec() on that type
                        */
                        let sliced = if both_friend_data_arr.len() > to{
                            let data = &both_friend_data_arr[from..to+1];
                            data.to_vec()
                        } else{
                            let data = &both_friend_data_arr[from..both_friend_data_arr.len()];
                            data.to_vec()
                        };
    
                        Some(serde_json::to_value(sliced).unwrap())
                    }
                },
                invitation_requests: fan_data.invitation_requests,
                created_at: fan_data.created_at.to_string(),
                updated_at: fan_data.updated_at.to_string(),
            }
        )

    }

    /* -------------------- 
    // get those ones inside the owner friend data who
    // the owner has accepted their requests
    -------------------- */
    pub async fn get_all_my_followers(owner_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
    -> Result<UserFanData, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0) as usize;
        let to = limit.to.unwrap_or(10) as usize;

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }

        let user_fan_data = users_fans
            .filter(users_fans::user_screen_cid.eq(owner_screen_cid))
            .first::<UserFan>(connection);

        let Ok(fan_data) = user_fan_data else{

            let resp = Response{
                data: Some(owner_screen_cid),
                message: NO_FANS_FOUND,
                status: 404,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            )

        };

        // get followers
        let friends_data = fan_data.clone().friends;
        let mut decoded_friends_data = if friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
        } else{
            vec![]
        }; 

        let mut owner_followers = decoded_friends_data
            .into_iter()
            .map(|frd| {
                if frd.is_accepted{
                    Some(frd)
                } else{
                    None
                }
            })
            .collect::<Vec<Option<FriendData>>>();
        owner_followers.retain(|frd| frd.is_some());

        Ok(
            UserFanData{
                id: fan_data.id,
                user_screen_cid: fan_data.user_screen_cid,
                friends: if owner_followers.is_empty(){
                        Some(serde_json::to_value(owner_followers).unwrap())
                    } else{

                        /* sorting friend requests in desc order */
                        owner_followers.sort_by(|frd1, frd2|{
                            
                            let frd1_default = FriendData::default();
                            let frd2_default = FriendData::default();
                            let frd1 = frd1.as_ref().unwrap_or(&frd1_default);
                            let frd2 = frd2.as_ref().unwrap_or(&frd2_default);
                            
                            let frd1_requested_at = frd1.requested_at;
                            let frd2_requested_at = frd2.requested_at;
                            frd2_requested_at.cmp(&frd1_requested_at)
    
                        });
                        
                        /*  
                            first we need to slice the current vector convert that type into 
                            another vector, the reason behind doing this is becasue we can't
                            call to_vec() on the slice directly since the lifetime fo the slice
                            will be dropped while is getting used we have to create a longer 
                            lifetime then call to_vec() on that type
                        */
                        let sliced = if owner_followers.len() > to{
                            let data = &owner_followers[from..to+1];
                            data.to_vec()
                        } else{
                            let data = &owner_followers[from..owner_followers.len()];
                            data.to_vec()
                        };
    
                        Some(serde_json::to_value(sliced).unwrap())
                    },
                invitation_requests: fan_data.invitation_requests,
                created_at: fan_data.created_at.to_string(),
                updated_at: fan_data.updated_at.to_string(),
            }
        )   

    }

    /* -------------------- 
    // get those ones inside the users_fans table who
    // have owner in their friend data (or) they have 
    // accepted the owner request
    -------------------- */
    pub async fn get_all_my_followings(who_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<Vec<UserFanData>, PanelHttpResponse>{

            let from = limit.from.unwrap_or(0);
            let to = limit.to.unwrap_or(10);

            if to < from {
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_QUERY_LIMIT,
                    status: 406,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                )
            }

            let all_fans_data = users_fans
                .order(created_at.desc())
                .offset(from)
                .limit((to - from) + 1)    
                .load::<UserFan>(connection);

            let Ok(fans_data) = all_fans_data else{

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: NO_USER_FANS,
                    status: 404,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                )

            };

            let mut followings = vec![];
            for fan_data in fans_data{

                if fan_data.user_screen_cid == who_screen_cid{
                    continue;
                }

                let friends_data = fan_data.clone().friends;
                let mut decoded_friends_data = if friends_data.is_some(){
                    serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
                } else{
                    vec![]
                }; 
                
                for friend in decoded_friends_data{
                    // if friend.screen_cid == who_screen_cid && friend.is_accepted{
                    if friend.screen_cid == who_screen_cid{
                        followings.push({
                            UserFanData{
                                id: fan_data.id,
                                user_screen_cid: fan_data.clone().user_screen_cid,
                                friends: fan_data.clone().friends,
                                invitation_requests: fan_data.clone().invitation_requests,
                                created_at: fan_data.created_at.to_string(),
                                updated_at: fan_data.updated_at.to_string(),
                            }
                        });
                    }
                }

            }

            // can't return UserFanData cause user might have no one send him a request yet
            // so there is no record for who_screen_cid yet thus users_fans would be empty.
            Ok(
                followings
            ) 

        }

    pub async fn get_user_relations(who_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserRelations, PanelHttpResponse>{

        // in-place initialization and returning
        Ok(
            UserRelations{
                user_info: {
                    let get_user_info = User::fetch_wallet_by_username_or_mail_or_scid(who_screen_cid, connection).await;
                    let Ok(user_wallet_info) = get_user_info else{
                        let err_resp = get_user_info.unwrap_err();
                        return Err(err_resp);
                    };
                    user_wallet_info
                },
                followers: {
                    let get_followers = Self::get_all_my_followers(who_screen_cid, limit.clone(), connection).await;
                    let Ok(user_followers) = get_followers else{
                        let err_resp = get_followers.unwrap_err();
                        return Err(err_resp);
                    };
                    user_followers
                },
                friends: {
                    let get_friends = Self::get_all_my_friends(who_screen_cid, limit.clone(), connection).await;
                    let Ok(user_friends) = get_friends else{
                        let err_resp = get_friends.unwrap_err();
                        return Err(err_resp);
                    };
                    user_friends
                },
                followings: {
                    let get_followings = Self::get_all_my_followings(who_screen_cid, limit, connection).await;
                    let Ok(user_followings) = get_followings else{
                        let err_resp = get_followings.unwrap_err();
                        return Err(err_resp);
                    };
                    user_followings
                },
            }
        )

    }

    pub async fn remove_follower(remove_follower_request: RemoveFollower,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<UserFanData, PanelHttpResponse>{
        
            let RemoveFollower { owner_cid, follower_screen_cid, tx_signature, hash_data } 
                = remove_follower_request;
            
            let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid.clone());
            let get_user_fan = Self::get_user_fans_data_for(&owner_screen_cid, connection).await;
            let Ok(user_fan_data) = get_user_fan else{
                let resp_error = get_user_fan.unwrap_err();
                return Err(resp_error);
            };
    
            let user_friends_data = user_fan_data.friends;
            let mut decoded_friends_data = if user_friends_data.is_some(){
                serde_json::from_value::<Vec<FriendData>>(user_friends_data.unwrap()).unwrap()
            } else{
                vec![]
            };
    
            
            if decoded_friends_data.clone().into_iter().any(|frd| {
                if frd.screen_cid == follower_screen_cid && frd.is_accepted == true{
                    let f_idx = decoded_friends_data.iter().position(|f| *f == frd).unwrap();
                    decoded_friends_data.remove(f_idx);
                    true
                } else{
                    false
                }
            }){
            
                Self::update(&owner_screen_cid, UpdateUserFanData{ 
                    friends: Some(serde_json::to_value(decoded_friends_data).unwrap()), 
                    invitation_requests: user_fan_data.invitation_requests
                }, connection).await
                
            } else{

                let resp = Response::<'_, String>{
                    data: Some(follower_screen_cid),
                    message: NO_FRIEND_FOUND,
                    status: 404,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                )

            }


    }

    pub async fn remove_freind(remove_friend_request: RemoveFriend,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<UserFanData, PanelHttpResponse>{
        
            let RemoveFriend { owner_cid, friend_screen_cid, tx_signature, hash_data } 
                = remove_friend_request;
    
            let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid.clone());
            let get_user_fan = Self::get_user_fans_data_for(&friend_screen_cid, connection).await;
            let Ok(user_fan_data) = get_user_fan else{
                let resp_error = get_user_fan.unwrap_err();
                return Err(resp_error);
            };
    
            let user_friends_data = user_fan_data.friends;
            let mut decoded_friends_data = if user_friends_data.is_some(){
                serde_json::from_value::<Vec<FriendData>>(user_friends_data.unwrap()).unwrap()
            } else{
                vec![]
            };
    
            if decoded_friends_data.clone().into_iter().any(|frd| {
                if frd.screen_cid == owner_screen_cid.to_owned() && frd.is_accepted == true{
                    let f_idx = decoded_friends_data.iter().position(|f| *f == frd).unwrap();
                    decoded_friends_data.remove(f_idx);
                    
                    // remove frd from private gallery if his in there
                    // since the owner is not his friend any more
                    let get_all_private_galleries = UserPrivateGallery::get_all_for_without_limit(owner_screen_cid, connection);
                    if get_all_private_galleries.is_ok(){
                        let galleries = get_all_private_galleries.unwrap();
                        for gallery in galleries{
                            let invited_friends = gallery.clone().invited_friends;
                            if invited_friends.is_some(){
                                let mut inv_frds = invited_friends.clone().unwrap();
                                for inv_frd in inv_frds.clone(){
                                    if inv_frd.unwrap_or(String::from("")) == frd.screen_cid{
                                        let frd_idx = inv_frds.iter().position(|invfrd| *invfrd.clone().unwrap() == frd.screen_cid).unwrap();
                                        inv_frds.remove(frd_idx);
                                        // update private gallery
                                        UserPrivateGallery::update_none_async(&gallery.clone().owner_screen_cid, 
                                            UpdateUserPrivateGalleryRequest{
                                                owner_cid: owner_cid.clone(),
                                                collections: gallery.clone().collections,
                                                gal_name: gallery.clone().gal_name,
                                                gal_description: gallery.clone().gal_description,
                                                invited_friends: Some(inv_frds.clone()),
                                                extra: gallery.clone().extra,
                                                tx_signature: String::from(""),
                                                hash_data: String::from(""),
                                            }, gallery.clone().id, connection);
                                        
                                    }
                                }
                            }
                        }
                    }

                    true
                } else{
                    false
                }
            }){
            
                Self::update(&friend_screen_cid, UpdateUserFanData{ 
                    friends: Some(serde_json::to_value(decoded_friends_data).unwrap()), 
                    invitation_requests: user_fan_data.invitation_requests
                }, connection).await
                
            } else{

                let resp = Response::<'_, String>{
                    data: Some(friend_screen_cid),
                    message: NO_FRIEND_FOUND,
                    status: 404,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                )

            }


    }

    pub async fn send_friend_request_to(send_friend_request: SendFriendRequest,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<UserFanData, PanelHttpResponse>{
        
        let SendFriendRequest{ owner_cid, to_screen_cid, tx_signature, hash_data } 
            = send_friend_request;


        let caller_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid); // narni 
        let get_user = User::find_by_screen_cid(caller_screen_cid, connection).await;
        let Ok(user) = get_user else{

            let resp_err = get_user.unwrap_err();
            return Err(resp_err);
        };

        let get_friend = User::find_by_screen_cid(&to_screen_cid.clone(), connection).await;
        let Ok(friend_info) = get_friend else{

            let resp_err = get_friend.unwrap_err();
            return Err(resp_err);
        };

        let user_screen_cid_ = friend_info.screen_cid.unwrap();
        match Self::get_user_fans_data_for(&user_screen_cid_, connection).await{

            /* already inserted just update the friends field */
            Ok(user_fan_data) => {

                let friends_data = user_fan_data.clone().friends;
                let mut decoded_friends_data = if friends_data.is_some(){
                    serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
                } else{
                    vec![]
                };

                let friend_data = FriendData{ 
                    screen_cid: caller_screen_cid.clone(), //
                    requested_at: chrono::Local::now().timestamp(), 
                    is_accepted: false, /* user_screen_cid_ must accept it later */
                    username: user.username, // username of the one who has sent the request
                    user_avatar: user.avatar, // avatar of the one who has sent the request
                };
                
                /* 
                    if the user has not already requested regardless of that owner is accepted or not
                    we'll push it to friends, also the owner might have deleted/removed/unfollowed the 
                    user so next time his friend can send a request again 
                */
                if !decoded_friends_data.iter().any(|f| f.screen_cid == caller_screen_cid.to_owned()){
                    decoded_friends_data.push(friend_data);
                } 

                Self::update(&user_screen_cid_, 
                    UpdateUserFanData{ 
                        friends: Some(serde_json::to_value(decoded_friends_data).unwrap()), 
                        invitation_requests: user_fan_data.invitation_requests 
                    }, connection).await
                
            },
            /* insert new record */
            Err(resp) => {
                
                let new_fan_data = InsertNewUserFanRequest{
                    user_screen_cid: user_screen_cid_.to_owned(),
                    friends: {
                        let friend_data = FriendData{ 
                            screen_cid: caller_screen_cid.clone(), // caller is sending request to user_screen_cid
                            requested_at: chrono::Local::now().timestamp(), 
                            is_accepted: false,
                            username: user.username,
                            user_avatar: user.avatar, 
                        };

                        Some(serde_json::to_value(vec![friend_data]).unwrap())
                    },
                    invitation_requests: Some(serde_json::to_value::<Vec<InvitationRequestData>>(vec![]).unwrap()),
                };
                
                /* return the last record */
                match diesel::insert_into(users_fans)
                    .values(&new_fan_data)
                    .returning(UserFan::as_returning())
                    .get_result::<UserFan>(connection)
                    {
                        Ok(user_fan_data) => {
        
                            Ok(
                                UserFanData{ 
                                    id: user_fan_data.id, 
                                    user_screen_cid: user_fan_data.user_screen_cid, 
                                    friends: user_fan_data.friends, 
                                    invitation_requests: user_fan_data.invitation_requests, 
                                    created_at: user_fan_data.created_at.to_string(), 
                                    updated_at: user_fan_data.updated_at.to_string() 
                                }
                            )
        
                        },
                        Err(e) => {
        
                            let resp_err = &e.to_string();

                            /* custom error handler */
                            use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();  
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserFan::send_friend_request_to");
                            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
        
                            let resp = Response::<&[u8]>{
                                data: Some(&[]),
                                message: resp_err,
                                status: 500,
                                is_error: true
                            };
                            return Err(
                                Ok(HttpResponse::InternalServerError().json(resp))
                            );
        
                        }
                    }

            }
        }

    }

    /* 
        pushing a new invitation request for the passed in user by doing this we're updating the 
        invitation_request_data field for the user so client can get the invitation_request_data
        in an interval and check for new unaccepted ones. use get_user_unaccpeted_invitation_requests 
        method to fetch those ones that their `is_accepted` are false.
    */
    pub async fn push_invitation_request_for(owner_screen_cid: &str, 
        invitation_request_data: InvitationRequestData,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<InvitationRequestDataResponse, PanelHttpResponse>{
        
        let get_user = User::find_by_screen_cid(owner_screen_cid, connection).await;
        let Ok(user) = get_user else{

            let resp_err = get_user.unwrap_err();
            return Err(resp_err);
        };
        
        let get_user_fan_data = Self::get_user_fans_data_for(owner_screen_cid, connection).await;
        let Ok(user_fan_data) = get_user_fan_data else{

            let error_resp = get_user_fan_data.unwrap_err();
            return Err(error_resp);
        };

        let request_sender = invitation_request_data.clone().from_screen_cid;

        let check_we_are_friend = Self::are_we_friends(
            &request_sender, 
            owner_screen_cid, connection).await;
        
        let Ok(are_we_friend) = check_we_are_friend else{
            let err_resp = check_we_are_friend.unwrap_err();
            return Err(err_resp);
        };
        
        if are_we_friend{

            let user_invitation_request_data = user_fan_data.invitation_requests;
            let mut decoded_invitation_request_data = if user_invitation_request_data.is_some(){
                serde_json::from_value::<Vec<InvitationRequestData>>(user_invitation_request_data.unwrap()).unwrap()
            } else{
                vec![]
            };

            if !decoded_invitation_request_data
                .clone()
                .into_iter()
                .any(|invrd| invrd.gallery_id == invitation_request_data.gallery_id){

                    decoded_invitation_request_data.push(invitation_request_data.clone());
                }

            let new_updated_data = Self::update(owner_screen_cid, 
                UpdateUserFanData{ 
                    friends: user_fan_data.friends, 
                    invitation_requests: Some(serde_json::to_value(decoded_invitation_request_data).unwrap()), 
                }, connection).await;

            let Ok(udpated_data) = new_updated_data else{

                let resp_err = new_updated_data.unwrap_err();
                return Err(resp_err);
            };

            Ok(
                InvitationRequestDataResponse{
                    to_screen_cid: owner_screen_cid.to_string(),
                    from_screen_cid: request_sender,
                    requested_at: invitation_request_data.requested_at,
                    gallery_id: invitation_request_data.gallery_id,
                    is_accepted: invitation_request_data.is_accepted,
                    username: invitation_request_data.username,
                    user_avatar: invitation_request_data.user_avatar
                }
            )


        } else{

            let resp_msg = format!("{request_sender:} Is Not A Friend Of {owner_screen_cid:}");
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: &resp_msg,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }


    }
    
    pub async fn accept_invitation_request(accept_invitation_request: AcceptInvitationRequest,
        redis_client: RedisClient, redis_actor: Addr<RedisActor>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<UserFanData, PanelHttpResponse>{

        let AcceptInvitationRequest { owner_cid, from_screen_cid, gal_id, tx_signature, hash_data } 
            = accept_invitation_request;

        let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid.clone());
        let user = User::find_by_screen_cid(&owner_screen_cid, connection).await.unwrap();
        if user.screen_cid.is_none(){
            let resp = Response{
                data: Some(owner_screen_cid),
                message: USER_SCREEN_CID_NOT_FOUND,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::InternalServerError().json(resp))
            );
        }
        
        let get_user_fan = Self::get_user_fans_data_for(&owner_screen_cid, connection).await;
        let Ok(user_fan_data) = get_user_fan else{
            let resp_error = get_user_fan.unwrap_err();
            return Err(resp_error);
        };

        let user_invitation_request_data = user_fan_data.invitation_requests;
        let mut decoded_invitation_request_data = if user_invitation_request_data.is_some(){
            serde_json::from_value::<Vec<InvitationRequestData>>(user_invitation_request_data.unwrap()).unwrap()
        } else{
            vec![]
        };

        // update invited_friends with the owner_screen_cid
        let get_gallery_data = UserPrivateGallery::find_by_id(gal_id, connection).await;
        let Ok(gallery) = get_gallery_data else{
            let resp_error = get_gallery_data.unwrap_err();
            return Err(resp_error);
        };


        /* mutating a structure inside a vector of InvitationRequestData structs using &mut pointer */
        'updateinvreqblock: for inv_req in &mut decoded_invitation_request_data{

            if inv_req.is_accepted == false && 
                inv_req.from_screen_cid == from_screen_cid && 
                inv_req.gallery_id == gal_id{
            
                inv_req.is_accepted = true;
                break 'updateinvreqblock;

            }
        }

        match Self::update(&owner_screen_cid, UpdateUserFanData{ 
            friends: user_fan_data.friends, 
            invitation_requests: Some(serde_json::to_value(decoded_invitation_request_data).unwrap())
        }, connection).await{

            Ok(updated_user_fan_data) => {

                // if the updation process of users_fans data was successful then we simply
                // add the caller to invited_friend of the gallery owner
                let gallery_invited_friends = gallery.invited_friends;
                let mut invited_friends = if gallery_invited_friends.is_some(){
                    gallery_invited_friends.unwrap()
                } else{
                    vec![]
                };

                if !invited_friends.contains(&Some(owner_screen_cid.to_string())){
                    invited_friends.push(Some(owner_screen_cid.to_string()));
                }
                
                /* 
                    update the invited_friends field inside the gallery since the user 
                    is accepted the request and he can see the gallery contents
                */
                match UserPrivateGallery::update(&gallery.owner_screen_cid, 
                    UpdateUserPrivateGalleryRequest{
                        owner_cid,
                        collections: gallery.collections,
                        gal_name: gallery.gal_name,
                        gal_description: gallery.gal_description,
                        invited_friends: Some(invited_friends),
                        extra: gallery.extra,
                        tx_signature,
                        hash_data,
                    }, gal_id, connection).await{

                        Ok(_) => Ok(updated_user_fan_data),
                        Err(resp) => return Err(resp),
                    }


            },
            Err(resp) => return Err(resp),
        }

        
    }

    pub async fn enter_private_gallery_request(enter_private_gallery_request: EnterPrivateGalleryRequest,
        redis_client: RedisClient, redis_actor: Addr<RedisActor>,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<UserFanData, PanelHttpResponse>{

        let EnterPrivateGalleryRequest { caller_cid, owner_screen_cid, gal_id, tx_signature, hash_data } 
            = enter_private_gallery_request;

        // caller
        let caller_screen_cid = &walletreq::evm::get_keccak256_from(caller_cid.clone());
        let get_user_fan = Self::get_user_fans_data_for(&caller_screen_cid, connection).await;
        let Ok(user_fan_data) = get_user_fan else{
            let resp_error = get_user_fan.unwrap_err();
            return Err(resp_error);
        };

        let user_invitation_request_data = user_fan_data.invitation_requests;
        let mut decoded_invitation_request_data = if user_invitation_request_data.is_some(){
            serde_json::from_value::<Vec<InvitationRequestData>>(user_invitation_request_data.unwrap()).unwrap()
        } else{
            vec![]
        };

        // update invited_friends with the caller_screen_cid
        let get_gallery_data = UserPrivateGallery::find_by_id(gal_id, connection).await;
        let Ok(gallery) = get_gallery_data else{
            let resp_error = get_gallery_data.unwrap_err();
            return Err(resp_error);
        };
        

        /* ---- considering gallery entry price ---- */
        let mut g_entry_price = 0;
        if gallery.extra.is_some(){
            let g_extra = gallery.extra.as_ref().unwrap();
            if g_extra.is_array(){
                let g_extra_arr = g_extra.as_array().unwrap();
                for obj in g_extra_arr{
                    g_entry_price = obj["entry_price"].as_i64().unwrap_or(0);
                }
            }
        }

        // gallery must have price for this api perhaps is not of type i64
        if g_entry_price == 0{
            let resp = Response{
                data: Some(caller_screen_cid),
                message: INVALID_GALLERY_PRICE,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::InternalServerError().json(resp))
            );
        }

        let user = User::find_by_screen_cid(&caller_screen_cid, connection).await.unwrap();
        // update balance of the one who accepted the request
        // cause he must pay for the entry price of the gallery
        let new_balance = user.balance.unwrap() - g_entry_price;
        let update_user_balance = User::update_balance(user.id, new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
        if update_user_balance.is_err(){
            let err_resp = update_user_balance.unwrap_err();
            return Err(err_resp);
        }

        let owner = User::find_by_screen_cid(&owner_screen_cid, connection).await.unwrap();
        // update balance of the owner
        let new_owner_balance = owner.balance.unwrap() + g_entry_price;
        let update_owner_balance = User::update_balance(owner.id, new_owner_balance, redis_client.clone(), redis_actor.clone(), connection).await;
        if update_owner_balance.is_err(){
            let err_resp = update_owner_balance.unwrap_err();
            return Err(err_resp);
        }

        if !decoded_invitation_request_data
                .clone()
                .into_iter()
                .any(|invrd| invrd.gallery_id == gallery.id){

                    decoded_invitation_request_data.push(InvitationRequestData{
                        username: owner.username,
                        user_avatar: owner.avatar,
                        from_screen_cid: owner_screen_cid.clone(),
                        requested_at: chrono::Local::now().timestamp(),
                        gallery_id: gallery.id,
                        is_accepted: true,
                    });
                }


        match Self::update(&caller_screen_cid, UpdateUserFanData{ 
            friends: user_fan_data.friends, 
            invitation_requests: Some(serde_json::to_value(decoded_invitation_request_data).unwrap())
        }, connection).await{

            Ok(updated_user_fan_data) => {

                // if the users_fans record was updated successfully then we simply push the 
                // user who has paid for the gllary into the invited_friends field of the gallery
                let gallery_invited_friends = gallery.invited_friends;
                let mut invited_friends = if gallery_invited_friends.is_some(){
                    gallery_invited_friends.unwrap()
                } else{
                    vec![]
                };

                // caching the gallery entrance fees in redis, we'll use this to payback the caller_screen_cid
                // when the owner wants to remove the caller_screen_cid from his gallery 
                let mut conn = redis_client.get_async_connection().await.unwrap();
                let get_galleries_with_entrance_fee: redis::RedisResult<String> = conn.get("galleries_with_entrance_fee").await;
                if get_galleries_with_entrance_fee.is_ok(){
                    let mut galleries_with_entrance_fee = serde_json::from_str::<HashMap<i32, (Vec<String>, i64)>>(&get_galleries_with_entrance_fee.unwrap()).unwrap();
                    // also at the time of entering we have to cache the enterance fee so later on 
                    // we should be able to payback the user with this fee once the owner kicks him out
                    // cause the owner might have updated the gallery entery price
                    let get_friend_scids = galleries_with_entrance_fee.get(&gal_id);
                    if get_friend_scids.is_some(){
                        let mut friend_scids = get_friend_scids.clone().unwrap().0.clone();
                        if !friend_scids.contains(&caller_screen_cid.to_string()){
                            friend_scids.push(caller_screen_cid.to_string());
                        }
                        galleries_with_entrance_fee.insert(gal_id, (friend_scids.to_owned(), g_entry_price));
                    } else{
                        galleries_with_entrance_fee.insert(gal_id, (vec![caller_cid], g_entry_price));
                    }

                    let stringified_ = serde_json::to_string_pretty(&galleries_with_entrance_fee).unwrap();
                    let ـ : RedisResult<String> = conn.set("galleries_with_entrance_fee", stringified_).await;
                }

                // push this caller into invited_friends anyways!
                if !invited_friends.contains(&Some(caller_screen_cid.to_string())){
                    invited_friends.push(Some(caller_screen_cid.to_string()));
                }

                match UserPrivateGallery::update(&gallery.owner_screen_cid, 
                    UpdateUserPrivateGalleryRequest{
                        owner_cid: {
                            let user = User::find_by_screen_cid_none_async(&owner_screen_cid, connection);
                            user.unwrap().cid.unwrap()
                        },
                        collections: gallery.collections,
                        gal_name: gallery.gal_name,
                        gal_description: gallery.gal_description,
                        invited_friends: Some(invited_friends),
                        extra: gallery.extra,
                        tx_signature,
                        hash_data,
                    }, gal_id, connection).await{

                        Ok(_) => Ok(updated_user_fan_data),
                        Err(resp) => {
                            
                            // revert the payment process, pay the gallery price back the user 
                            let new_balance = user.balance.unwrap() + g_entry_price;
                            let update_user_balance = User::update_balance(user.id, new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
                            
                            // charge the owner for the gallery price
                            let new_owner_balance = owner.balance.unwrap() - g_entry_price;
                            let update_owner_balance = User::update_balance(owner.id, new_owner_balance, redis_client.clone(), redis_actor.clone(), connection).await;
                            
                            return Err(resp)
                        },
                    }


            },
            Err(resp) => {

                // revert the payment process, pay the gallery price back the user
                let new_balance = user.balance.unwrap() + g_entry_price;
                let update_user_balance = User::update_balance(user.id, new_balance, redis_client.clone(), redis_actor.clone(), connection).await;
                
                // charge the owner for the gallery price
                let new_owner_balance = owner.balance.unwrap() - g_entry_price;
                let update_owner_balance = User::update_balance(owner.id, new_owner_balance, redis_client.clone(), redis_actor.clone(), connection).await;

                return Err(resp);
            }
        }

        
    }

    pub async fn update(owner_screen_cid: &str, new_user_fan_data: UpdateUserFanData, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
        -> Result<UserFanData, PanelHttpResponse>{


        match diesel::update(users_fans.filter(users_fans::user_screen_cid.eq(owner_screen_cid)))
            .set(&new_user_fan_data)
            .returning(UserFan::as_returning())
            .get_result(connection)
            {
            
                Ok(uf) => {
                    Ok(
                        UserFanData{
                            id: uf.id,
                            user_screen_cid: uf.user_screen_cid,
                            friends: uf.friends,
                            invitation_requests: uf.invitation_requests,
                            created_at: uf.created_at.to_string(),
                            updated_at: uf.updated_at.to_string(),
                        }
                    )

                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserFan::update");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
                }
            
            }
            

    }

}