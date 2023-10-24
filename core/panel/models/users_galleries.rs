


use crate::*;


// create private romm
// create public room
// all galleries contain pastel artworks 

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Gallery{
    pub id: i32,
    pub nfts: Vec<String>, // sql field: TEXT[] DEFAULT ARRAY[]::TEXT[]
    pub is_private: bool,
}