



use crate::*;
use std::io::Write;


#[derive(Debug)]
pub struct PanelError{
    pub code: i32,
    pub msg: [u8; 32], // reason 
    pub kind: ErrorKind // service
}

#[derive(Debug)]
pub enum ErrorKind{
    Server, // actix
    Storage(std::io::Error), // diesel, redis
}

unsafe impl Send for PanelError{}
unsafe impl Sync for PanelError{}

impl From<std::io::Error> for ErrorKind{
    fn from(error: std::io::Error) -> Self {
        ErrorKind::Storage(error)
    }
}


impl From<([u8; 32], i32, ErrorKind)> for PanelError{
    fn from(msg_code_kind: ([u8; 32], i32, ErrorKind)) -> PanelError{
        /* 
            can't return a borrow from the function since it's a borrow 
            to a type that by executing the function the type will be dropped 
            out of the function scope and from the ram 
        */
        // let msg = msg_code_kind.0.as_str();
        PanelError { code: msg_code_kind.1, msg: msg_code_kind.0, kind: msg_code_kind.2 }
    }
}


impl PanelError{

    fn new(code: i32, msg: [u8; 32], kind: ErrorKind) -> Self{
        
        let err = PanelError::from((msg, code, kind));

        err
    }

    fn write(&self) -> impl Write{ /* the return type is a trait which will be implemented for every type that satisfied the Write trait */

        let Self { code, msg, kind } = self;

        /* 
            passing a mutable reference to buffer to write! macro so  
            the buffer can be mutated outside of the write! scope
            also Vec types implemented the Write trait already 
            we just need to use it in here
        */
        let mut buffer = Vec::new(); 
        let msg_content = borsh::try_from_slice_with_schema::<String>(msg.as_slice());
        let message = format!("error occurred at {} due to {} of kind {:?}", chrono::Local::now(), msg_content.unwrap(), kind);
        let mut writer = write!(&mut buffer, "{}", message); /* writing to buffer */
        buffer
    
    }

}