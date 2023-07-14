


use crate::*;


#[derive(Clone, Debug, Serialize, Default, Deserialize)]
pub struct Reveal{
    pub roles: String,
    pub event_id: String,
}