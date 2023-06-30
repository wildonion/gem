



use crate::*;
use std::io::Write;
use tokio::fs::OpenOptions;


#[derive(Debug)]
pub struct PanelError{
    pub code: u16,
    pub msg: [u8; 32], // reason 
    pub kind: ErrorKind // service
}


#[derive(Debug)]
pub enum StorageError{
    Redis(redis::RedisError),
    Diesel(diesel::result::Error)
}
#[derive(Debug)]
pub enum ErrorKind{
    Server(std::io::Error), // actix io 
    Storage(StorageError), // diesel, redis
}

unsafe impl Send for PanelError{}
unsafe impl Sync for PanelError{}

impl From<std::io::Error> for ErrorKind{
    fn from(error: std::io::Error) -> Self {
        ErrorKind::Server(error)
    }
}

impl From<redis::RedisError> for ErrorKind{
    fn from(error: redis::RedisError) -> Self{
        ErrorKind::Storage(StorageError::Redis(error))
    }
}

impl From<diesel::result::Error> for ErrorKind{
    fn from(error: diesel::result::Error) -> Self{
        ErrorKind::Storage(StorageError::Diesel(error))
    }
}

impl From<([u8; 32], u16, ErrorKind)> for PanelError{
    fn from(msg_code_kind: ([u8; 32], u16, ErrorKind)) -> PanelError{
        /* 
            can't return a borrow from the function since it's a borrow 
            to a type that by executing the function the type will be dropped 
            out of the function scope and from the ram, thus i decided to 
            have a fixed size of message in a form of array slices contains
            32 bytes of utf8 elements
            
            let msg = msg_code_kind.0.as_str();
        */
        PanelError { code: msg_code_kind.1, msg: msg_code_kind.0, kind: msg_code_kind.2 }
    }
}


impl PanelError{

    pub fn new(code: u16, msg: [u8; 32], kind: ErrorKind) -> Self{
        
        let err = PanelError::from((msg, code, kind));

        err
    }

    pub async fn write(&self) -> impl Write{ /* the return type is a trait which will be implemented for every type that satisfied the Write trait */
        
        let this = self;
        let Self { code, msg, kind } = this;

        /* 
            passing a mutable reference to buffer to write! macro so  
            the buffer can be mutated outside of the write! scope
            also Vec types implemented the Write trait already 
            we just need to use it in here
        */
        let msg_content = borsh::try_from_slice_with_schema::<String>(msg.as_slice());
        let error_log_content = format!("code: {} | message: {} | caused by: {:?} | time: {}", code, &msg_content.unwrap(), kind, chrono::Local::now().timestamp_millis());

        /* writing to buffer */
        let mut buffer = Vec::new(); 
        let _: () = write!(&mut buffer, "{}", error_log_content).unwrap(); /* writing to buffer */

        /* writing to file */
        let filepath = format!("logs/error-kind/panel-error.log");
        let mut panel_error_log;

        match tokio::fs::metadata("logs/error-kind/panel-error.log").await{
            Ok(_) => {
                /* ------- we found the file, append to it ------- */
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(filepath.as_str())
                    .await.unwrap();
                file.write_all(error_log_content.as_bytes()).await.unwrap(); // Write the data to the file
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                /* ------- we didn't found the file, create a new one ------- */
                panel_error_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                panel_error_log.write_all(error_log_content.as_bytes()).await.unwrap();
            },
            Err(e) => {
                /* ------- can't create a new file or append to it ------- */
                let log_name = format!("[{}]", chrono::Local::now());
                let filepath = format!("logs/error-kind/{}-panel-error-creating-log-file.log", log_name);
                let mut error_kind_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                error_kind_log.write_all(e.to_string().as_bytes()).await.unwrap();
            }
        }

        buffer /* returns the full filled buffer */
    
    }

}