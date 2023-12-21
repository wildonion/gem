



use crate::*;


// god dual event
// admins or gods can collaborate with others and share their events using redis based ecq 
// fire, emit or publish a redis event or topic like ecq event for a specific event id like `ecq-{event_id}`


#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CollaborationQueue{
    pub event_id: String,
    pub user_id: String,
}

impl CollaborationQueue{

}