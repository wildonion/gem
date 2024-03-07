


/* 
   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
        CONSE PANEL CUSTOM ERROR HELPER
   -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
   https://fettblog.eu/rust-enums-wrapping-errors/
   https://betterprogramming.pub/a-simple-guide-to-using-thiserror-crate-in-rust-eee6e442409b
   
   custom error handler, useful to specify the exact type of error at runtime instead of using 
   Box<dyn Error> which handles all possible errors at runtime dynamically and may causes the 
   app gets panicked at runtime we would have to use the one in here note that we should the ? 
   operator in any function that returns Result<T, E> or Option<T>, basically a custom error type 
   E which must be enum variant since Error is not impelemted for normal Rust types due to the
   fact that the Error trait and for example String type are both in different crates and based
   on orphant rule Rust doesn't allow us to impl Error for String, needs to have an implementation 
   of the trait std::error::Error in other for it to be compatible with Box<dyn Error> and use 
   it as the error part, type G can be a trait object T or be casted into trait T if it impls trait 
   T use Display to write the variant error message to the buffer and use Debug to write the exact 
   source of error to the buffer 
   
   returning trait as return type of method requires the returning instance impls the trait and 
   in our case since we don't know the return type that will cause the error we can't use -> impl 
   Error hence we should put the trait behind a box, (we stored on the heap to cover enough size 
   for the type causes the error since we don't know the implemenor size which helps us to avoid 
   overflowing) by doing so we tell rust that the type causes the error will be specified at runtime 
   and all we know is it impls Error trait so if that error happens map the type into the error 
   and return it as the error part of the result, to catch the error we are able to use the ? operator 
   to convert different types of errors coming from different methods into the same Boxed Error 
   type. if we would specify the exact type of error in error part of the result we must create an 
   instance of the error and return that if we don't want to use ?, another way is to map the error 
   of each function into the error type as the conclusion we must know the exact type of the error 
   instead of using Box<dyn Error> to make the type caused the error as the return type by calling 
   the map_err() method on the method to map the error into the exact error type as well as use ? 
   operator to unwrap the exact error we should impl Error, Debug and Display traits for the error 
   type which what thiserror is currently doing. usually we use an enum for the error type to cover 
   all possible runtime errors then we could pass the variant the causes the error into the map_err() 
   method and finally use the ? operator to return the exact error from the function with Result<(), ErrorKind> 
   as its return type also in order to create Box<dyn Error> from Box<T> the T must implements the 
   Error trait already since in order to return a trait object from a method we need to return an 
   instance of the struct which impls that trait already.

   fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{} if we want to use 
   Result<(), impl std::error::Error + Send + Sync + 'static> as the return type of the error part, 
   the exact error type instance must be sepecified also the Error trait must be implemented for the 
   error type (impl Error for ErrorType{}) since we're implementing the Error trait for the error type 
   in return type which insists that the instance of the type implements the Error trait. by returning 
   a boxed error trait we're returning the Error trait as a heap object behind a valid pointer which 
   handles all error type at runtime, this is the solution to return traits as an object cause we don't 
   know what type causes the error at runtiem and is the implementor of the Error trait which forces 
   us to return the trait as the error itself and since traits are dynamically sized we can't treat 
   them as a typed object directly we must put them behind pointer like &'valid dyn Trait or box them 
   to send them on the heap, also by bounding the Error trait to Send + Sync + 'static we'll make it 
   sefable, sendable and shareable to move it between different scopes and threads.
*/


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

pub enum ErrorKind{
    Server(ServerError), // actix server io 
    Storage(StorageError), // diesel, redis
    ThirdPartyApi(ThirdPartyApiError) // reqwest response text
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
    Reqwest(reqwest::Error),
}

/* ----------------------------- */
/* -----------------------------
    make it sendable to be shared between threads also note that 
    Send and Sync can only be implement for a type that is inside 
    the current crate thus can't be implemented for actix_web::HttpResponse
*/
unsafe impl Send for PanelError{}
unsafe impl Sync for PanelError{}

/* ----------------------------- */
/* -----------------------------
    implementing Error, Display, Debug and From traits for ErrorKind enum so we can 
    return the Error trait as the return type of the error part from the method, to do 
    so we can return the instance of the ErrorKind in place of the return type of 
    the error part like so: Result<(), ErrorKind> and then use the ? operator to 
    map the type causes the error into the exact error variant.
    
    display a based error message for current variant then log the cause of error using 
    Debug which logs the current source value returned by the source() method.
*/
impl std::error::Error for ErrorKind{

    // the return type is a borrow with the self lifetime of trait 
    // object of type Error with static lifetime, trait objects must
    // be behind pointer in our case we're putting it behind the pointer
    // with the lifetime of the self, this is for identifying the 
    // underlying lower level error that caused your error.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>{
        match self{ // all the following sources must be an object trait of type Error
            Self::Server(ServerError::ActixWeb(s)) => Some(s),
            Self::Server(ServerError::Ws(s)) => Some(s),
            Self::Storage(StorageError::Diesel(s)) => Some(s),
            Self::Storage(StorageError::RedisAsync(s)) => Some(s),
            Self::Storage(StorageError::Redis(s)) => Some(s),
            Self::ThirdPartyApi(ThirdPartyApiError::Reqwest(s)) => Some(s), 
        }
    }

}

impl std::fmt::Display for ErrorKind{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        match self{ // write! macros writes data into the mutable buffer f by converting the string data into utf8 bytes
            Self::Server(ServerError::ActixWeb(_)) => f.write_str(&format!("[ACTIX WEB] - failed to start actix web server")),
            Self::Server(ServerError::Ws(_)) => f.write_str(&format!("[ACTIX WS] - failed to read from ws stream")),
            Self::Storage(StorageError::Diesel(_)) => f.write_str(&format!("[DIESEL] - failed to do postgres db operation")),
            Self::Storage(StorageError::RedisAsync(_)) => f.write_str(&format!("[REDIS ASYNC] - failed to subscribe to channel")),
            Self::Storage(StorageError::Redis(_)) => f.write_str(&format!("[REDIS] - failed to store in redis")),
            Self::ThirdPartyApi(ThirdPartyApiError::Reqwest(_)) => f.write_str(&format!("[REQWEST] - failed to send api request")),
        }
    }
}

impl std::fmt::Debug for ErrorKind{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?; // writing into the mutable buffer, we would have access it every where
        if let Some(source) = self.source(){
            writeln!(f, "Caused by: \n\t{}", source)?;
        }
        Ok(())
    }
}

// the error however can be made by calling from() method on the ErrorKind struct.
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

impl From<reqwest::Error> for ErrorKind{
    fn from(error: reqwest::Error) -> Self {
        ErrorKind::ThirdPartyApi(ThirdPartyApiError::Reqwest(error))
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