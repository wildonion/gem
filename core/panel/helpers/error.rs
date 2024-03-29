


use crate::*;
use crate::constants::LOGS_FOLDER_ERROR_KIND;
use std::error::Error;
use std::io::{Write, Read};
use serenity::model::misc;
use tokio::fs::OpenOptions;
use tokio::io::ReadBuf;



/* ----------------------------- */
/* ----------------------------- 
    a custom error handler for the panel apis
    can be built during the execution of the app
    to log the runtime crashes and errors
*/
#[derive(Debug)]
pub struct PanelError{
    pub code: u16,
    pub msg: Vec<u8>, // reason 
    pub kind: ErrorKind, // due to what service 
    pub method_name: String // in what method
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
pub enum ThirdPartyApiError{
    ReqwestTextResponse(String),
}
#[derive(Debug)]
pub enum ErrorKind{
    Server(ServerError), // actix server io 
    Storage(StorageError), // diesel, redis
    ThirdPartyApi(ThirdPartyApiError) // reqwest response text
}

/* ----------------------------- */
/* -----------------------------
    make it sendable to be shared between threads also note that 
    Send and Sync can only be implement for a type that is inside 
    the current crate thus can't be implemented for actix_web::HttpResponse
*/
unsafe impl Send for PanelError{}
unsafe impl Sync for PanelError{}


impl From<std::io::Error> for ErrorKind{ // std::io::Error can also be caused by file read/write process
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

impl From<String> for ErrorKind{
    fn from(error: String) -> Self {
        ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(error))
    }
}

impl From<(Vec<u8>, u16, ErrorKind, String)> for PanelError{
    fn from(msg_code_kind_method: (Vec<u8>, u16, ErrorKind, String)) -> PanelError{
        PanelError { code: msg_code_kind_method.1, msg: msg_code_kind_method.0, kind: msg_code_kind_method.2, method_name: msg_code_kind_method.3 }
    }
}


impl PanelError{

    pub fn new(code: u16, msg: Vec<u8>, kind: ErrorKind, method_name: &str) -> Self{
        
        let err = PanelError::from((msg, code, kind, method_name.to_string()));

        err
    }

    pub async fn write(&self) -> impl Write{ /* the return type is a trait which will be implemented for every type that has satisfied the Write trait */
        
        let this = self;
        let Self { code, msg, kind, method_name } = this;

        let e = match self{
            PanelError{
                code: _,
                msg,
                kind,
                method_name,
            } if code <= &400 => {},
            PanelError{
                code: _,
                msg,
                ..
            } => {},
            _ => ()
        };

        /* creating the logs/error-kind folder if it doesn't exist */
        tokio::fs::create_dir_all(LOGS_FOLDER_ERROR_KIND).await.unwrap();
        let filepath = format!("{}/panel-error.log", LOGS_FOLDER_ERROR_KIND);

        let mut panel_error_log;
        let msg_content = String::from_utf8(msg.to_owned());
        let error_log_content = format!("code: {} | message: {} | due to: {:?} | time: {} | method name: {}\n", code, &msg_content.unwrap(), kind, chrono::Local::now().timestamp_millis(), method_name);
        
        /* writing to file */
        match tokio::fs::metadata(filepath.clone()).await{
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
                panel_error_log = tokio::fs::File::create(filepath.clone().as_str()).await.unwrap();
                panel_error_log.write_all(error_log_content.as_bytes()).await.unwrap();
            },
            Err(e) => {
                /* ------- can't create a new file or append to it ------- */
                let log_name = format!("[{}]", chrono::Local::now());
                let filepath = format!("{}/{}-panel-error-custom-error-handler-log-file.log", log_name, LOGS_FOLDER_ERROR_KIND);
                let mut error_kind_log = tokio::fs::File::create(filepath.as_str()).await.unwrap();
                error_kind_log.write_all(e.to_string().as_bytes()).await.unwrap();
            }
        }

        /* writing to buffer using write macro */
        let mut buffer = Vec::new(); 
        let _: () = write!(&mut buffer, "{}", error_log_content).unwrap(); /* writing to buffer using write macro */
        
        /* OR */
        // serde_json::to_writer_pretty(buffer, &error_log_content);

        buffer /* returns the full filled buffer from the error  */
    
    }

    pub fn write_sync(&self) -> impl Write{ /* the return type is a trait which will be implemented for every type that is satisfied the Write trait */
        
        let this = self;
        let Self { code, msg, kind, method_name } = this;

        /* creating the logs/error-kind folder if it doesn't exist */
        std::fs::create_dir_all(LOGS_FOLDER_ERROR_KIND).unwrap();
        let filepath = format!("{}/panel-error.log", LOGS_FOLDER_ERROR_KIND);

        let mut panel_error_log;
        let msg_content = serde_json::from_slice::<String>(msg.as_slice());
        let error_log_content = format!("code: {} | message: {} | due to: {:?} | time: {} | method name: {}\n", code, &msg_content.unwrap(), kind, chrono::Local::now().timestamp_millis(), method_name);

        /* --------------------------------------------------------------------------------- */
        /* -------------- read from file buffer and decode it into the String -------------- */
        /* --------------------------------------------------------------------------------- */
        let loaded_file = std::fs::OpenOptions::new()
            .read(true)
            .open(filepath.clone())
            .unwrap();
        
        /* reading the full filled bytes of the file and put it into a buffer reader */
        let buf_reader = std::io::BufReader::new(loaded_file);

        /* OR 

        let mut file_content_buffer = vec![];
        loop{
            let bytes_read = loaded_file.read(&mut file_content_buffer).unwrap();
            /* 
                if the zero bytes are in there means we've 
                read all the bytes and filled the buffer with 
                the file bytes
            */
            if bytes_read == 0{ // means there is nothing has been written into the buffer
                break;
            }
        }

        */

        /* decoding the buffer reader into the String struct */
        let decoded_error_log_content: String = serde_json::from_reader(buf_reader).unwrap();
        /* --------------------------------------------------------------------------------- */
        /* --------------------------------------------------------------------------------- */
        /* --------------------------------------------------------------------------------- */

        /* writing to file */
        match std::fs::metadata(filepath.clone()){
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
                panel_error_log = std::fs::File::create(filepath.clone().as_str()).unwrap();
                panel_error_log.write_all(error_log_content.as_bytes()).unwrap();
            },
            Err(e) => {
                /* ------- can't create a new file or append to it ------- */
                let log_name = format!("[{}]", chrono::Local::now());
                let filepath = format!("{}/{}-panel-error-custom-error-handler-log-file.log", log_name, LOGS_FOLDER_ERROR_KIND);
                let mut error_kind_log = std::fs::File::create(filepath.as_str()).unwrap();
                error_kind_log.write_all(e.to_string().as_bytes()).unwrap();
            }
        }

        /* writing to buffer using write macro */
        let mut buffer = Vec::new(); 
        let _: () = write!(&mut buffer, "{}", error_log_content).unwrap(); /* writing to buffer using write macro */
        
        /* OR */
        // serde_json::to_writer_pretty(buffer, &error_log_content);
        
        buffer /* returns the full filled buffer from the error  */
    
    }

}