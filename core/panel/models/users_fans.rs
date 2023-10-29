


use crate::{*, schema::users_fans};




/* 

    diesel migration generate users_fans ---> create users_fans migration sql files
    diesel migration run                 ---> apply sql files to db 
    diesel migration redo                ---> drop tables 

*/
#[derive(Queryable, Selectable, Serialize, Deserialize, Insertable, Identifiable, Debug, PartialEq, Clone)]
#[diesel(table_name=users_fans)]
pub struct UserFan{
    pub id: i32,
    pub user_screen_cid: String,
    pub friends: serde_json::Value, /* pg key, value based json binary object */
    pub invitation_requests: serde_json::Value, /* pg key, value based json binary object */
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
pub struct UserFanData{
    pub id: i32,
    pub user_screen_cid: String,
    pub friends: serde_json::Value,
    pub invitation_requests: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl UserFan{

    pub async fn add_user_to_friend(user_screen_cid: &str, friend_screen_cid: &str,
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // update the is_accepted field to accept the friend request
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

    pub async fn remove_user_from_friend(user_screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // ...

        Ok(())

    }

    pub async fn send_friend_request_to(user_screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // add new FriendData into friends field of the user UserFan data
        // set is_accepted field to false by default 
        // ...

        Ok(())

    }

    pub async fn push_invitation_request_for(user_screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // ...

        Ok(())

    }
    
    pub async fn accept_invitation_request(user_screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{

        // update invited_friends field in user gallery
        // ...

        Ok(())
        
    }

}