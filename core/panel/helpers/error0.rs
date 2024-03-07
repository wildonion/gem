


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
   it as the error part
   
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
use thiserror::Error;



/* 
    thiserror impls Display, Error and From traits for the type to display an error message for each variant 
    causes the error in the source method then we could debug the source using our own Debug implementation

    #[error(/* */)] defines the Display representation of the enum variant it is applied to, e.g., if the key file is missing, the error would return the string failed to read the key file when displaying the error.
    #[source] is used to denote what should be returned as the root cause in Error::source. Which is used in our debug implementation.
    #[from] automatically derives an implementation of From for the type it has been applied to into the top-level error type (e.g., impl From<reqwest:Error> for CustomError {/* */}). The field annotated with #[from] is also used as an error source, saving us from having to use two annotations on the same field (e.g., #[source] #[from] reqwest::Error). Notice how we are unable put #[from] to the two std::io::Error variants, as there cannot be multiple From<std::io::Error> implementations for the same type.
*/

#[derive(Error, Debug)]
pub struct PanelError{
    pub code: u16,
    pub msg: Vec<u8>, // reason 
    pub kind: ErrorKind, // due to what service 
    pub method_name: String // in what method
}

#[derive(Error)]
pub enum ErrorKind{
    #[error("Large Number of Workers, Must be {}", .0)]
    Workers(u16),
    #[error("Actix HTTP or WS Server Error")]
    Server(ServerError), // actix server io 
    #[error("Redis or Diesel Error")]
    Storage(StorageError), // diesel, redis
    #[error("Api Reqwest Error")]
    ThirdPartyApi(ThirdPartyApiError) // reqwest response text
}

#[derive(Error, Debug)]
pub enum StorageError{
    #[error("[REDIS] - failed to store in redis")]
    Redis(#[from] redis::RedisError),
    #[error("[REDIS ASYNC] - failed to subscribe to channel")]
    RedisAsync(#[from] redis_async::error::Error),
    #[error("[DIESEL] - failed to do postgres db operation")]
    Diesel(#[from] diesel::result::Error)
}
#[derive(Error, Debug)]
pub enum ServerError{
    #[error("[ACTIX WEB] - failed to start actix web server")]
    ActixWeb(#[from] std::io::Error),
    #[error("[ACTIX WS] - failed to read from ws stream")]
    Ws(#[from] ws::ProtocolError),
}
#[derive(Error, Debug)]
pub enum ThirdPartyApiError{
    #[error("[REQWEST] - failed to send api request")]
    Reqwest(#[from] reqwest::Error),
}

// thiserrror's only requirement is for the type to implement the Debug trait
// here we're implementing the Debug trait manually to write the error source
// into the formatter buffer
impl std::fmt::Debug for ErrorKind{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?; // writing into the mutable buffer, we would have access it every where
        if let Some(source) = self.source(){
            writeln!(f, "Caused by: \n\t{}", source)?;
        }
        Ok(())
    }
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
    implementing an actix error responder for the PanelError struct, 
    allows us to use PanelError as the error part of the http response 
    result instead of actix_web::Error to avoid unknown runtime actix
    crashes
*/
impl actix_web::ResponseError for PanelError{
    
    // actix will detect the type that causes the error at runtime
    // then choose its related variant and then show its message to the client 
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody>{ // the error response contains a boxed body bytes
        HttpResponse::build(self.status_code()).json(
            helpers::misc::Response::<'_, &[u8]>{
                data: Some(&[]),
                message: {
                    let string_err = std::str::from_utf8(&self.msg).unwrap();
                    &string_err
                },
                status: self.status_code().as_u16(),
                is_error: true,
            }
        )
    }

    fn status_code(&self) -> StatusCode{
        match &self.kind{
            ErrorKind::Server(ServerError::ActixWeb(s)) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::Server(ServerError::Ws(s)) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::Storage(StorageError::Diesel(s)) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::Storage(StorageError::RedisAsync(s)) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::Storage(StorageError::Redis(s)) => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::ThirdPartyApi(ThirdPartyApiError::Reqwest(s)) => StatusCode::EXPECTATION_FAILED,
        }
    }

}
impl std::fmt::Display for PanelError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // write the PanelError instance into the buffer
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