


 

use crate::*;
use crate::constants::{NO_FANS_FOUND, STORAGE_IO_ERROR_CODE, INVALID_QUERY_LIMIT, NO_FRIEND_FOUND};
use crate::misc::{Response, Limit};
use crate::schema::users_fans::dsl::*;
use crate::schema::users_fans;
use super::users::User;
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest};



/* 

    diesel migration generate users_fans ---> create users_fans migration sql files
    diesel migration run                 ---> apply sql files to db 
    diesel migration redo                ---> drop tables 

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
    pub username: String,
    pub user_avatar: Option<String>,
    pub screen_cid: String,
    pub requested_at: i64,
    pub is_accepted: bool,
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
pub struct AcceptFriendRequest{
    pub owner_cid: String, 
    pub friend_screen_cid: String, 
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
        'updatefrienddata: for frn_req in &mut decoded_friends_data{

            if frn_req.is_accepted == false && 
                frn_req.screen_cid == friend_screen_cid{
                    
                frn_req.is_accepted = true;
                break 'updatefrienddata;

            }
        }

        Self::update(&owner_screen_cid, UpdateUserFanData{ 
            friends: Some(serde_json::to_value(decoded_friends_data).unwrap()), 
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

        /* sorting invitation requests in desc order */
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

    pub async fn get_all_user_fans_data_for(owner_screen_cid: &str, limit: web::Query<Limit>,
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

        let user_fan_data = users_fans
            .filter(users_fans::user_screen_cid.eq(owner_screen_cid))
            .order(created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .load::<UserFan>(connection);

        let Ok(fans_data) = user_fan_data else{

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
            
            fans_data
                .into_iter()
                .map(|f|{

                    UserFanData{
                        id: f.id,
                        user_screen_cid: f.user_screen_cid,
                        friends: f.friends,
                        invitation_requests: f.invitation_requests,
                        created_at: f.created_at.to_string(),
                        updated_at: f.updated_at.to_string(),
                    }

                })
                .collect::<Vec<UserFanData>>()
        )

    }

    pub async fn remove_user_from_friend(remove_friend_request: RemoveFriend,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<UserFanData, PanelHttpResponse>{
        
            let RemoveFriend { owner_cid, friend_screen_cid, tx_signature, hash_data } 
                = remove_friend_request;
    
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
                if frd.screen_cid == friend_screen_cid{
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


        let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid);
        let get_user = User::find_by_screen_cid(owner_screen_cid, connection).await;
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
                    screen_cid: to_screen_cid.clone(), 
                    requested_at: chrono::Local::now().timestamp(), 
                    is_accepted: false, /* user must accept it later */
                    username: friend_info.username,
                    user_avatar: friend_info.avatar, 
                };
                
                /* 
                    if the user has not already requested regardless of the owner is accepted or not
                    we'll push it to friends, also the owner might have deleted/removed/unfollowed the 
                    user so next time his friend can send a request again 
                */
                if !decoded_friends_data.iter().any(|f| f.screen_cid == to_screen_cid){
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
                    user_screen_cid: owner_screen_cid.to_owned(),
                    friends: {
                        let friend_data = FriendData{ 
                            screen_cid: to_screen_cid, 
                            requested_at: chrono::Local::now().timestamp(), 
                            is_accepted: false,
                            username: friend_info.username,
                            user_avatar: friend_info.avatar, 
                        };

                        Some(serde_json::to_value(vec![friend_data]).unwrap())
                    },
                    invitation_requests: Some(serde_json::to_value::<Vec<InvitationRequestData>>(vec![]).unwrap()),
                };
            
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

        let friends_data = user_fan_data.clone().friends;
        let decoded_friends_data = if friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
        } else{
            vec![]
        };
        
        let request_sender = invitation_request_data.clone().from_screen_cid;
        if decoded_friends_data.iter().any(|f| {
            /*  check that the owner_screen_cid has a friend with the screen_cid of the one who has send the request or not */
            /*  check that the passed in friend has accepted the request or not */
            if f.screen_cid == request_sender
                && f.is_accepted{
                    true
                } else{
                    false
                }
        }){

            
            let user_invitation_request_data = user_fan_data.invitation_requests;
            let mut decoded_invitation_request_data = if user_invitation_request_data.is_some(){
                serde_json::from_value::<Vec<InvitationRequestData>>(user_invitation_request_data.unwrap()).unwrap()
            } else{
                vec![]
            };

            decoded_invitation_request_data.push(invitation_request_data.clone());

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
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<UserFanData, PanelHttpResponse>{

        let AcceptInvitationRequest { owner_cid, from_screen_cid, gal_id, tx_signature, hash_data } 
            = accept_invitation_request;

        let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid.clone());
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


        /* mutating a structure inside a vector of InvitationRequestData structs using &mut pointer */
        'updateinvreq: for inv_req in &mut decoded_invitation_request_data{

            if inv_req.is_accepted == false && 
                inv_req.from_screen_cid == from_screen_cid && 
                inv_req.gallery_id == gal_id{
            
                inv_req.is_accepted = true;
                break 'updateinvreq;

            }
        }

        match Self::update(&owner_screen_cid, UpdateUserFanData{ 
            friends: user_fan_data.friends, 
            invitation_requests: Some(serde_json::to_value(decoded_invitation_request_data).unwrap())
        }, connection).await{

            Ok(updated_user_fan_data) => {

                // update invited_friends with the owner_screen_cid
                let get_gallery_data = UserPrivateGallery::find_by_id(gal_id, connection).await;
                let Ok(gallery) = get_gallery_data else{
                    let resp_error = get_gallery_data.unwrap_err();
                    return Err(resp_error);
                };

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
            Err(resp) => {

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