


use crate::*;


// create private romm
// create public room


#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Gallery{
    pub id: i32,
    pub collections: Vec<i32>,
    pub is_private: bool
}