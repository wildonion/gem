


use crate::*;




#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserFan{
    pub id: i32,
    pub friends: Vec<FriendData>,
    pub invitation_requests: Vec<InvitationRequestData>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserFanData{
    pub id: i32,
    pub friends: Vec<FriendData>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FriendData{
    pub screen_cid: String,
    pub added_at: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InvitationRequestData{
    pub from_screen_cid: String,
    pub requested_at: i64,
    pub gallery_id: i32,
    pub is_accepted: bool
}

impl UserFan{

    pub async fn add_user_to_friend(screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // ...

        Ok(())

    }

    pub async fn remove_user_to_friend(screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // ...

        Ok(())

    }

    pub async fn push_invitation_request_for(screen_cid: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) 
            -> Result<(), PanelHttpResponse>{
        
        // check that there is a user with this screen_cid
        // ...

        Ok(())

    }

}