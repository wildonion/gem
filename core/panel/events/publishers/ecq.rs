



use crate::*;


// god dual event
// admins or gods can collaborate with others and share their events using redis based ecq 
// fire, emit or publish a redis event or topic like ecq event for a specific event id like `ecq-{event_id}`


#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CollaborationQueue{
    pub event_id: String,
    pub god_id: String,
}

impl CollaborationQueue{

    fn generate_event_time_hash<'t>(&self) -> [u8; 32]{
        
        let keccak256 = keccak256(self.event_id.as_bytes());
        keccak256

    }

}

// fire/emit/publish UserNotif events through the ws channels
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserNotif{
    user_id: String,
    notifs: Vec<NotifData>,
    updated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct NotifData{
    fired_at: Option<i64>,
    seen: bool,
    topic: String, // json string contains the actual data like fireing the player status (role and state changing) during the game 
}

impl UserNotif{
    fn set(&mut self, notif_data: NotifData) -> Self{
        self.notifs.push(notif_data);
        let user_notif = UserNotif { user_id: self.user_id.clone(), notifs: self.notifs.clone(), updated_at: self.updated_at };
        UserNotif{
            ..user_notif /* filling all the fields with the user_notif ones */
        }
    }
    fn get(&mut self) -> Self{
        let this = UserNotif { ..Default::default() };
        this
    }
}

pub trait NotifExt{
    type Data;
    fn set_user_notif(&mut self, notif_data: NotifData) -> Self;
    fn get_user_notif(&self) -> Vec<NotifData>;
}

impl NotifExt for UserNotif{
    type Data = Self;

    fn get_user_notif(&self) -> Vec<NotifData> {
        self.notifs.clone()
    }

    fn set_user_notif(&mut self, new_notif: NotifData) -> Self { // since the set() method of the UserNotif instance is mutable this method must be mutable too
        self.set(new_notif)
    }

}