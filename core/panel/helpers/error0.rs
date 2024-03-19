


/* 
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
        CONSE PANEL CUSTOM ERROR HELPER USING THISERROR CRATE
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
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

   in the following example the error part of the result is a boxed version of Error trait
   means that using ? operator to unwrap the error on any result type inside the main function 
   requires the std::error::Error trait be implemented for the type caused the error in order 
   to build the error using from() method or the error part be an Error trait object, in our 
   case the error part of the open() method implements the Error trait and it's a trait
   object of type std::io::Error, on the other hand the boxed version of Error trait supports 
   showing error at runtime on every type that implements Error which allows us to use ? operator 
   on any result type to convert the error into the error part in result type using from() method 
   properly to return the instance of the error part to the caller and logs to the console.

    #[tokio::main]
    async fn main() -> 
        // it can be an on the heap Error trait object itself or a boxed struct instance which impls Error trait
        // in cases we don't have custom error handler we can use Boxed Error trait which detect the type of error
        // at runtime it only requires the error type happening at runtime implements the Error trait.
        Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{ 

        // ERROR to the console: Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }
        let file = std::fs::File::open("openme.txt")?;

    }

    final note: Box<dyn std::error::Error> is a boxed object safe trait which is used for dynamic dispatch at runtime 
                this would be called on any object that implements the Error trait to return the source of the error, 
                we've to make sure this is an object safe trait to dispatch the call dynamically cause the compiler must 
                not be aware of the implementor size in order to do the call at runtime.

*/

use crate::*;
use crate::constants::LOGS_FOLDER_ERROR_KIND;
use std::error::Error;
use std::io::{Write, Read};
use serenity::model::misc;
use tokio::fs::OpenOptions;
use tokio::io::ReadBuf;
use thiserror::Error;


/* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
thiserror impls Display (log the error variant into human readable format), Error and From traits for 
the type to display an error message for the variant causes the error in the source method then we could 
debug the source using our own Debug implementation also note that if we want to return the PanelErrorResponse 
as the error part of Result which allows us to use ? operator on the error type, the Error, Display, Debug 
and From traits must be implemented for that also the From trait must be implemented for every single error 
variant that makes the PanelErrorResponse like if we want to use ? to unwrap a file opening process the From<std::io::Error> 
must be implemented for the PanelErrorResponse struct.

? needs to create error from the type so the From trait must be implemented for the 
type to build the instance contains the caused error Rust uses the type passed to from() 
method to log and display the source of error into the console, also impl Debug for 
each error variant inside the enum to log the source to the console and Display for 
the error handler struct to log and instance of the error handler and finally the Error
trait for the error handler struct or the field that contains the enum error variants 
that wants to be used as the error part in Result type.

#[error(/* */)] defines the Display representation of the enum variant it is applied to, e.g., if the key file is missing, the error would return the string failed to read the key file when displaying the error in the terminal.
#[source] is used to denote what should be returned as the root cause in Error::source. Which is used in our debug implementation.
#[from] automatically derives an implementation of From for the type it has been applied to into the top-level error type (e.g., impl From<reqwest:Error> for CustomError {/* */}). The field annotated with #[from] is also used as an error source, saving us from having to use two annotations on the same field (e.g., #[source] #[from] reqwest::Error). Notice how we are unable put #[from] to the two std::io::Error variants, as there cannot be multiple From<std::io::Error> implementations for the same type.
-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
*/

#[derive(Error, Debug)]
pub struct PanelErrorResponse{
    pub code: u16,
    pub msg: Vec<u8>, // reason 
    pub kind: ErrorKind, // due to what service 
    pub method_name: String // in what method
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// since we've implemented the Error, Debug, Display and From traits 
// for each variant of ErrorKind enum, it can be used as a separate
// error handler in case of unwrapping the error using ? operator, so
// if we're using the ErrorKind as an error part in a result type and 
// unwrapping the error using ? operator the fulfilled buffer inside
// the Debug and Display traits methods will be logged to the console
// for the variant caused the error.
// NOTE => since From trait is not implemented for each variant directly
//         we can't unwrap the error on ErrorKind using ? operator cause 
//         in order to map the type into an error From must be implmented 
//         for an Error trait object like std::io::Error in opening a one 
//         non existent file, which none the following variants contains 
//         trait object directly, they're nested enum variants.
// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[derive(Error)]
pub enum ErrorKind{
    #[error("Large Number of Workers {}, Maximum Is 10", .0)]
    Workers(u16),
    #[error("File Read/Write Error")]
    File(FileEror),
    #[error("Actix HTTP or WS Server Error")]
    Server(ServerError), // actix server io 
    #[error("Redis or Diesel Error")]
    Storage(StorageError), // diesel, redis
    #[error("Api Reqwest Error")]
    ThirdPartyApi(ThirdPartyApiError) // reqwest response text
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// in the following enums, From is impelemented directly 
// for each variant to convert the type caused the error 
// into the error by calling from() method when we use ? 
// operator 
// NOTE => we get the message inside the #[error] if we use unwrap() or match over the result to cover the error part unless we call the source() method on the error variant to get the source message of the exact error
// NOTE => #[from] will be used to unwrap the error using ? so it contains the exact error message inside the source() method of std::error::Error trait
// NOTE => message inside the #[error] is used to log into the console using Display trait 
// NOTE => logging the cause or the source of error along with the message inside the #[error] macro which is written into the buffer using Display trait can be done with Debug trait
// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[derive(Error)]
pub enum FileEror{
    #[error("[FILE] - failed to read from or write to file")]
    ReadWrite(#[from] std::io::Error) 
}

#[derive(Error)]
pub enum StorageError{
    #[error("[REDIS] - failed to store in redis")]
    Redis(#[from] redis::RedisError),
    #[error("[REDIS ASYNC] - failed to subscribe to channel")]
    RedisAsync(#[from] redis_async::error::Error), 
    #[error("[DIESEL] - failed to do postgres db operation")]
    Diesel(#[from] diesel::result::Error) 
}
#[derive(Error)]
pub enum ServerError{
    #[error("[ACTIX WEB] - failed to start actix web server")]
    ActixWeb(#[from] std::io::Error),
    #[error("[ACTIX WS] - failed to read from ws stream")]
    Ws(#[from] ws::ProtocolError), 
}
#[derive(Error)]
pub enum ThirdPartyApiError{
    #[error("[REQWEST] - failed to send api request")]
    Reqwest(#[from] reqwest::Error), 
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// thiserrror's only requirement is for the type to implement the Debug trait
// here we're implementing the Debug trait manually to write the error source
// into the formatter buffer so we can see the logs in the terminal
// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl std::fmt::Debug for ErrorKind{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?; // writing the self into the mutable buffer
        if let Some(source) = self.source(){
            writeln!(f, "Caused by: \n\t{}", source)?; // writing the source of the error into the mutable buffer
        }
        Ok(())
    }
}

impl std::fmt::Debug for FileEror{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?; // writing the self into the mutable buffer
        if let Some(source) = self.source(){
            writeln!(f, "Caused by: \n\t{}", source)?; // writing the source of the error into the mutable buffer
        }
        Ok(())
    }
}

impl std::fmt::Debug for StorageError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?; // writing the self into the mutable buffer
        if let Some(source) = self.source(){
            writeln!(f, "Caused by: \n\t{}", source)?; // writing the source of the error into the mutable buffer
        }
        Ok(())
    }
}

impl std::fmt::Debug for ServerError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?; // writing the self into the mutable buffer
        if let Some(source) = self.source(){
            writeln!(f, "Caused by: \n\t{}", source)?; // writing the source of the error into the mutable buffer
        }
        Ok(())
    }
}

impl std::fmt::Debug for ThirdPartyApiError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?; // writing the self into the mutable buffer
        if let Some(source) = self.source(){
            writeln!(f, "Caused by: \n\t{}", source)?; // writing the source of the error into the mutable buffer
        }
        Ok(())
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/*
    make it sendable to be shared between threads also note that 
    Send and Sync can only be implement for a type that is inside 
    the current crate thus can't be implemented for actix_web::HttpResponse
*/
// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
unsafe impl Send for PanelErrorResponse{}
unsafe impl Sync for PanelErrorResponse{}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
/* 
    implementing an actix error responder for the PanelErrorResponse struct, 
    allows us to use PanelErrorResponse as the error part of the http response 
    result instead of actix_web::Error to avoid unknown runtime actix
    crashes
*/
// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
impl actix_web::ResponseError for PanelErrorResponse{
    
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
    // when we use ? operator on the result type to unwrap the error Rust
    // get started looking for the From implementation for the type that
    // caused the error like if we're using ? to unwrap the error on a file
    // reading process there must be From<std::io::Error> implementation for
    // the PanelErrorResponse with some error message, since it allows Rust to log 
    // the error to the console, in the following we're creating a response 
    // object from the error detected by ? to send it back to the client, 
    // note that in the place of the message we've used the error message
    // inside the From implementation, also since we're handling possible errors
    // using PanelErrorResponse there is no need to match over ok or the err part
    // of any result, we can directly use ? operator Rust will take care of 
    // the rest process and then if there is an error an http response containing 
    // the error will be returned back to the client.
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody>{ // the error response contains a boxed body bytes
        HttpResponse::build(self.status_code()).json(
            helpers::misc::Response::<'_, &[u8]>{
                data: Some(&[]),
                message: {
                    // converting the error bytes caused by one of the ErrorKind variant back to the string
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
            ErrorKind::Workers(s) => StatusCode::NOT_ACCEPTABLE,
            ErrorKind::File(s) => StatusCode::EXPECTATION_FAILED,
        }
    }

}
impl std::fmt::Display for PanelErrorResponse{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // write the PanelErrorResponse instance into the buffer
    }
}

/*  -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    From implementations to make the error from the a source error like std::io::Error, when used to return 
    an instance of PanelErrorResponse which contains an error variant, it mainly allows us to use ? operator 
    to convert the type into instance of PanelErrorResponse by calling the from method to return the error 
    caused by an unsuccessful related operations like when we're using ? operator on opening a file result, 
    if the operation goes wrong like the file doesn't get found it eventually build a PanelErrorResponse 
    instance which contains the io error by calling from() method then the code gets panicked in there which 
    causes to return an instance of PanelErrorResponse to the caller, albeit to log the error the Dispaly and 
    Debug traits must be implemented for the PanelErrorResponse. basically to return type E as error part in
    Result in order to be able to use ? operator on the process contains a result, the From trait must be 
    implemented for each error variant (that we've detected might happened at runtime) of type E 
    NOTE => in the following methods, error param is the exact source of the error in which the app gets
            crashed at runtime due to 
  -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
*/
impl From<std::io::Error> for PanelErrorResponse{
    fn from(error: std::io::Error) -> Self {
        Self{ 
            code: 0, 
            msg: error.to_string().as_bytes().to_vec(), // this is the exact source of error and is being used to build an http response with message so we need to have an error string
            kind: ErrorKind::File(FileEror::ReadWrite(error)), 
            method_name: String::from("") 
        }
    }
}

impl From<ws::ProtocolError> for PanelErrorResponse{
    fn from(error: ws::ProtocolError) -> Self {
        Self{ 
            code: 0, 
            msg: error.to_string().as_bytes().to_vec(), // this is the exact source of error and is being used to build an http response with message so we need to have an error string
            kind: ErrorKind::Server(ServerError::Ws(error)), 
            method_name: String::from("") 
        }
    }
}

impl From<redis::RedisError> for PanelErrorResponse{
    fn from(error: redis::RedisError) -> Self {
        Self{ 
            code: 0, 
            msg: error.to_string().as_bytes().to_vec(), // this is the exact source of error and is being used to build an http response with message so we need to have an error string
            kind: ErrorKind::Storage(StorageError::Redis(error)),
            method_name: String::from("") 
        }
    }
}

impl From<redis_async::error::Error> for PanelErrorResponse{
    fn from(error: redis_async::error::Error) -> Self {
        Self{ 
            code: 0, 
            msg: error.to_string().as_bytes().to_vec(), // this is the exact source of error and is being used to build an http response with message so we need to have an error string
            kind: ErrorKind::Storage(StorageError::RedisAsync(error)),
            method_name: String::from("") 
        }
    }
}

impl From<diesel::result::Error> for PanelErrorResponse{
    fn from(error: diesel::result::Error) -> Self {
        Self{ 
            code: 0, 
            msg: error.to_string().as_bytes().to_vec(), // this is the exact source of error and is being used to build an http response with message so we need to have an error string
            kind: ErrorKind::Storage(StorageError::Diesel(error)),
            method_name: String::from("") 
        }
    }
}

impl From<(Vec<u8>, u16, ErrorKind, String)> for PanelErrorResponse{
    fn from(msg_code_kind_method: (Vec<u8>, u16, ErrorKind, String)) -> PanelErrorResponse{
        PanelErrorResponse { code: msg_code_kind_method.1, msg: msg_code_kind_method.0, kind: msg_code_kind_method.2, method_name: msg_code_kind_method.3 }
    }
}


impl PanelErrorResponse{

    pub fn new(code: u16, msg: Vec<u8>, kind: ErrorKind, method_name: &str) -> Self{
        
        let err = PanelErrorResponse::from((msg, code, kind, method_name.to_string()));

        err
    }
}