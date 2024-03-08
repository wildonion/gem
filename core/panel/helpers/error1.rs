


/* 
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
        CONSE PANEL CUSTOM ERROR HELPER WITHOUT USING THISERROR CRATE
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
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