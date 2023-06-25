



use crate::*;



// todo - impl From for PanelError

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanelError<'m>{
    pub code: i32,
    pub msg: &'m str, // reason 
    pub kind: ErrorKind // service
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ErrorKind{
    Server, // actix
    Storage, // diesel, redis
}

unsafe impl Send for PanelError<'_>{}
unsafe impl Sync for PanelError<'_>{}


impl<'m> PanelError<'m>{

    fn new(code: i32, msg: &'m str, kind: ErrorKind) -> Self{
        
        Self { code, msg, kind }
    }
}