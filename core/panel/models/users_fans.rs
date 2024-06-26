


 

use actix::Addr;
use chrono::NaiveDateTime;

use crate::*;
use crate::constants::{NO_FANS_FOUND, STORAGE_IO_ERROR_CODE, INVALID_QUERY_LIMIT, NO_FRIEND_FOUND, NO_USER_FANS, USER_SCREEN_CID_NOT_FOUND, INVALID_GALLERY_PRICE};
use crate::events::publishers::action::{SingleUserNotif, NotifData, ActionType};
use crate::helpers::misc::{Response, Limit};
use crate::schema::users_fans::dsl::*;
use crate::schema::users_fans;
use super::galleries_invitation_requests::{NewPrivateGalleryInvitationRequest, PrivateGalleryInvitationRequest};
use super::users::{User, UserWalletInfoResponse, UserData};
use super::users_friends::{NewFriendRequest, UserFriend};
use super::users_galleries::{UserPrivateGallery, UpdateUserPrivateGalleryRequest};



/* 

    diesel migration generate users_fans ---> create users_fans migration sql files
    diesel migration run                 ---> apply sql files to db 
    diesel migration redo                ---> drop tables 

    friends                              ---> those one who have sent requests to user_screen_cid
    invitation_requests                  ---> those one who have sent invitation requests of their own gallery to user_screen_cid
    
    >_ user_screen_cid can accept each request he wants inside the friends field
    >_ friends are the ones inside `friends` field who have sent requests to each other and both of them accepted each other's request
    >_ followers are the ones inside `friends` field who their requests are accepted by the user_screen_cid but they're not friend with each other
    >_ followings are the ones inside `friends` field who you've send request to them and they're not friend with each other
    
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
    pub followers: UserFanData, // followers are the one who are inside friends data field with follower condition 
    pub friends: UserFanData, // followers are the one who are inside friends data field with friend condition
    pub followings: Vec<UserFanDataWithWalletInfo> // followings are the one who are inside the table with the following conditions
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
#[derive(PartialEq)]
pub struct UserFanDataWithWalletInfo{
    pub id: i32,
    pub user_wallet_info: UserWalletInfoResponse,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RemoveFollowing{
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

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct FriendOwnerCount{
    pub owner_wallet_info: UserWalletInfoResponse,
    pub friends_count: usize,
}

impl UserFan{

    pub fn construct_friends_data(&self, connection: &mut DbPoolConnection) -> Option<serde_json::Value>{

        let user = User::find_by_screen_cid_none_async(&self.user_screen_cid, connection).unwrap();
        let get_friend_requests_data = UserFriend::get_all_for_user(user.id, connection);
        let friend_requests_data = if get_friend_requests_data.is_ok(){
            get_friend_requests_data.unwrap()
        } else{
            vec![]
        };

        let mut friends_data = vec![];
        for frd_req in friend_requests_data{
            let friend_info = User::find_by_id_none_async(frd_req.friend_id, connection).unwrap();
            friends_data.push(
                FriendData{
                    username: friend_info.clone().username,
                    user_avatar: friend_info.clone().avatar,
                    screen_cid: friend_info.clone().screen_cid.unwrap(),
                    requested_at: frd_req.requested_at,
                    is_accepted: frd_req.is_accepted,
                }
            )
        }

        Some(
            serde_json::to_value(&friends_data).unwrap()
        )

    }

    pub async fn get_owners_with_lots_of_followers(owners: Vec<UserData>, limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) 
        -> Result<Vec<FriendOwnerCount>, PanelHttpResponse>{

        let mut friends_owner_map = vec![];
        for owner in owners{

            if owner.screen_cid.is_none() || owner.screen_cid.clone().unwrap().is_empty(){
                continue;
            }
            
            let owner_screen_cid_ = owner.screen_cid.unwrap();
            let get_all_owner_friends = UserFan::get_all_my_followers(&owner_screen_cid_, limit.clone(), connection).await;
            let all_owner_friends = if get_all_owner_friends.is_ok(){
                get_all_owner_friends.unwrap()
            } else{
                UserFanData::default()
            };

            let user_friends_data = all_owner_friends.construct_friends_data(connection);
            let mut decoded_friends_data = if user_friends_data.is_some(){
                serde_json::from_value::<Vec<FriendData>>(user_friends_data.unwrap()).unwrap()
            } else{
                vec![]
            }; 

            let user = User::find_by_screen_cid(&owner_screen_cid_, connection).await.unwrap();
            let user_wallet_info = UserWalletInfoResponse{
                username: user.username,
                avatar: user.avatar,
                bio: user.bio,
                banner: user.banner,
                mail: user.mail,
                screen_cid: user.screen_cid,
                extra: user.extra,
                stars: user.stars,
                created_at: user.created_at.to_string(),
            };

            friends_owner_map.push(
                FriendOwnerCount{
                    owner_wallet_info: user_wallet_info,
                    friends_count: decoded_friends_data.len()
                }
            )
        }

        friends_owner_map.sort_by(|f1, f2|{

            let f1_count = f1.friends_count;
            let f2_count = f2.friends_count;

            f2_count.cmp(&f1_count)

        });
        
        Ok(friends_owner_map)
                
    }

    pub async fn update_user_fans_data_with_this_user(latest_user_info: UserData,
        connection: &mut DbPoolConnection) 
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
                                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                    
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
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserFan::update_user_fans_data_with_this_user");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
    

                    Err(resp_err.to_owned())
                    
                }
            }


    }

    pub async fn accept_friend_request(accept_friend_request: AcceptFriendRequest, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
            -> Result<UserFanData, PanelHttpResponse>{

        let AcceptFriendRequest { owner_cid, friend_screen_cid, tx_signature, hash_data } 
            = accept_friend_request;

        let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid.clone());
        let get_user_fan = Self::get_user_fans_data_for(&owner_screen_cid, connection).await;
        let Ok(user_fan_data) = get_user_fan else{
            let resp_error = get_user_fan.unwrap_err();
            return Err(resp_error);
        };

        let get_user = User::find_by_screen_cid(owner_screen_cid, connection).await;
        let Ok(user) = get_user else{

            let resp_err = get_user.unwrap_err();
            return Err(resp_err);
        };

        let get_friend = User::find_by_screen_cid(&friend_screen_cid.clone(), connection).await;
        let Ok(friend_info) = get_friend else{

            let resp_err = get_friend.unwrap_err();
            return Err(resp_err);
        };

        let user_friends_data = user_fan_data.construct_friends_data(connection);
        let mut decoded_friends_data = if user_friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(user_friends_data.unwrap()).unwrap()
        } else{
            vec![]
        };
        

        match UserFriend::accept_request(user.id, friend_info.id, connection).await{

            Ok(friend_req_data) => {

                let user_fan_data = UserFanData{
                    id: user_fan_data.id,
                    user_screen_cid: user_fan_data.clone().user_screen_cid,
                    friends: user_fan_data.clone().construct_friends_data(connection),
                    invitation_requests: user_fan_data.clone().construct_gallery_invitation_requests(connection),
                    created_at: user_fan_data.clone().created_at.to_string(),
                    updated_at: user_fan_data.clone().updated_at.to_string(),
                };
                
                /** -------------------------------------------------------------------- */
                /** ----------------- publish new event data to `on_user_action` channel */
                /** -------------------------------------------------------------------- */
                let actioner_wallet_info = UserWalletInfoResponse{
                    username: user.username,
                    avatar: user.avatar,
                    bio: user.bio,
                    banner: user.banner,
                    mail: user.mail,
                    screen_cid: user.screen_cid,
                    extra: user.extra,
                    stars: user.stars,
                    created_at: user.created_at.to_string(),
                };
                let user_wallet_info = UserWalletInfoResponse{
                    username: friend_info.username,
                    avatar: friend_info.avatar,
                    bio: friend_info.bio,
                    banner: friend_info.banner,
                    mail: friend_info.mail,
                    screen_cid: friend_info.screen_cid,
                    extra: friend_info.extra,
                    stars: friend_info.stars,
                    created_at: friend_info.created_at.to_string(),
                };
                let user_notif_info = SingleUserNotif{
                    wallet_info: user_wallet_info,
                    notif: NotifData{
                        actioner_wallet_info,
                        fired_at: Some(chrono::Local::now().timestamp()),
                        action_type: ActionType::AcceptFriendRequest,
                        action_data: serde_json::to_value(user_fan_data.clone()).unwrap()
                    }
                };
                let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

                Ok(
                    user_fan_data
                )
            },
            Err(err) => Err(err)
        }        
        

    }

    /*  
        this will be used to fetch the user unaccepted invitation requests
        inside the user fan data from any gallery owner
    */
    pub async fn get_user_unaccepted_invitation_requests(owner_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) 
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
        let Ok(mut user_fan_data) = get_user_fan_data else{

            let resp_err = get_user_fan_data.unwrap_err();
            return Err(resp_err);
        };


        let user_invitation_request_data = user_fan_data.construct_gallery_invitation_requests(connection);
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
        let sliced = if from < unaccepted_ones.len(){
            if unaccepted_ones.len() > to{
                let data = &unaccepted_ones[from..to+1];
                data.to_vec()
            } else{
                let data = &unaccepted_ones[from..unaccepted_ones.len()];
                data.to_vec()
            }
        } else{
            vec![]
        };

        Ok(
            if sliced.contains(&None){
                vec![]
            } else{
                sliced.to_owned()
            }
        )

    }

    pub async fn get_user_unaccepted_friend_requests(owner_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) 
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


        let user_friends_data = user_fan_data.construct_friends_data(connection);
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
        let sliced = if from < unaccepted_ones.len(){
            if unaccepted_ones.len() > to{
                let data = &unaccepted_ones[from..to+1];
                data.to_vec()
            } else{
                let data = &unaccepted_ones[from..unaccepted_ones.len()];
                data.to_vec()
            }
        } else{
            vec![]
        };

        Ok(
            if sliced.contains(&None){
                vec![]
            } else{
                sliced.to_owned()
            }
        )

    }

    pub async fn get_user_unaccepted_friend_requests_without_limit(owner_screen_cid: &str,
        connection: &mut DbPoolConnection) 
    -> Result<Vec<Option<FriendData>>, PanelHttpResponse>{
        
        let get_user_fan_data = Self::get_user_fans_data_for(owner_screen_cid, connection).await;
        let Ok(user_fan_data) = get_user_fan_data else{

            let resp_err = get_user_fan_data.unwrap_err();
            return Err(resp_err);
        };


        let user_friends_data = user_fan_data.construct_friends_data(connection);
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

        Ok(
            if unaccepted_ones.contains(&None){
                vec![]
            } else{
                unaccepted_ones.to_owned()
            }
        )

    }

    pub async fn get_user_fans_data_for(owner_screen_cid: &str, 
        connection: &mut DbPoolConnection) 
    -> Result<UserFanData, PanelHttpResponse>{


        let user_fan_data = users_fans
            .filter(users_fans::user_screen_cid.eq(owner_screen_cid))
            .first::<UserFan>(connection);

        let Ok(mut fan_data) = user_fan_data else{

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
                user_screen_cid: fan_data.clone().user_screen_cid,
                friends: fan_data.clone().construct_friends_data(connection),
                invitation_requests: fan_data.clone().construct_gallery_invitation_requests(connection),
                created_at: fan_data.clone().created_at.to_string(),
                updated_at: fan_data.clone().updated_at.to_string(),
            }
        )

    }


    /* -------------------- 
    // both screen cids must have each other in their friends data 
    // and they must have accepted each other's request
    -------------------- */
    pub async fn are_we_friends(first_screen_cid: &str, second_screen_cid: &str, connection: &mut DbPoolConnection) -> Result<bool, PanelHttpResponse>{

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
        let first_friends_data = first_fan_data.clone().construct_friends_data(connection);
        let decoded_first_friends_data = if first_friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(first_friends_data.clone().unwrap()).unwrap()
        } else{
            vec![]
        };

        let second_friends_data = second_fan_data.clone().construct_friends_data(connection);
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
        connection: &mut DbPoolConnection) 
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

        let Ok(mut fan_data) = user_fan_data else{

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

        let friends_data = fan_data.clone().construct_friends_data(connection);
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
                user_screen_cid: fan_data.clone().user_screen_cid,
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
                        let sliced = if from < both_friend_data_arr.len(){
                            if both_friend_data_arr.len() > to{
                                let data = &both_friend_data_arr[from..to+1];
                                data.to_vec()
                            } else{
                                let data = &both_friend_data_arr[from..both_friend_data_arr.len()];
                                data.to_vec()
                            }
                        } else{
                            vec![]
                        };
    
                        Some(serde_json::to_value(sliced).unwrap())
                    }
                },
                invitation_requests: fan_data.clone().construct_gallery_invitation_requests(connection),
                created_at: fan_data.clone().created_at.to_string(),
                updated_at: fan_data.clone().updated_at.to_string(),
            }
        )

    }

    pub async fn get_all_my_friends_without_limit(owner_screen_cid: &str,
        connection: &mut DbPoolConnection) 
    -> Result<UserFanData, PanelHttpResponse>{

        let user_fan_data = users_fans
            .filter(users_fans::user_screen_cid.eq(owner_screen_cid))
            .first::<UserFan>(connection);

        let Ok(mut fan_data) = user_fan_data else{

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

        let friends_data = fan_data.clone().construct_friends_data(connection);
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
                user_screen_cid: fan_data.clone().user_screen_cid,
                friends: {

                    if both_friend_data_arr.is_empty(){
                        Some(serde_json::to_value(both_friend_data_arr).unwrap())
                    } else{
                        
                        Some(serde_json::to_value(both_friend_data_arr).unwrap())
                    }
                },
                invitation_requests: fan_data.clone().construct_gallery_invitation_requests(connection),
                created_at: fan_data.clone().created_at.to_string(),
                updated_at: fan_data.clone().updated_at.to_string(),
            }
        )

    }

    /* -------------------- 
    // get those ones inside the owner friend data who
    // the owner has accepted their requests but they
    // must not be friend with each other
    -------------------- */
    pub async fn get_all_my_followers(owner_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) 
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

        let Ok(mut fan_data) = user_fan_data else{

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
        let friends_data = fan_data.clone().construct_friends_data(connection);
        let mut decoded_friends_data = if friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
        } else{
            vec![]
        }; 

        let mut owner_followers = vec![];
        for dfrd in decoded_friends_data{
            // owner_screen_cid and frd.screen_cid must not be friend already
            let are_we_friends = Self::are_we_friends(
                &owner_screen_cid, 
                &dfrd.screen_cid, 
                connection
            ).await;

            if are_we_friends.is_ok() && are_we_friends.unwrap(){
                continue;
            } 
            
            if dfrd.is_accepted{
                owner_followers.push(Some(dfrd));
            } else{
                owner_followers.push(None);
            }
            
        }

        owner_followers.retain(|frd| frd.is_some());

        Ok(
            UserFanData{
                id: fan_data.id,
                user_screen_cid: fan_data.clone().user_screen_cid,
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
                        let sliced = if from < owner_followers.len(){
                            if owner_followers.len() > to{
                                let data = &owner_followers[from..to+1];
                                data.to_vec()
                            } else{
                                let data = &owner_followers[from..owner_followers.len()];
                                data.to_vec()
                            }
                        } else{
                            vec![]
                        };
    
                        Some(serde_json::to_value(sliced).unwrap())
                    },
                invitation_requests: fan_data.construct_gallery_invitation_requests(connection),
                created_at: fan_data.created_at.to_string(),
                updated_at: fan_data.updated_at.to_string(),
            }
        )   

    }

    pub async fn get_all_my_followers_without_limit(owner_screen_cid: &str,
        connection: &mut DbPoolConnection) 
    -> Result<UserFanData, PanelHttpResponse>{

        let user_fan_data = users_fans
            .filter(users_fans::user_screen_cid.eq(owner_screen_cid))
            .first::<UserFan>(connection);

        let Ok(mut fan_data) = user_fan_data else{

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
        let friends_data = fan_data.clone().construct_friends_data(connection);
        let mut decoded_friends_data = if friends_data.is_some(){
            serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
        } else{
            vec![]
        }; 

        let mut owner_followers = vec![];
        for dfrd in decoded_friends_data{
            // owner_screen_cid and frd.screen_cid must not be friend already
            let are_we_friends = Self::are_we_friends(
                &owner_screen_cid, 
                &dfrd.screen_cid, 
                connection
            ).await;

            if are_we_friends.is_ok() && are_we_friends.unwrap(){
                continue;
            } 
            
            if dfrd.is_accepted{
                owner_followers.push(Some(dfrd));
            } else{
                owner_followers.push(None);
            }
            
        }

        owner_followers.retain(|frd| frd.is_some());

        Ok(
            UserFanData{
                id: fan_data.id,
                user_screen_cid: fan_data.clone().user_screen_cid,
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
    
                        Some(serde_json::to_value(owner_followers).unwrap())
                    },
                invitation_requests: fan_data.construct_gallery_invitation_requests(connection),
                created_at: fan_data.created_at.to_string(),
                updated_at: fan_data.updated_at.to_string(),
            }
        )   

    }

    /* -------------------- 
    // get those ones inside the users_fans table who
    // have owner in their friend data and are 
    // not friend we each other
    -------------------- */
    pub async fn get_all_my_followings(who_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) 
        -> Result<Vec<UserFanDataWithWalletInfo>, PanelHttpResponse>{

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

            let all_fans_data = users_fans
                .order(created_at.desc())
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
            for mut fan_data in fans_data{

                // ignore the caller friends field 
                if fan_data.user_screen_cid == who_screen_cid{
                    continue;
                }

                let friends_data = fan_data.clone().construct_friends_data(connection);
                let mut decoded_friends_data = if friends_data.is_some(){
                    serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
                } else{
                    vec![]
                }; 
                
                for friend in decoded_friends_data{
                    
                    // who_screen_cid and friend.screen_cid must not be friend already
                    let are_we_friends = Self::are_we_friends(
                        &who_screen_cid, 
                        &friend.screen_cid, 
                        connection
                    ).await;

                    if are_we_friends.is_ok() && are_we_friends.unwrap(){
                        continue;
                    }
                    
                    if friend.screen_cid == who_screen_cid{
                        followings.push({
                            UserFanDataWithWalletInfo{
                                id: fan_data.id,
                                user_wallet_info: {
                                    let user = User::find_by_screen_cid(&fan_data.clone().user_screen_cid, connection).await.unwrap_or(User::default());
                                    UserWalletInfoResponse{
                                        username: user.username,
                                        avatar: user.avatar,
                                        bio: user.bio,
                                        banner: user.banner,
                                        mail: user.mail,
                                        screen_cid: user.screen_cid,
                                        extra: user.extra,
                                        stars: user.stars,
                                        created_at: user.created_at.to_string(),
                                    }
                                },
                                friends: fan_data.clone().construct_friends_data(connection),
                                invitation_requests: fan_data.clone().construct_gallery_invitation_requests(connection),
                                created_at: fan_data.created_at.to_string(),
                                updated_at: fan_data.updated_at.to_string(),
                            }
                        });
                    }
                }

            }

            followings.sort_by(|uf1, uf2|{

                let uf1_created_at = NaiveDateTime
                    ::parse_from_str(uf1.clone().created_at.as_str(), "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();
    
                let uf2_created_at = NaiveDateTime
                    ::parse_from_str(uf2.clone().created_at.as_str(), "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();
    
                uf2_created_at.cmp(&uf1_created_at)
    
            });      
            
            let sliced = if from < followings.len(){
                if followings.len() > to{
                    let data = &followings[from..to+1];
                    data.to_vec()
                } else{
                    let data = &followings[from..followings.len()];
                    data.to_vec()
                }
            } else{
                vec![]
            };

            // can't return UserFanData cause user might have no one sent him a request yet
            // so there is no record for who_screen_cid yet thus users_fans would be empty.
            Ok(
                sliced
            ) 

        }
    
        pub async fn get_all_my_followings_without_limit(who_screen_cid: &str,
            connection: &mut DbPoolConnection) 
            -> Result<Vec<UserFanDataWithWalletInfo>, PanelHttpResponse>{

            let all_fans_data = users_fans
                .order(created_at.desc())
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
            for mut fan_data in fans_data{

                // ignore the caller friends field 
                if fan_data.user_screen_cid == who_screen_cid{
                    continue;
                }

                let friends_data = fan_data.clone().construct_friends_data(connection);
                let mut decoded_friends_data = if friends_data.is_some(){
                    serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
                } else{
                    vec![]
                }; 
                
                for friend in decoded_friends_data{
                    
                    // who_screen_cid and friend.screen_cid must not be friend already
                    let are_we_friends = Self::are_we_friends(
                        &who_screen_cid, 
                        &friend.screen_cid, 
                        connection
                    ).await;

                    if are_we_friends.is_ok() && are_we_friends.unwrap(){
                        continue;
                    }
                    
                    if friend.screen_cid == who_screen_cid{
                        followings.push({
                            UserFanDataWithWalletInfo{
                                id: fan_data.id,
                                user_wallet_info: {
                                    let user = User::find_by_screen_cid(&fan_data.clone().user_screen_cid, connection).await.unwrap_or(User::default());
                                    UserWalletInfoResponse{
                                        username: user.username,
                                        avatar: user.avatar,
                                        bio: user.bio,
                                        banner: user.banner,
                                        mail: user.mail,
                                        screen_cid: user.screen_cid,
                                        extra: user.extra,
                                        stars: user.stars,
                                        created_at: user.created_at.to_string(),
                                    }
                                },
                                friends: fan_data.clone().construct_friends_data(connection),
                                invitation_requests: fan_data.clone().construct_gallery_invitation_requests(connection),
                                created_at: fan_data.created_at.to_string(),
                                updated_at: fan_data.updated_at.to_string(),
                            }
                        });
                    }
                }

            }

            followings.sort_by(|uf1, uf2|{

                let uf1_created_at = NaiveDateTime
                    ::parse_from_str(uf1.clone().created_at.as_str(), "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();
    
                let uf2_created_at = NaiveDateTime
                    ::parse_from_str(uf2.clone().created_at.as_str(), "%Y-%m-%d %H:%M:%S%.f")
                    .unwrap();
    
                uf2_created_at.cmp(&uf1_created_at)
    
            });      

            Ok(
                followings
            ) 

        }

    pub async fn get_user_relations(who_screen_cid: &str, limit: web::Query<Limit>,
        connection: &mut DbPoolConnection) 
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

    pub async fn get_user_relations_without_limit(who_screen_cid: &str,
        connection: &mut DbPoolConnection) 
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
                    let get_followers = Self::get_all_my_followers_without_limit(who_screen_cid, connection).await;
                    let Ok(user_followers) = get_followers else{
                        let err_resp = get_followers.unwrap_err();
                        return Err(err_resp);
                    };
                    user_followers
                },
                friends: {
                    let get_friends = Self::get_all_my_friends_without_limit(who_screen_cid, connection).await;
                    let Ok(user_friends) = get_friends else{
                        let err_resp = get_friends.unwrap_err();
                        return Err(err_resp);
                    };
                    user_friends
                },
                followings: {
                    let get_followings = Self::get_all_my_followings_without_limit(who_screen_cid, connection).await;
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
        connection: &mut DbPoolConnection) 
            -> Result<UserFanData, PanelHttpResponse>{
        
            let RemoveFollower { owner_cid, follower_screen_cid, tx_signature, hash_data } 
                = remove_follower_request;
            
            let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid.clone());
            let get_user_fan = Self::get_user_fans_data_for(&owner_screen_cid, connection).await;
            let Ok(mut user_fan_data) = get_user_fan else{
                let resp_error = get_user_fan.unwrap_err();
                return Err(resp_error);
            };
    
            let user_friends_data = user_fan_data.clone().construct_friends_data(connection);
            let mut decoded_friends_data = if user_friends_data.is_some(){
                serde_json::from_value::<Vec<FriendData>>(user_friends_data.unwrap()).unwrap()
            } else{
                vec![]
            };

            let get_user = User::find_by_screen_cid(owner_screen_cid, connection).await;
            let Ok(user) = get_user else{

                let resp_err = get_user.unwrap_err();
                return Err(resp_err);
            };

            let get_friend = User::find_by_screen_cid(&follower_screen_cid.clone(), connection).await;
            let Ok(friend_info) = get_friend else{

                let resp_err = get_friend.unwrap_err();
                return Err(resp_err);
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
            
                match UserFriend::remove_follower(user.id, friend_info.id, connection){
                    Ok(removed_records) => {
                        Ok(
                            UserFanData{
                                id: user_fan_data.id,
                                user_screen_cid: user_fan_data.clone().user_screen_cid,
                                friends: user_fan_data.clone().construct_friends_data(connection),
                                invitation_requests: user_fan_data.clone().construct_gallery_invitation_requests(connection),
                                created_at: user_fan_data.clone().created_at.to_string(),
                                updated_at: user_fan_data.clone().updated_at.to_string(),
                            }
                        )
                    },
                    Err(resp) => Err(resp)
                }
                
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

    pub async fn delete_by_screen_cid(u_scid: &str, connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<usize, PanelHttpResponse>{

        match diesel::delete(users_fans
            .filter(users_fans::user_screen_cid.eq(u_scid)))
            .execute(connection)
            {
                Ok(mut num_deleted) => {
                    
                    /* also delete any users record if there was any */

                    Ok(num_deleted)
                
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "UserFan::delete_by_screen_cid");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn remove_friend(remove_friend_request: RemoveFriend, redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
            -> Result<UserFanData, PanelHttpResponse>{
        
            let RemoveFriend { owner_cid, friend_screen_cid, tx_signature, hash_data } 
                = remove_friend_request;
    
            let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid.clone());
            let get_user_fan = Self::get_user_fans_data_for(&friend_screen_cid, connection).await;
            let Ok(mut user_fan_data) = get_user_fan else{
                let resp_error = get_user_fan.unwrap_err();
                return Err(resp_error);
            };

            let get_user = User::find_by_screen_cid(owner_screen_cid, connection).await;
            let Ok(user) = get_user else{

                let resp_err = get_user.unwrap_err();
                return Err(resp_err);
            };

            let get_friend = User::find_by_screen_cid(&friend_screen_cid.clone(), connection).await;
            let Ok(friend_info) = get_friend else{

                let resp_err = get_friend.unwrap_err();
                return Err(resp_err);
            };
    
            let user_friends_data = user_fan_data.clone().construct_friends_data(connection);
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
                    let get_all_private_galleries = UserPrivateGallery::get_all_for_without_limit(owner_screen_cid, redis_client.clone(), connection);
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
                                        futures::executor::block_on( // we can't use .await in none async context thus we're using block_on() method
                                            // update private gallery
                                            UserPrivateGallery::update(&gallery.clone().owner_screen_cid, 
                                                UpdateUserPrivateGalleryRequest{
                                                        owner_cid: owner_cid.clone(),
                                                        collections: gallery.clone().collections,
                                                        gal_name: gallery.clone().gal_name,
                                                        gal_description: gallery.clone().gal_description,
                                                        invited_friends: Some(inv_frds.clone()),
                                                        extra: gallery.clone().extra,
                                                        tx_signature: String::from(""),
                                                        hash_data: String::from(""),
                                                    }, 
                                                    redis_client.clone(),
                                                    redis_actor.clone(),
                                                    gallery.clone().id,
                                                    connection)
                                                
                                        );
                                        
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
            
                // remove friend from table
                match UserFriend::remove(user.id, friend_info.id, connection){
                    Ok(removed_records) => {
                        Ok(
                            UserFanData{
                                id: user_fan_data.id,
                                user_screen_cid: user_fan_data.clone().user_screen_cid,
                                friends: user_fan_data.clone().construct_friends_data(connection),
                                invitation_requests: user_fan_data.clone().construct_gallery_invitation_requests(connection),
                                created_at: user_fan_data.clone().created_at.to_string(),
                                updated_at: user_fan_data.clone().updated_at.to_string(),
                            }
                        )
                    },
                    Err(resp) => Err(resp)
                }
                
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

    pub async fn remove_following(remove_friend_request: RemoveFollowing, redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
            -> Result<UserFanData, PanelHttpResponse>{
        
            let RemoveFollowing { owner_cid, friend_screen_cid, tx_signature, hash_data } 
                = remove_friend_request;
    
            let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid.clone());
            let get_user_fan = Self::get_user_fans_data_for(&friend_screen_cid, connection).await;
            let Ok(mut user_fan_data) = get_user_fan else{
                let resp_error = get_user_fan.unwrap_err();
                return Err(resp_error);
            };
    
            let user_friends_data = user_fan_data.clone().construct_friends_data(connection);
            let mut decoded_friends_data = if user_friends_data.is_some(){
                serde_json::from_value::<Vec<FriendData>>(user_friends_data.unwrap()).unwrap()
            } else{
                vec![]
            };

            let get_user = User::find_by_screen_cid(owner_screen_cid, connection).await;
            let Ok(user) = get_user else{

                let resp_err = get_user.unwrap_err();
                return Err(resp_err);
            };

            let get_friend = User::find_by_screen_cid(&friend_screen_cid.clone(), connection).await;
            let Ok(friend_info) = get_friend else{

                let resp_err = get_friend.unwrap_err();
                return Err(resp_err);
            };
            
            for friend_data in decoded_friends_data{
                if friend_data.screen_cid == *owner_screen_cid && !friend_data.is_accepted{
                    return match UserFriend::remove(user.id, friend_info.id, connection){
                        Ok(removed_records) => {
                            Ok(
                                UserFanData{
                                    id: user_fan_data.id,
                                    user_screen_cid: user_fan_data.clone().user_screen_cid,
                                    friends: user_fan_data.clone().construct_friends_data(connection),
                                    invitation_requests: user_fan_data.clone().construct_gallery_invitation_requests(connection),
                                    created_at: user_fan_data.clone().created_at.to_string(),
                                    updated_at: user_fan_data.clone().updated_at.to_string(),
                                }
                            )
                        },
                        Err(resp) => Err(resp)
                    }
                }
            } 

        // return the whole data of the user fan even if there was a removed record
        Ok(
            UserFanData{
                id: user_fan_data.id,
                user_screen_cid: user_fan_data.clone().user_screen_cid,
                friends: user_fan_data.clone().construct_friends_data(connection),
                invitation_requests: user_fan_data.clone().construct_gallery_invitation_requests(connection),
                created_at: user_fan_data.clone().created_at.to_string(),
                updated_at: user_fan_data.clone().updated_at.to_string(),
            }
        )

    }

    pub async fn send_friend_request_to(send_friend_request: SendFriendRequest, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
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

        let user_screen_cid_ = friend_info.clone().screen_cid.unwrap_or_default();
        match Self::get_user_fans_data_for(&user_screen_cid_, connection).await{

            /* already inserted just update the friends field */
            Ok(user_fan_data) => {

                let friends_data = user_fan_data.clone().construct_friends_data(connection);
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

                match UserFriend::insert(NewFriendRequest{
                    user_id: user.id,
                    friend_id: friend_info.id,
                    is_accepted: false,
                    requested_at: chrono::Local::now().timestamp(),
                }, connection).await{
                    Ok(friend_req_data) => { // it might be a found data or inserted one
                        Ok(
                            UserFanData{
                                id: user_fan_data.id,
                                user_screen_cid: user_fan_data.clone().user_screen_cid,
                                friends: user_fan_data.clone().construct_friends_data(connection),
                                invitation_requests: user_fan_data.clone().construct_gallery_invitation_requests(connection),
                                created_at: user_fan_data.clone().created_at.to_string(),
                                updated_at: user_fan_data.clone().updated_at.to_string(),
                            }
                        )
                    },
                    Err(resp) => Err(resp)
                }
                
            },
            /* insert new record */
            Err(resp) => {
                
                match UserFriend::insert(NewFriendRequest{
                    user_id: user.id,
                    friend_id: friend_info.id,
                    is_accepted: false,
                    requested_at: chrono::Local::now().timestamp(),
                }, connection).await{
                    Ok(friend_req_data) => { // it might be a found data or inserted one

                        let new_fan_data = InsertNewUserFanRequest{
                            user_screen_cid: user_screen_cid_.to_owned(),
                            friends: Some(serde_json::to_value::<Vec<FriendData>>(vec![]).unwrap()),
                            invitation_requests: Some(serde_json::to_value::<Vec<InvitationRequestData>>(vec![]).unwrap()),
                        };

                        /* return the last record */
                        match diesel::insert_into(users_fans)
                        .values(&new_fan_data)
                        .returning(UserFan::as_returning())
                        .get_result::<UserFan>(connection)
                        {
                            Ok(mut user_fan_data) => {

                                let fan_info = UserFanData{ 
                                    id: user_fan_data.id, 
                                    user_screen_cid: user_fan_data.clone().user_screen_cid, 
                                    friends: user_fan_data.clone().construct_friends_data(connection), 
                                    invitation_requests: user_fan_data.clone().construct_gallery_invitation_requests(connection), 
                                    created_at: user_fan_data.clone().created_at.to_string(), 
                                    updated_at: user_fan_data.clone().updated_at.to_string() 
                                };

                                /** -------------------------------------------------------------------- */
                                /** ----------------- publish new event data to `on_user_action` channel */
                                /** -------------------------------------------------------------------- */
                                let actioner_wallet_info = UserWalletInfoResponse{
                                    username: user.username,
                                    avatar: user.avatar,
                                    bio: user.bio,
                                    banner: user.banner,
                                    mail: user.mail,
                                    screen_cid: user.screen_cid,
                                    extra: user.extra,
                                    stars: user.stars,
                                    created_at: user.created_at.to_string(),
                                };
                                let user_wallet_info = UserWalletInfoResponse{
                                    username: friend_info.username,
                                    avatar: friend_info.avatar,
                                    bio: friend_info.bio,
                                    banner: friend_info.banner,
                                    mail: friend_info.mail,
                                    screen_cid: friend_info.screen_cid,
                                    extra: friend_info.extra,
                                    stars: friend_info.stars,
                                    created_at: friend_info.created_at.to_string(),
                                };
                                let user_notif_info = SingleUserNotif{
                                    wallet_info: user_wallet_info,
                                    notif: NotifData{
                                        actioner_wallet_info,
                                        fired_at: Some(chrono::Local::now().timestamp()),
                                        action_type: ActionType::FriendRequestFrom,
                                        action_data: serde_json::to_value(fan_info.clone()).unwrap()
                                    }
                                };
                                let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                                events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;
            
                                Ok(
                                    fan_info
                                )
            
                            },
                            Err(e) => {
            
                                let resp_err = &e.to_string();

                                /* custom error handler */
                                use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                
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

                    },
                    Err(resp) => Err(resp)
                }

            }
        }

    }

    pub fn construct_gallery_invitation_requests(&mut self, connection: &mut DbPoolConnection) -> Option<serde_json::Value>{

        let owner_id = User::find_by_screen_cid_none_async(&self.user_screen_cid, connection).unwrap().id;
        
        let get_invitation_requests_data = PrivateGalleryInvitationRequest::get_all_for_user(owner_id, connection);
        let invitation_requests_data = if get_invitation_requests_data.is_ok(){
            get_invitation_requests_data.unwrap()
        } else{
            vec![]
        };

        let mut all_inv_reqs = vec![];
        for inv_req in invitation_requests_data{
            all_inv_reqs.push(
                InvitationRequestData{
                    username: {
                        User::find_by_id_none_async(inv_req.invitee_id, connection).unwrap().username
                    },
                    user_avatar: {
                        User::find_by_id_none_async(inv_req.invitee_id, connection).unwrap().avatar
                    },
                    from_screen_cid: {
                        User::find_by_id_none_async(inv_req.from_user_id, connection).unwrap().screen_cid.unwrap()
                    },
                    requested_at: inv_req.requested_at,
                    gallery_id: inv_req.gal_id,
                    is_accepted: inv_req.is_accepted,
                }
            )
        }

        Some(
            serde_json::to_value(&all_inv_reqs).unwrap()
        )

    }

    pub async fn push_invitation_request_for(owner_screen_cid: &str, redis_actor: Addr<RedisActor>,
        invitation_request_data: InvitationRequestData,
        connection: &mut DbPoolConnection) 
            -> Result<InvitationRequestDataResponse, PanelHttpResponse>{
        
        let request_sender = invitation_request_data.clone().from_screen_cid;

        let get_user = User::find_by_screen_cid(owner_screen_cid, connection).await;
        let Ok(user) = get_user else{

            let resp_err = get_user.unwrap_err();
            return Err(resp_err);
        };

        let get_request_sender_info = User::find_by_screen_cid(&request_sender.clone(), connection).await;
        let Ok(request_sender_info) = get_request_sender_info else{

            let resp_err = get_request_sender_info.unwrap_err();
            return Err(resp_err);
        };

        let check_we_are_friend = Self::are_we_friends(
            &request_sender, 
            owner_screen_cid, connection).await;
        
        let Ok(are_we_friend) = check_we_are_friend else{
            let err_resp = check_we_are_friend.unwrap_err();
            return Err(err_resp);
        };
        
        if are_we_friend{

            // getting all gallery invitation requests for the passed in invitee
            let get_invitation_requests_data = PrivateGalleryInvitationRequest::get_all_for_user(user.id, connection);
            let invitation_requests_data = if get_invitation_requests_data.is_ok(){
                get_invitation_requests_data.unwrap()
            } else{
                vec![]
            };

            let mut all_inv_reqs = vec![];
            for inv_req in invitation_requests_data{
                all_inv_reqs.push(
                    InvitationRequestData{
                        username: {
                            User::find_by_id(inv_req.invitee_id, connection).await.unwrap().username
                        },
                        user_avatar: {
                            User::find_by_id(inv_req.invitee_id, connection).await.unwrap().avatar
                        },
                        from_screen_cid: {
                            User::find_by_id(inv_req.from_user_id, connection).await.unwrap().screen_cid.unwrap()
                        },
                        requested_at: inv_req.requested_at,
                        gallery_id: inv_req.gal_id,
                        is_accepted: inv_req.is_accepted,
                    }
                )
            }


            let mut is_this_new_invitation = false;
            if !all_inv_reqs
                .clone()
                .into_iter()
                .any(|invrd| invrd.gallery_id == invitation_request_data.gallery_id){

                    is_this_new_invitation = true;

                }

            if is_this_new_invitation{
                // insert into PrivateGalleryInvitationRequest table
                let get_inserted_inv_req_data = PrivateGalleryInvitationRequest::insert(
                    NewPrivateGalleryInvitationRequest{
                        invitee_id: user.id,
                        from_user_id: request_sender_info.id,
                        gal_id: invitation_request_data.gallery_id,
                        is_accepted: false,
                        requested_at: invitation_request_data.requested_at
                    }, connection).await;
                let Ok(inserted_inv_req_data) = get_inserted_inv_req_data else{
                    let err_resp = get_inserted_inv_req_data.unwrap_err();
                    return Err(err_resp);
                };
            }


            let invitation_request_data_response = InvitationRequestDataResponse{
                to_screen_cid: owner_screen_cid.to_string(),
                from_screen_cid: request_sender,
                requested_at: invitation_request_data.requested_at,
                gallery_id: invitation_request_data.gallery_id,
                is_accepted: invitation_request_data.is_accepted,
                username: invitation_request_data.username,
                user_avatar: invitation_request_data.user_avatar
            };

            /** -------------------------------------------------------------------- */
            /** ----------------- publish new event data to `on_user_action` channel */
            /** -------------------------------------------------------------------- */
            let actioner_wallet_info = UserWalletInfoResponse{
                username: request_sender_info.username,
                avatar: request_sender_info.avatar,
                bio: request_sender_info.bio,
                banner: request_sender_info.banner,
                mail: request_sender_info.mail,
                screen_cid: request_sender_info.screen_cid,
                extra: request_sender_info.extra,
                stars: request_sender_info.stars,
                created_at: request_sender_info.created_at.to_string(),
            };
            let user_wallet_info = UserWalletInfoResponse{
                username: user.username,
                avatar: user.avatar,
                bio: user.bio,
                banner: user.banner,
                mail: user.mail,
                screen_cid: user.screen_cid,
                extra: user.extra,
                stars: user.stars,
                created_at: user.created_at.to_string(),
            };
            let user_notif_info = SingleUserNotif{
                wallet_info: user_wallet_info,
                notif: NotifData{
                    actioner_wallet_info,
                    fired_at: Some(chrono::Local::now().timestamp()),
                    action_type: ActionType::InvitationRequestFrom,
                    action_data: serde_json::to_value(invitation_request_data_response.clone()).unwrap()
                }
            };
            let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
            events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;
            
            Ok(
                invitation_request_data_response
            )


        } else{

            let request_sender_username = request_sender_info.clone().username;
            let owner_username = user.clone().username;
            let resp_msg = format!("{request_sender_username:} Is Not A Friend Of {owner_username:}");
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
        connection: &mut DbPoolConnection) 
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

        let get_request_sender_info = User::find_by_screen_cid(&from_screen_cid.clone(), connection).await;
        let Ok(request_sender_info) = get_request_sender_info else{

            let resp_err = get_request_sender_info.unwrap_err();
            return Err(resp_err);
        };

        let get_gallery_data = UserPrivateGallery::find_by_id(gal_id, redis_client.clone(), connection).await;
        let Ok(gallery) = get_gallery_data else{
            let resp_error = get_gallery_data.unwrap_err();
            return Err(resp_error);
        };

        let get_user_fan = Self::get_user_fans_data_for(&owner_screen_cid, connection).await;
        let Ok(mut user_fan_data) = get_user_fan else{
            let resp_error = get_user_fan.unwrap_err();
            return Err(resp_error);
        };

        match PrivateGalleryInvitationRequest::accept_request(user.id, gal_id, connection).await{

            Ok(updated_private_gallery_invitation_request) => {

                let final_user_fan_data = UserFanData{
                    id: user_fan_data.id,
                    user_screen_cid: user_fan_data.clone().user_screen_cid,
                    friends: user_fan_data.clone().construct_friends_data(connection),
                    invitation_requests: user_fan_data.clone().construct_gallery_invitation_requests(connection),
                    created_at: user_fan_data.clone().created_at,
                    updated_at: user_fan_data.clone().updated_at,
                };

                // if the updation process of users_fans data was successful then we simply
                // add the caller to invited_friend of the gallery owner
                let gallery_invited_friends = gallery.invited_friends;
                let mut invited_friends = if gallery_invited_friends.is_some(){
                    gallery_invited_friends.unwrap()
                } else{
                    vec![]
                };

                // push the one who has accepted the request into the invited_friends
                // of the gallery owner if it wasn't already in there
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
                    }, redis_client.clone(), redis_actor.clone(), gal_id, connection).await{

                        Ok(_) => {
                            
                            /** -------------------------------------------------------------------- */
                            /** ----------------- publish new event data to `on_user_action` channel */
                            /** -------------------------------------------------------------------- */
                            let actioner_wallet_info = UserWalletInfoResponse{
                                username: user.username,
                                avatar: user.avatar,
                                bio: user.bio,
                                banner: user.banner,
                                mail: user.mail,
                                screen_cid: user.screen_cid,
                                extra: user.extra,
                                stars: user.stars,
                                created_at: user.created_at.to_string(),
                            };
                            let user_wallet_info = UserWalletInfoResponse{
                                username: request_sender_info.username,
                                avatar: request_sender_info.avatar,
                                bio: request_sender_info.bio,
                                banner: request_sender_info.banner,
                                mail: request_sender_info.mail,
                                screen_cid: request_sender_info.screen_cid,
                                extra: request_sender_info.extra,
                                stars: request_sender_info.stars,
                                created_at: request_sender_info.created_at.to_string(),
                            };
                            let user_notif_info = SingleUserNotif{
                                wallet_info: user_wallet_info,
                                notif: NotifData{
                                    actioner_wallet_info,
                                    fired_at: Some(chrono::Local::now().timestamp()),
                                    action_type: ActionType::AcceptInvitationRequest,
                                    action_data: serde_json::to_value(final_user_fan_data.clone()).unwrap()
                                }
                            };
                            let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                            events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;
                            
                            Ok(final_user_fan_data)
                        
                        },
                        Err(resp) => return Err(resp),
                    }


            },
            Err(resp) => return Err(resp),
        }

        
    }

    pub async fn enter_private_gallery_request(enter_private_gallery_request: EnterPrivateGalleryRequest,
        redis_client: RedisClient, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
            -> Result<UserFanData, PanelHttpResponse>{

        let EnterPrivateGalleryRequest { caller_cid, owner_screen_cid, gal_id, tx_signature, hash_data } 
            = enter_private_gallery_request;

        // caller
        let caller_screen_cid = &walletreq::evm::get_keccak256_from(caller_cid.clone());
        let get_user_fan = Self::get_user_fans_data_for(&caller_screen_cid, connection).await;
        let Ok(mut user_fan_data) = get_user_fan else{
            let resp_error = get_user_fan.unwrap_err();
            return Err(resp_error);
        };

        let user_invitation_request_data = user_fan_data.construct_gallery_invitation_requests(connection);
        let mut decoded_invitation_request_data = if user_invitation_request_data.is_some(){
            serde_json::from_value::<Vec<InvitationRequestData>>(user_invitation_request_data.unwrap()).unwrap()
        } else{
            vec![]
        };

        // update invited_friends with the caller_screen_cid
        let get_gallery_data = UserPrivateGallery::find_by_id(gal_id, redis_client.clone(), connection).await;
        let Ok(gallery) = get_gallery_data else{
            let resp_error = get_gallery_data.unwrap_err();
            return Err(resp_error);
        };

        let user = User::find_by_screen_cid(&caller_screen_cid, connection).await.unwrap();
        let owner = User::find_by_screen_cid(&owner_screen_cid, connection).await.unwrap();
        let mut updated_user_balance = None;
        let mut updated_owner_balance = None;
        

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

        // don't push new invitation request to the caller related field
        // since the caller might be gets invited by gallery owner request
        let mut is_this_new_request = false;
        if !decoded_invitation_request_data
                .clone()
                .into_iter()
                .any(|invrd| invrd.gallery_id == gallery.id){

                    is_this_new_request = true;
                }

        if is_this_new_request{
            let get_inserted_inv_req_data = PrivateGalleryInvitationRequest::insert(
                NewPrivateGalleryInvitationRequest{
                    invitee_id: user.id,
                    from_user_id: owner.id,
                    gal_id: gallery.id,
                    is_accepted: true,
                    requested_at: chrono::Local::now().timestamp(),
                }, connection).await;
            let Ok(inserted_inv_req_data) = get_inserted_inv_req_data else{
                let err_resp = get_inserted_inv_req_data.unwrap_err();
                return Err(err_resp);
            };
        }

        // if the users_fans record was updated successfully then we simply push the 
        // user who has paid for the gllary into the invited_friends field of the gallery
        let gallery_invited_friends = gallery.invited_friends;
        let mut invited_friends = if gallery_invited_friends.is_some(){
            gallery_invited_friends.unwrap()
        } else{
            vec![]
        };

        /** -------------------------------------------------------------------------------------------- */
        /** ------------------------------ gallery entrance redis caching ------------------------------ */
        // caching the gallery entrance fees in redis, we'll use this to payback the caller_screen_cid
        // when the owner wants to remove the caller_screen_cid from his gallery 
        let mut conn = redis_client.get_async_connection().await.unwrap();
        let get_galleries_with_entrance_fee: redis::RedisResult<String> = conn.get("galleries_with_entrance_fee").await;
        let updated_redis_gals = match get_galleries_with_entrance_fee{
            Ok(galleries_with_entrance_fee) if !galleries_with_entrance_fee.is_empty() => {
                let mut galleries_with_entrance_fee = serde_json::from_str::<HashMap<i32, (Vec<String>, i64)>>(&galleries_with_entrance_fee).unwrap();
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
                galleries_with_entrance_fee
            },
            _ => {
                let mut gals: HashMap<i32, (Vec<String>, i64)> = HashMap::new();
                gals.insert(gal_id, (vec![caller_cid], g_entry_price));
                gals
            }
        };

        let stringified_ = serde_json::to_string_pretty(&updated_redis_gals).unwrap();
        let ـ : RedisResult<String> = conn.set("galleries_with_entrance_fee", stringified_).await;
        /** -------------------------------------------------------------------------------------------- */
        /** -------------------------------------------------------------------------------------------- */

        // if caller_screen_cid has not been invited yet means that
        // gallery owner didn't send request to him and he's entering 
        // by paing the fee so we charge him so we'll push him into 
        // invited_friends field
        if !invited_friends.contains(&Some(caller_screen_cid.to_string())){
            
            // update balance of the one who accepted the request
            // cause he must pay for the entry price of the gallery
            let new_balance = user.balance.unwrap() - g_entry_price;
            updated_user_balance = Some(User::update_balance(user.id, "EnterPrivateGallery", "Debit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await.unwrap());
    
            // update balance of the owner
            let new_owner_balance = owner.balance.unwrap() + g_entry_price;
            updated_owner_balance = Some(User::update_balance(owner.id, "EnterPrivateGallery", "Credit", new_owner_balance, redis_client.clone(), redis_actor.clone(), connection).await.unwrap());

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
            }, redis_client.clone(), redis_actor.clone(), gal_id, connection).await{

                Ok(_) => 
                    Ok(
                        UserFanData{
                            id: user_fan_data.id,
                            user_screen_cid: user_fan_data.clone().user_screen_cid,
                            friends: user_fan_data.clone().construct_friends_data(connection),
                            invitation_requests: user_fan_data.clone().construct_gallery_invitation_requests(connection),
                            created_at: user_fan_data.clone().created_at.to_string(),
                            updated_at: user_fan_data.clone().updated_at.to_string(),
                        }
                    ),
                Err(resp) => {
                    
                    if updated_user_balance.is_some() && updated_owner_balance.is_some(){
                        // revert the payment process, pay the gallery price back the user 
                        let new_balance = updated_user_balance.unwrap().balance.unwrap() + g_entry_price;
                        let updated_user_balance = User::update_balance(user.id, "ErrorEnterPrivateGallery", "Credit", new_balance, redis_client.clone(), redis_actor.clone(), connection).await.unwrap();
                        
                        // charge the owner for the gallery price
                        let new_owner_balance = updated_owner_balance.unwrap().balance.unwrap() - g_entry_price;
                        let updated_owner_balance = User::update_balance(owner.id, "ErrorEnterPrivateGallery", "Debit", new_owner_balance, redis_client.clone(), redis_actor.clone(), connection).await.unwrap();
                    }
                    
                    return Err(resp)
                },
            }


        
    }

}

impl UserFanData{

    pub fn construct_gallery_invitation_requests(&mut self, connection: &mut DbPoolConnection) -> Option<serde_json::Value>{

        let owner_id = User::find_by_screen_cid_none_async(&self.user_screen_cid, connection).unwrap().id;
        
        let get_invitation_requests_data = PrivateGalleryInvitationRequest::get_all_for_user(owner_id, connection);
        let invitation_requests_data = if get_invitation_requests_data.is_ok(){
            get_invitation_requests_data.unwrap()
        } else{
            vec![]
        };

        let mut all_inv_reqs = vec![];
        for inv_req in invitation_requests_data{
            all_inv_reqs.push(
                InvitationRequestData{
                    username: {
                        User::find_by_id_none_async(inv_req.invitee_id, connection).unwrap().username
                    },
                    user_avatar: {
                        User::find_by_id_none_async(inv_req.invitee_id, connection).unwrap().avatar
                    },
                    from_screen_cid: {
                        User::find_by_id_none_async(inv_req.from_user_id, connection).unwrap().screen_cid.unwrap()
                    },
                    requested_at: inv_req.requested_at,
                    gallery_id: inv_req.gal_id,
                    is_accepted: inv_req.is_accepted,
                }
            )
        }

        Some(
            serde_json::to_value(&all_inv_reqs).unwrap()
        )

    }

    pub fn construct_friends_data(&self, connection: &mut DbPoolConnection) -> Option<serde_json::Value>{

        let user = User::find_by_screen_cid_none_async(&self.user_screen_cid, connection).unwrap();
        let get_friend_requests_data = UserFriend::get_all_for_user(user.id, connection);
        let friend_requests_data = if get_friend_requests_data.is_ok(){
            get_friend_requests_data.unwrap()
        } else{
            vec![]
        };

        let mut friends_data = vec![];
        for frd_req in friend_requests_data{
            let friend_info = User::find_by_id_none_async(frd_req.friend_id, connection).unwrap();
            friends_data.push(
                FriendData{
                    username: friend_info.clone().username,
                    user_avatar: friend_info.clone().avatar,
                    screen_cid: friend_info.clone().screen_cid.unwrap(),
                    requested_at: frd_req.requested_at,
                    is_accepted: frd_req.is_accepted,
                }
            )
        }

        Some(
            serde_json::to_value(&friends_data).unwrap()
        )

    }

}