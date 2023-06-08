



use crate::*;


// every instance of PanelError must have to_string() method
// custom error struct (lifetime, slice types, generic, return pointer (Box<dyn Trait> or &dyn Trait and -> impl Trait from its methods) 

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanelError<'m>{
    pub code: i32,
    pub msg: &'m str,
    pub kind: ErrorKind
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ErrorKind{
    Server, // actix
    Storage, // diesel
}
