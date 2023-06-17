






use crate::*;




#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Mention{
    pub text: String,
    pub created_at: Option<String>
}