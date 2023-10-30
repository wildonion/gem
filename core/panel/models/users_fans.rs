


use crate::*;
use crate::constants::{NO_FANS_FOUND, STORAGE_IO_ERROR_CODE};
use crate::misc::Response;
use crate::schema::users_fans::dsl::*;
use crate::schema::users_fans;

use super::users::User;



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
pub struct FriendData{
    pub screen_cid: String,
    pub requested_at: i64,
    pub is_accepted: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InvitationRequestData{
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
    pub is_accepted: bool
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

#[derive(Clone, Debug, Serialize, Deserialize, Default, AsChangeset)]
#[diesel(table_name=users_fans)]
pub struct UpdateUserFanData{
    pub friends: Option<serde_json::Value>,
    pub invitation_requests: Option<serde_json::Value>,
}

impl UserFan{

    pub async fn add_user_to_friend(owner_screen_cid: &str, friend_screen_cid: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // update the is_accepted field to accept the friend request
        // ...

        // upsert process
        // ...

        // fetched_user_fans
        // let mut decoded_friends = serde_json::from_value::<Vec<UserNftData>>(fetched_user_fans.friends).unwrap();
        // decoded_friends.push(FriendData{
        //      screen_cid: friend_screen_cid,
        //      requested_at: ....,
        //      is_accepted: true
        // });
        // update user_fan record
        

        Ok(())

    }

    pub async fn get_user_unaccpeted_invitation_requests(owner_screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
    -> Result<UserFanData, PanelHttpResponse>{
    
    
        Ok(
            UserFanData::default()
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

    pub async fn remove_user_from_friend(owner_screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // ...

        Ok(())

    }

    pub async fn send_friend_request_to(owner_screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // add new FriendData into friends field of the user UserFan data
        // set is_accepted field to false by default 
        // ...

        Ok(())

    }

    /* 
        pushing a new invitation request for the passed in user by doing this we're updating the 
        invitation_request_data field for the user so client can get the invitation_request_data
        in an interval and check for new unaccepted ones. use get_user_unaccpeted_invitation_requests 
        method to fetch those ones that is_accepted are false.
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
        let Ok(mut user_fan_data) = get_user_fan_data else{

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
                serde_json::from_value::<Vec<InvitationRequestData>>(friends_data.unwrap()).unwrap()
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
                }
            )


        } else{

            let resp_msg = format!("{request_sender:} Is Not A Friend Of {owner_screen_cid:}");
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: &resp_msg,
                status: 406,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }


    }
    
    pub async fn accept_invitation_request(owner_screen_cid: &str, from_screen_cid: &str, gal_id: i32,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{

        // find an invitation request data in invitation_requests field then update is_accepted field
        // update invited_friends field in user gallery so they can see the galley collections
        // ...

        Ok(())
        
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
                        status: 500
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
                }
            
            }
            

    }

}