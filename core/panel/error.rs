



use crate::*;
use std::io::Write;
use tokio::fs::OpenOptions;


#[derive(Debug)]
pub struct PanelError{
    pub code: u16,
    pub msg: Vec<u8>, // reason 
    pub kind: ErrorKind // due to what service 
}


#[derive(Debug)]
pub enum StorageError{
    Redis(redis::RedisError),
    RedisAsync(redis_async::error::Error),
    Diesel(diesel::result::Error)
}
#[derive(Debug)]
pub enum ServerError{
    ActixWeb(std::io::Error),
    Ws(ws::ProtocolError),
}
#[derive(Debug)]
pub enum ErrorKind{
    Server(ServerError), // actix server io 
    Storage(StorageError), // diesel, redis
}

/* make it senable to be shared between threads */
unsafe impl Send for PanelError{}
unsafe impl Sync for PanelError{}

/* can be made using from() method */
impl From<std::io::Error> for ErrorKind{
    fn from(error: std::io::Error) -> Self {
        ErrorKind::Server(ServerError::ActixWeb(error))
    }
}

impl From<ws::ProtocolError> for ErrorKind{
    fn from(error: ws::ProtocolError) -> Self {
        ErrorKind::Server(ServerError::Ws(error))
    }
}

impl From<redis::RedisError> for ErrorKind{
    fn from(error: redis::RedisError) -> Self{
        ErrorKind::Storage(StorageError::Redis(error))
    }
}

impl From<redis_async::error::Error> for ErrorKind{
    fn from(error: redis_async::error::Error) -> Self{
        ErrorKind::Storage(StorageError::RedisAsync(error))
    }
}

impl From<diesel::result::Error> for ErrorKind{
    fn from(error: diesel::result::Error) -> Self{
        ErrorKind::Storage(StorageError::Diesel(error))
    }
}

impl From<(Vec<u8>, u16, ErrorKind)> for PanelError{
    fn from(msg_code_kind: (Vec<u8>, u16, ErrorKind)) -> PanelError{
        PanelError { code: msg_code_kind.1, msg: msg_code_kind.0, kind: msg_code_kind.2 }
    }
}


impl PanelError{

    pub fn new(code: u16, msg: Vec<u8>, kind: ErrorKind) -> Self{
        
        let err = PanelError::from((msg, code, kind));

        err
    }

    pub async fn write(&self) -> impl Write{ /* the return type is a trait which will be implemented for every type that satisfied the Write trait */
        
        let this = self;
        let Self { code, msg, kind } = this;

        let msg_content = String::from_utf8(msg.to_owned());
        let error_log_content = format!("code: {} | message: {} | due to: {:?} | time: {}\n", code, &msg_content.unwrap(), kind, chrono::Local::now().timestamp_millis());

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
                let filepath = format!("logs/error-kind/{}-panel-error-custom-error-handler-log-file.log", log_name);
                let mut error_kind_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                error_kind_log.write_all(e.to_string().as_bytes()).await.unwrap();
            }
        }

        buffer /* returns the full filled buffer from the error  */
    
    }

    pub fn write_sync(&self) -> impl Write{ /* the return type is a trait which will be implemented for every type that satisfied the Write trait */
        
        let this = self;
        let Self { code, msg, kind } = this;

        let msg_content = serde_json::from_slice::<String>(msg.as_slice());
        let error_log_content = format!("code: {} | message: {} | due to: {:?} | time: {}\n", code, &msg_content.unwrap(), kind, chrono::Local::now().timestamp_millis());

        /* writing to buffer */
        let mut buffer = Vec::new(); 
        let _: () = write!(&mut buffer, "{}", error_log_content).unwrap(); /* writing to buffer */

        /* writing to file */
        let filepath = format!("logs/error-kind/panel-error.log");
        let mut panel_error_log;

        match std::fs::metadata("logs/error-kind/panel-error.log"){
            Ok(_) => {
                /* ------- we found the file, append to it ------- */
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(filepath.as_str())
                    .unwrap();
                file.write_all(error_log_content.as_bytes()).unwrap(); // Write the data to the file
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                /* ------- we didn't found the file, create a new one ------- */
                panel_error_log = std::fs::File::create(filepath.as_str()).unwrap();
                panel_error_log.write_all(error_log_content.as_bytes()).unwrap();
            },
            Err(e) => {
                /* ------- can't create a new file or append to it ------- */
                let log_name = format!("[{}]", chrono::Local::now());
                let filepath = format!("logs/error-kind/{}-panel-error-custom-error-handler-log-file.log", log_name);
                let mut error_kind_log = std::fs::File::create(filepath.as_str()).unwrap();
                error_kind_log.write_all(e.to_string().as_bytes()).unwrap();
            }
        }

        buffer /* returns the full filled buffer from the error  */
    
    }

}