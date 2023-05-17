


use crate::*;
use std::borrow::Borrow;
use std::io::Write;
use std::sync::{Arc, mpsc::channel as heavy_mpsc, mpsc}; use std::time::{SystemTime, UNIX_EPOCH}; // NOTE - mpsc means multiple thread can access the Arc<Mutex<T>> (use Arc::new(&Arc<Mutex<T>>) to clone the arced and mutexed T which T can also be Receiver<T>) but only one of them can mutate the T out of the Arc by locking on the Mutex
use std::{env, thread, fs}; 
use async_std::path::PathBuf;
use chrono::Utc;
use futures::TryStreamExt;
use futures::{executor::block_on, future::{BoxFuture, FutureExt}}; // NOTE - block_on() function will block the current thread to solve the task
use log::info;
use mongodb::Client;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{self, doc};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use rand::prelude::*;
use routerify::Error;
use crate::{constants::*, schemas};
use serde::{Serialize, Deserialize};
use borsh::{BorshDeserialize, BorshSerialize};
use routerify_multipart::Multipart;
use hyper::{Client as HyperClient, Response, Body, Uri, Server, server::conn::AddrIncoming};
use async_trait::async_trait;
use std::net::SocketAddr;
use routerify::{RouterService, Router};
use std::{pin::Pin, sync::Mutex};
use crate::{constants::*, misc};
use futures::Future;
use uuid::Uuid;
use tokio::sync::oneshot::Receiver;
use log::error;




//// --------------------------------------------------------------------------
//// ------------------------------- app module -------------------------------
//// --------------------------------------------------------------------------
pub mod app{

    use super::*;

    pub const APP_NAME: &str = "Conse";
    //// future objects must be Send and static and types that must be shared between threads must be send sync and static 
    //// Box<dyn Future<Output=Result<u8, 8u>> + Send + Sync + 'static> means this future can be shared acorss threads and .awaits safely
    type Callback = Box<dyn 'static + FnMut(hyper::Request<Body>, hyper::http::response::Builder) -> CallbackResponse>; //// capturing by mut T - the closure inside the Box is valid as long as the Callback is valid due to the 'static lifetime and will never become invalid until the variable that has the Callback type drop
    type CallbackResponse = Box<dyn Future<Output=ConseResult<hyper::Response<Body>, hyper::Error>> + Send + Sync + 'static>; //// CallbackResponse is a future object which will be returned by the closure and has bounded to Send to move across threads and .awaits - the future inside the Box is valid as long as the CallbackResponse is valid due to the 'static lifetime and will never become invalid until the variable that has the CallbackResponse type drop
    type SafeShareAsync = Arc<Mutex<Pin<Box<dyn Future<Output=u8> + Send + Sync + 'static>>>>; //// this type is a future object which has pinned to the ram inside a Box pointer and can be shared between thread safely also it can be mutated by threads - pinning the Boxed future object into the ram to prevent from being moved (cause rust don't have gc and each type will be dropped once it goes out of its scope) since that future object must be valid across scopes and in the entire lifetime of the app until we await on it 
    type SafeShareClosure = Arc<Mutex<Box<dyn FnOnce(hyper::Request<Body>) -> hyper::Response<Body> + Send + Sync + 'static>>>; //// this type is safe and sendable to share between threads also it can be mutated by a thread using a mutex guard; we have to use the &dyn keyword or put them inside the Box<dyn> for traits if we want to treat them as a type since they have no sepecific size at compile time thus they must be referenced by the &dyn or the Box<dyn> 

    unsafe impl Send for Api{}
    unsafe impl Sync for Api{}

    pub struct Api{
        pub name: String,
        pub req: Option<hyper::Request<Body>>,
        pub res: Option<hyper::http::response::Builder>,
        pub callback: Option<Callback>, //// the generic type of the callback field is the Callback type which is FnMut and a Future object for its return type inside the Box
        pub access_level: Option<u8>, //// it might be None and the api doesn't require an access level
    }

    impl Api{

        // -----------------------------------------------------------------------------------------------------------------------------
        // NOTE - if Clone trait is not implemented for a type and that type is also a field of a structure we can't have &self in
        //        structure methods since using a shared reference requires Clone trait be implemented for all types of the structure 
        //        otherwise we can't share none of the fields of the structure and by calling a method of the structure on the instance
        //        the instance will be no longer valid and be moved.
        // NOTE - we can borrow the req and res cause Request and Response structs are not bounded to Copy and Clone traits 
        //        thus cb closure (callback) arguments must be references to Request and Response objects.
        // NOTE - we can use as_ref() method to borrow the self.req and self.res cause as_ref() 
        //        converts Option<T> to Option<&T> then we can unwrap them to get the borrowed objects.
        // NOTE - don't put & behind self or borrow Api fields cause sharing Api fields between other threads using a shared reference
        //        with & or borrowing the ownership is impossible cause by not implemented trait Clone (a super trait of Copy) 
        //        for hyper Request and Response structs error (neither we can copy nor clone the api object).
        // NOTE - the body of the `cb` in post and get methods is an async move{} means it'll return a future object
        //        which we can solve it using .await later
        // NOTE - every api router method must be bounded to Send + Syn + 'static traits to be shareable, safe to send 
        //        and have valid lifetime across threads and .awaits.
        // NOTE - since we can't put & behind the mut self thus we can't have the instance of the Api in later scopes
        //        after calling its post or get methods and due to this fact we've built controllers which implements
        //        only one Api instance per writing api pattern, means we can have only one Api instance inside
        //        a crate therefore we must have one controller per each Api instance to handle the incoming request
        //        inside that controller which is related to a specific route (MVC like design pattern).
        // NOTE - we can't have api.post().await and api.get().await inside the same scope from one instance since with the first 
        //        use the api instance will be moved and its lifetime will be dropped due to the above third NOTE.
        // NOTE - since both api.post() and api.get() methods are async thus we have to await on them to run their callback closures
        //        which contain the logic of the whole controller. 
        // -----------------------------------------------------------------------------------------------------------------------------

        /*
            //// Example to create Api object:
        
                let api = Api::new(Some(req), Some(Response::builder()));
                api.post("/home", |req, res| async move{
                    
                }).await
        */

        pub fn new(request: Option<hyper::Request<Body>>, response: Option<hyper::http::response::Builder>) -> Self{
            Api{
                name: String::from(""),
                req: request,
                res: response,
                callback: None, // TODO - caching using closures
                access_level: None, // TODO
            }
        } 
        
        pub async fn post<F, C>(mut self, endpoint: &str, mut cb: F) -> ConseResult<hyper::Response<Body>, hyper::Error> //// defining self (an instance of the object) as mutable cause we want to assign the name of the api; since we didn't borrow the self (the instance itself) using & we can't call this method for the second call cause the ownership of the instance will be moved in first call  
                            where F: FnMut(hyper::Request<Body>, hyper::http::response::Builder) -> C + Send + Sync + 'static, //// capturing by mut T - generic type C can be bounded to Send + Sync + 'static traits and lifetime to be shreable, safe to send and valid across threads and .awaits
                            C: Future<Output=ConseResult<hyper::Response<Body>, hyper::Error>> + Send + Sync + 'static, //// C is a future object which will be returned by the closure and has bounded to Send to move across threads and .awaits
        {
            self.name = endpoint.to_string(); //// setting the api name to the current endpoint
            let req = self.req.unwrap();
            let res = self.res.unwrap();
            let cb_res = cb(req, res).await.unwrap(); //// calling the passed in closure to the post method by passing the request and response objects since this closure callback contains the body of the controller method - this would be of type either hyper::Response<Body> or hyper::Error
            Ok(cb_res)
        }


        pub async fn get<F, C>(mut self, endpoint: &str, mut cb: F) -> ConseResult<hyper::Response<Body>, hyper::Error> //// defining self (an instance of the object) as mutable cause we want to assign the name of the api; since we didn't borrow the self (the instance itself) using & we can't call this method for the second call cause the ownership of the instance will be moved in first call  
                            where F: FnMut(hyper::Request<Body>, hyper::http::response::Builder) -> C + Send + Sync + 'static, //// capturing by mut T - generic type C can be bounded to Send + Sync + 'static traits and lifetime to be shreable, safe to send and valid across threads and .awaits
                            C: Future<Output=ConseResult<hyper::Response<Body>, hyper::Error>> + Send + Sync + 'static, //// C is a future object which will be returned by the closure and has bounded to Send to move across threads and .awaits
        {
            self.name = endpoint.to_string(); //// setting the api name to the current endpoint
            let req = self.req.unwrap();
            let res = self.res.unwrap();
            let cb_res = cb(req, res).await.unwrap(); //// calling the passed in closure to the post method by passing the request and response objects since this closure callback contains the body of the controller method - this would be of type either hyper::Response<Body> or hyper::Error
            Ok(cb_res)
        }

        pub async fn set_name(&mut self, endpoint: &str){ //// we must define self as mutable cause we want to change the name field
            let endpoint_name = endpoint.to_string();
            self.name = endpoint_name;
        }

        pub async fn get_name(&self) -> String{
            let endpoint_name = self.name.to_string(); //// self.name is the dereferenced value of the &self.name and will be done automatically by the compiler 
            endpoint_name 
        }
    }

    #[derive(Clone, Debug)] //// can't bound Copy trait cause engine and url are String which are heap data structure 
    pub struct Db{
        pub mode: Mode,
        pub engine: Option<String>,
        pub url: Option<String>,
        pub instance: Option<Client>,
    }

    impl Default for Db{
        fn default() -> Db {
            Db{
                mode: self::Mode::Off,
                engine: None,
                url: None,
                instance: None,
            }
        }
    }

    impl Db{
        
        pub async fn new() -> Result<Db, Box<dyn std::error::Error>>{
            Ok(
                Db{ //// building an instance with generic type C which is the type of the db client instance
                    mode: super::app::Mode::On, //// 1 means is on 
                    engine: None, 
                    url: None,
                    instance: None,
                }
            )
        }
        
        //// don't call a method which has self (not &self) as it's first 
        //// param since by call it on the instance the instance will be 
        //// dropped from the ram move borrowed form of the type in most 
        //// cases unless its pointer is a shared pointer in which we 
        //// must deref it using * or clone
        //
        //// Client object uses std::sync::Arc internally, so it can safely be 
        //// shared across threads or async tasks like tokio::spawn(async move{}) 
        //// green threads also it is highly recommended to create a single 
        //// Client and persist it for the lifetime of your application.
        pub async fn GetMongoDbInstance(&self) -> Client{ //// it'll return an instance of the mongodb client - we set the first argument to &self in order to have the instance of the object later on after calling this method and prevent ownership moving
            Client::with_uri_str(self.url.as_ref().unwrap()).await.unwrap() //// building mongodb client instance
        }

    }

    #[derive(Clone, Debug)]
    pub struct Storage{
        pub id: Uuid,
        pub db: Option<Db>, //// we could have no db at all
    }

    impl Storage{
        pub async fn get_db(&self) -> Option<&Client>{
            match self.db.as_ref().unwrap().mode{
                Mode::On => self.db.as_ref().unwrap().instance.as_ref(), //// return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Client> or Option<&T>
                Mode::Off => None, //// no db is available cause it's off
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub enum Mode{ //// enum uses 8 bytes (usize which is 64 bits on 64 bits arch) tag which is a pointer pointing to the current variant - the total size of this enum is 8 bytes tag + the largest variant size = 8 + 0 = 8 bytes; cause in our case On and Off variant both have 0 size
        On, //// zero byte size
        Off, //// zero byte size
    }

    /*
        can't bound the T to ?Sized since 
        T is inside the Option which the size
        of the Option depends on the T at 
        compile time 
    */
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Response<'m, T>{
        pub data: Option<T>,
        pub message: &'m str, //// &str are a slice of String thus they're behind a pointer and every pointer needs a valid lifetime which is 'm in here 
        pub status: u16,
    }

    pub struct OtpInfo{
        pub otp_auth: Box<dyn misc::otp::Otp + Send + Sync + 'static>, //// otp_auth is a trait of type Otp which must be Send Sync and static to be shareable between routers' threads - since we can't have a trait as a struct field directly due to its unknown size at compile time thus  we've put the Otp trait inside the Box since the Box has its own lifetime which avoid us using references and lifetimes inside the struct fields
    }

    #[derive(Serialize, Deserialize)]
    pub struct Nill<'n>(pub &'n [u8]); //// this will be used for empty data inside the data field of the Response struct - 'n is the lifetime of the &[u8] type cause every pointer needs a lifetime in order not to point to an empty location inside the memory (dangling pointer)

    pub async fn shutdown_signal(signal: Receiver<u8>){
        match signal.await{ //// await on signal to get the message in down side of the channel
            Ok(s) => {
                if s == 0{
                    info!("🔌 shutting down the server - {}", chrono::Local::now().naive_local());
                    tokio::signal::ctrl_c().await.expect("😖 install the plugin CTRL+C signal to the server");
                } else if s == 1 { // TODO - freez the server
                    // ...
                }
            },
            Err(e) => {
                error!("receiving error: [{}] cause sender is not available - {}", e, chrono::Local::now().naive_local())
            }
        }
    }
}




//// --------------------------------------------------------------------------
//// ------------------------------- jwt module -------------------------------
//// --------------------------------------------------------------------------
pub mod jwt{

    use std::env;
    use chrono::Utc;
    use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, TokenData};
    use serde::{Serialize, Deserialize};
    use mongodb::bson::oid::ObjectId;



    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims{
        pub _id: Option<ObjectId>, //// mongodb object id
        pub username: String,
        pub access_level: u8,
        pub exp: i64, //// expiration timestamp
        pub iat: i64, //// issued timestamp
    }



    pub async fn construct(payload: Claims) -> Result<String, jsonwebtoken::errors::Error>{
        let encoding_key = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let token = encode(&Header::new(Algorithm::HS512), &payload, &EncodingKey::from_secret(encoding_key.as_bytes()));
        token
    }

    pub async fn deconstruct(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error>{
        let encoding_key = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let decoded_token = decode::<Claims>(token, &DecodingKey::from_secret(encoding_key.as_bytes()), &Validation::new(Algorithm::HS512));
        decoded_token
    }

    pub async fn gen_times() -> (i64, i64){
        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nano to sec
        let exp_time = now + env::var("JWT_EXPIRATION").expect("⚠️ found no jwt expiration time").parse::<i64>().unwrap();
        (now, exp_time)
    }

}



//// --------------------------------------------------------------------------
//// ------------------------------- otp module -------------------------------
//// --------------------------------------------------------------------------
pub mod otp{


    use super::*; //// loading all the loaded crates inside the utils (super) itself


    
    
    pub struct OtpSuccess(pub Response<Body>, pub OtpInput); //// OtpSuccess is a tuple like struct with two inputs


    #[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Default)]
    pub struct OtpInput{
        pub id: String, //// the stringified of the Uuid
        pub code: Option<String>,
        pub phone: Option<String>,
    }

    #[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
    pub struct Auth{
        token: String,
        template: String,
        pub otp_input: OtpInput
    }

    impl Auth{

        //// in order to bound the Otp to what we're returning from the function the trait Otp must be implemented for the type that we're returning its instance from the function
        //// returning impl Otp from function means we're implementing the trait for the object that is returning from the function regardless of its type that we're returning from the function cause compiler will detect the correct type in compile time and implement or bound the trait for that type
        pub fn new(token: String, template: String) -> impl Otp{ //// since traits don't have fix size at compile time thus when we want to return them from the function we have to return an instance of the type that the trait has been implemented for since the trait size will be the size of that type which is bounded to the trait  
            Self{
                token,
                template,
                otp_input: OtpInput::default(),
            }
        }

    }

    //// we can define async method inside the trait; since traits' size are unknown at compile time 
    //// thus we can't have async method cause async method are future objects which must be pinned 
    //// to the ram to be awaited later for completion and we can solve this issue by rewriting the method 
    //// of the trait in such a way that it should return something like Pin<Box<dyn std::future::Future<Output = ()> + Send + 'async_trait>>>
    //// like what async_trait crate does.
    #[async_trait] 
    pub trait Otp{

        async fn send_code(&mut self) -> Result<OtpSuccess, hyper::Error>; //// the error part is of type hyper::Error which will be returned automatically if the success part gets failed
        async fn get_otp_input(&self) -> Option<OtpInput>;
        async fn set_otp_input(&mut self, otp_info: OtpInput) -> Option<OtpInput>;

    }

    #[async_trait]
    impl Otp for Auth{
        
        async fn send_code(&mut self) -> Result<OtpSuccess, hyper::Error>{

            let code: String = (0..4).map(|_|{
            let idx = gen_random_idx(random::<u8>() as usize); //// idx is one byte cause it's of type u8
                CHARSET[idx] as char //// CHARSET is of type utf8 bytes thus we can index it which it's length is 10 bytes (0-9)
            }).collect();

            self.otp_input.code = Some(code.clone());
            let recipient = self.otp_input.phone.clone().unwrap();
            let uri = format!("http://api.kavenegar.com/v1/{}/verify/lookup.json?receptor={}&token={}&template={}", self.token, recipient, code, self.template).as_str().parse::<Uri>().unwrap(); //// parsing it to hyper based uri
            let client = HyperClient::new();
            let sms_response_stream = client.get(uri).await.unwrap(); //// since we can't use .await inside trait methods thus we have to solve the future using block_on() function
            
            Ok(
                OtpSuccess(sms_response_stream, self.otp_input.clone()) //// we have to clone the self.otp_input to prevent its ownership moving since by moving it into the field of a structure it'll lose its ownership 
            )

        }

        async fn get_otp_input(&self) -> Option<OtpInput>{
            Some(
                self.otp_input.clone()
            )
        }

        async fn set_otp_input(&mut self, otp_info: OtpInput) -> Option<OtpInput>{
            self.otp_input = otp_info;
            Some(
                self.otp_input.clone()
            )
        }
        

    }

}











#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UploadFile{
    pub name: Option<String>, //// we've used Option since it might be no path at all
    pub time: u64,
}


pub struct Block;
pub const SEEDS: &[[[Block; 2]; 2]] = &[[[Block, Block], [Block, Block]]];





// ------------------------------ utility methods
// -----------------------------------------------------------------------------------------
// -----------------------------------------------------------------------------------------
// -----------------------------------------------------------------------------------------
pub async fn into_box_slice(u8_vector: &Vec<u8>) -> Result<Box<[u8; 4]>, String>{ //// the return type of this function is either a Box of [u8] slice with 4 bytes (32 bits) or a String of the error
    let to_owned_vec = u8_vector.to_owned(); //// creating owned vector from borrowed vector by cloning to call into_boxed_slice() method on the vector
    let boxed_slice = to_owned_vec.into_boxed_slice(); //// converting the collected bytes into a Box slice or array of utf8 bytes - we put it inside the Box cause the size of [u8] is not known at compile time
    let boxed_array: Box<[u8; 4]> = match boxed_slice.try_into() { //// Boxing u8 with size 4 cause our input number is 32 bits which is 4 packs of 8 bits
        Ok(arr) => return Ok(arr), //// returning a Box of 4 u8 slice or 4 packs of 8 bits
        Err(o) => return Err(format!("vector length must be 4 but it's {}", o.len())),
    };
}



// -----------------------------------
// handling a recursive async function
// -----------------------------------
// https://rust-lang.github.io/async-book/07_workarounds/04_recursion.html
// NOTE - Future trait is an object safe trait thus we have to Box it with dyn keyword to have kinda a pointer to the heap where the object is allocated in runtime
// NOTE - a recursive `async fn` will always return a Future object which must be rewritten to return a boxed `dyn Future` to prevent infinite size allocation in runtime from heppaneing some kinda maximum recursion depth exceeded prevention process
//// the return type can also be ... -> impl std::future::Future<Output=usize>
//// which implements the future trait for the usize output also BoxFuture<'static, usize>
//// is a pinned Box under the hood because in order to return a future as a type
//// we have to return its pinned pointer since future objects are traits and 
//// traits are not sized at compile time thus we have to put them inside the 
//// Box or use &dyn to return them as a type and for the future traits we have
//// to pin them into the ram in order to be able to solve them later so we must 
//// return the pinned Box (Box in here is a smart pointer points to the future)
//// or use impl Trait in function return signature. 
//
//// async block needs to be pinned into the ram and since they are traits of 
//// the Future their pointer will be either Box<dyn Trait> or &dyn Trait, 
//// to pin them into the ram to solve them later.
//
//// since async blocks are of type Future trait in roder to return them
//// as a type their pointer either Box<dyn Trait> or &dyn Trait must be
//// pinned into the ram to let us solve them later because rust doesn't 
//// have gc and it'll drop the type after it moved into the new scope or
//// another type thus for the future objects we must pin them to ram and 
//// tell rust hey we're moving this in other scopes but don't drop it because
//// we pinned it to the ram to solve it in other scopes, also it must have
//// valid lifetime during the the entire lifetime of the app.
//
//// BoxFuture<'fut, ()> is Pin<alloc::boxed::Box<dyn Future<Output=()> + Send + Sync + 'fut>>
pub fn async_gen_random_idx(idx: usize) -> BoxFuture<'static, usize>{ // NOTE - pub type BoxFuture<'a, T> = Pin<alloc::boxed::Box<dyn Future<Output = T> + Send + 'a>>
    async move{
        if idx <= CHARSET.len(){
            idx
        } else{
            gen_random_idx(random::<u8>() as usize)
        }
    }.boxed() //// wrap the future in a Box, pinning it
}
pub fn ret_boxed_future() -> std::pin::Pin<Box<dyn futures::future::Future<Output=()>>>{ //// Pin takes a pointer to the type and since traits are dynamic types thir pointer can be either &dyn ... or Box<dyn...>
    Box::pin(async move{ //// pinning the async block into the ram to solve it later 
        ()
    })
}


//// recursive random index generator
pub fn gen_random_idx(idx: usize) -> usize{
    if idx < CHARSET.len(){
        idx
    } else{
        gen_random_idx(random::<u8>() as usize)
    }
}



pub async fn upload_asset(path: &str, mut payload: Multipart<'_>, doc_id: &String) -> Option<String>{ //// parsing the incoming file stream into MultipartItem instances - Multipart struct takes a lifetime and we've passed an unnamed lifetime to that
    
    // https://github.com/hyperium/hyper/blob/master/examples/send_file.rs

    fs::create_dir_all(path).unwrap(); //// creating the directory which must be contains the file
    let mut filename = "".to_string();
    let mut filepath = "".to_string();
    while let Some(mut field) = payload.next_field().await.map_err(|err| Error::wrap(err)).unwrap(){ //// reading the next field which contains IO stream future object of utf8 bytes of the payload is a mutable process and due to this fact we've defined the payload as a mutable type; we've mapped each incoming utf8 bytes future into an error if there was any error on reading them 
        let field_name = field.name(); //// getting the field's name if provided in "Content-Disposition" header from the client
        let field_file_name = field.file_name(); //// getting the field's filename if provided in "Content-Disposition" header from the client
        filename = format!("{} - {}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(), field_file_name.unwrap()); //// creating the new filename with the server time
        filepath = format!("{}/{}/{}", path, doc_id, sanitize_filename::sanitize(&filename)); //// creating the new file path with the sanitized filename and the passed in document id
        let mut buffer_file = fs::File::create(filepath.clone()).unwrap();
        while let Some(chunk) = field.chunk().await.map_err(|err| Error::wrap(err)).unwrap(){ //// mapping the incoming IO stream of futre object which contains utf8 bytes into a file
            buffer_file.write_all(&chunk).unwrap(); //// filling the buffer_file with incoming chunks from each field and write itnto the server hard
        } //// this field will be dropped in here to get the next field
    }
    
    Some(filepath)

}



pub async fn set_user_access(username: String, new_access_level: i64, storage: Option<Arc<app::Storage>>) -> Result<schemas::auth::UserInfo, app::Nill<'static>>{ //// Nill struct requires a lifetime since there is no lifetime has passed to the function we have to use 'static lifetime  

    // NOTE - we can also use clone() method to clone the db instead of using the as_ref() method
    let app_storage = match storage.as_ref().unwrap().db.as_ref().unwrap().mode{
        app::Mode::On => storage.as_ref().unwrap().db.as_ref().unwrap().instance.as_ref(), //// return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Client>
        app::Mode::Off => None, //// no db is available cause it's off
    };

    ////////////////////////////////// DB Ops

    let update_option = FindOneAndUpdateOptions::builder().return_document(Some(ReturnDocument::After)).build();
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let users = app_storage.unwrap().database(&db_name).collection::<schemas::auth::UserInfo>("users"); //// selecting users collection to fetch all user infos into the UserInfo struct
    match users.find_one_and_update(doc!{"username": username}, doc!{"$set": {"access_level": new_access_level, "updated_at": Some(Utc::now().timestamp())}}, Some(update_option)).await.unwrap(){ //// finding user based on username to update access_level field to dev access
        Some(user_doc) => Ok(user_doc), 
        None => Err(app::Nill(&[])),
    }

    //////////////////////////////////

}



pub async fn event_belongs_to_god(god_id: ObjectId, event_id: ObjectId, app_storage: Client) -> bool{


    ////////////////////////////////// DB Ops

    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let events = app_storage.database(&db_name).collection::<schemas::event::EventInfo>("events");
    match events.find_one(doc!{"_id": event_id}, None).await.unwrap(){
        Some(event_doc) => {
            if event_doc.group_info.unwrap().god_id.unwrap() == god_id.to_string(){
                return true;
            } 
            false
        }, 
        None => false,
    }

    //////////////////////////////////

}


pub async fn get_random_doc(storage: Option<&Client>) -> Option<schemas::game::RoleInfo>{
    let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
    let mut all = vec![];
    let roles = storage.clone().unwrap().database(&db_name).collection::<schemas::game::RoleInfo>("roles");
    let random_record_setup = doc!{"$sample": {"size": 1}};
    let pipeline = vec![random_record_setup];

    ////////////////////////////////// DB Ops
    
    match roles.aggregate(pipeline, None).await{
        Ok(mut cursor) => {
            while let Some(random_doc) = cursor.try_next().await.unwrap(){
                let random_role_info = bson::from_document::<schemas::game::RoleInfo>(random_doc).unwrap();
                all.push(random_role_info)
            }
            let role = all[0].clone();
            Some(role)
        },
        Err(e) => None,
    }

    //////////////////////////////////

}



pub async fn build_server(router: Router<Body, hyper::Error>) -> Server<AddrIncoming, RouterService<Body, hyper::Error>>{ //// it'll create the server from the passed router apis

    let host = env::var("HOST").expect("⚠️ no host variable set");
    let port = env::var("CONSE_PORT").expect("⚠️ no port variable set");
    let server_addr = format!("{}:{}", host, port).as_str().parse::<SocketAddr>().unwrap();
    let conse_service = RouterService::new(router).unwrap();
    let conse_server = Server::bind(&server_addr).serve(conse_service);
    conse_server

}





// ------------------------------ simd using mpsc channel + tokio + native thread
// -----------------------------------------------------------------------------------------
// -----------------------------------------------------------------------------------------
// NOET - if we have a buffer we can convert it into chunks of bytes using simd algos 
// -----------------------------------------------------------------------------------------

pub async fn simd<F>(number: u32, ops: F) -> Result<u32, String> where F: Fn(u8) -> u8 + std::marker::Send + 'static + Clone{ //// in order to move the F between threads it must be bounded to Send trait
        
        
    let threads = 4; //// the total number of all packs or chunks containing 8 bits which in this case is 4 cause our number is of type u32
    let (sender, receiver) = mpsc::channel::<u8>();
    let big_end_bytes = number.to_be_bytes(); //// network bytes which is in form utf8 or big endian bytes - since there are 4 chunks of 8 bits in the context of u32 bits there will be 4 chunks of 8 bits each chunk between 0 up to 255 
    let mut index = 0;
    


    while index < big_end_bytes.len(){
        
        info!("chunk {:?} in utf8 format -> [{:?}] at time {:?}", index, big_end_bytes[index], chrono::Local::now().naive_local());
        let cloned_sender = sender.clone();
        let cloned_ops = ops.clone();
        tokio::spawn(async move{ //// spawning async task to solve it on the background using tokio green threads based on its event loop model
            thread::spawn(move || async move{ //// the return body of the closure is async block means it'll return a future object (trait Future has implemented for that) with type either () or a especific type and for solving it we have to be inside an async function - in order to capture the variables before spawning scope we have to use move keyword before ||
                let new_chunk = cloned_ops(big_end_bytes[index]);
                info!("\tsender-channel---(chunk {:?})---receiver-channel at time {:?} ", index, chrono::Local::now().naive_local());
                cloned_sender.send(new_chunk).unwrap(); //// sending new chunk to down side of the channel cause threads must communicate with each other through a mpsc channel to avoid data race condition   
            });
        });
        index+=1

    }

    
    info!("collecting all chunks received from the receiver at time {:?}", chrono::Local::now().naive_local());
    let bytes: Vec<u8> = receiver.iter().take(threads).collect(); //// collecting 4 packs of 8 bits to gather all incoming chunks from the channel
    info!("collected bytes -> {:?} at time {:?}", bytes, chrono::Local::now().naive_local());
    
    
    
    let boxed_array = self::into_box_slice(&bytes).await.unwrap(); //// converting &Vec<u8> to [u8] with a fixed size
    let result = *boxed_array; //// dereferencing the box pointer to get the value inside of it 
    let final_res = u32::from_be_bytes(result); //// will create a u32 number from 4 pack of 8 bits - from_be_bytes() method creates a native endian integer value from its representation as a byte array in big endian
    Ok(final_res) //// the final results might be different from the input due to the time takes to send the each chunks through the channel and receive them from the receiver thus the order of chunks will not be the same as the input



}






/*
    
    ┏———————————————┓
       NFT LAYRING
    ┗———————————————┛

    A MULTITHREADED AND ASYNC NFT LAYERING TOOLS

*/
pub async fn layering(){

    pub const WIDTH: usize = 32;
    pub const HEIGHT: usize = 32;
    pub const RGBA: usize = 4;

    pub struct Image{
        /*
            there must be HEIGHT number of [u8; RGBA]
            which is like 32 rows X 4 cols and  
            WIDTH number of [[u8; RGBA]; HEIGHT]
            which is like 32 X 32 X 4
        */
        pub hat: [[[u8; RGBA]; HEIGHT]; WIDTH], //// 32 X 32 X 4 => 32 Pixels and RGBA channels
        pub mask: [[[u8; RGBA]; HEIGHT]; WIDTH] //// 32 X 32 X 4 => 32 Pixels and RGBA channels
    }

    let (sender, receiver) = tokio::sync::mpsc::channel::<HashMap<&str, Vec<&str>>>(1024);

    let assets_path = "assets"; //// in the root of the project
    let nfts_path = "nfts"; //// in the root of the project

    fn update_asset_to_path<'s>(
        mut asset_to_path: HashMap<&'s str, Vec<String>>, 
        key: &'s str, key_images: Vec<String>) 
        -> HashMap<&'s str, Vec<String>>{
        asset_to_path.entry(key).and_modify(|v| *v = key_images);
        asset_to_path
    } 

    tokio::spawn(async move{

        // hashmap can be a 3d arr also and reading 
        // from it is slower that arr and vec 

        let assets_names = &["Beard", "Hat", "Mask"];
        let mut asset_to_path: HashMap<&str, Vec<String>> = HashMap::new(); //// a map of between asset name and their images path
        for asset in assets_names{
            asset_to_path.entry(asset).or_insert(vec![]);
        }

        let assets = std::fs::read_dir(assets_path).unwrap();
        for asset in assets{
            //// since unwrap() takes the ownership of the type 
            //// we've borrowed the asset using as_ref() method
            //// which returns a borrow of the asset object which
            //// let us to have the asset later in other scopes.
            let filename = asset.as_ref().unwrap().file_name();
            let filepath = asset.as_ref().unwrap().path();
            let filepath_string = filepath.display().to_string();
            let mut asset_to_path_clone = asset_to_path.clone();
            let asset_to_path_keys = asset_to_path_clone.keys();
            let filepath_string_clone = filepath_string.clone();
            for key in asset_to_path_keys{ 
                if filepath_string_clone.starts_with(*key){
                    //// if a type is behind an immutable shared reference 
                    //// it can't mutate the data unless we define it's 
                    //// pointer as mutable in the first place or convert 
                    //// it to an owned type which returns Self. 
                    let mut key_images = asset_to_path.get(key).unwrap().to_owned();
                    key_images.push(filepath_string_clone.clone());
                    asset_to_path = update_asset_to_path(asset_to_path.clone(), key, key_images);
                }
            }
        }


        let (sender_flag, mut receiver_flag) = 
        tokio::sync::mpsc::channel::<u8>(1024); //// mpsc means multiple thread can read the data but only one of them can mutate it at a time
        tokio::spawn(async move{

            type Job<T> = std::thread::JoinHandle<T>; 
            let job: Job<_> = std::thread::spawn(||{});
            
            std::thread::scope(|s|{
                s.spawn(|| async{ //// making the closure body as async to solve async task inside of it 
                    sender_flag.send(1).await.unwrap(); //// sending data to the downside of the tokio jobq channel
                    for asset_path in asset_to_path.values(){
                        tokio::spawn(async move{
                            // reading the shared sate data from the
                            // receiver_flag mpsc receiver to acquire 
                            // the lock on the mutexed data.
                            // ... 
                            // make a combo of each asset path in a separate thread asyncly 
                            // while idx < combos.len()!{
                            //     bin(i%3!).await;
                            //     010
                            //     01
                            // }
                            // ...
                        });
                    }
                });
                s.spawn(|| async{
                    sender_flag.send(2).await.unwrap();
                });
                s.spawn(|| async{ //// making the closure body as async to solve async task inside of it 
                    while let Some(input) = receiver_flag.recv().await{ //// waiting on data stream to receive them asyncly
                        // do whatever with the collected data of all workers 
                        // ...
                    }
                    let data: Vec<u8> = receiver_flag.try_recv().into_iter().take(2).collect();
                });
            });
        });
    });

}





// ------------------------------ macros
// -----------------------------------------------------------------------------------------
// -----------------------------------------------------------------------------------------
// -----------------------------------------------------------------------------------------

#[macro_export]
macro_rules! db {

    ($name:expr, $engine:expr, $host:expr, $port:expr, $username:expr, $password:expr) => {
                
        { //// this is the key! this curly braces is required to use if let statement, use libs and define let inside macro
            
            use crate::misc::app::*;
            
            let empty_app_storage = Some( //// putting the Arc-ed db inside the Option
                Arc::new( //// cloning app_storage to move it between threads
                    Storage{ //// defining db context 
                        id: Uuid::new_v4(),
                        db: Some(
                            Db{
                                mode: Mode::Off,
                                instance: None,
                                engine: None,
                                url: None,
                            }
                        ),
                    }
                )
            );
            let app_storage = if $engine.as_str() == "mongodb"{
                info!("➔ 🛢️ switching to mongodb on address: [{}:{}]", $host, $port);
                let environment = env::var("ENVIRONMENT").expect("⚠️ no environment variable set");
                let db_addr = if environment == "dev"{
                    format!("{}://{}:{}", $engine, $host, $port)
                } else if environment == "prod"{
                    format!("{}://{}:{}@{}:{}", $engine, $username, $password, $host, $port)
                } else{
                    "".to_string()
                };
                match Db::new().await{
                    Ok(mut init_db) => { //// init_db instance must be mutable since we want to mutate its fields
                        init_db.engine = Some($engine);
                        init_db.url = Some(db_addr);
                        let mongodb_instance = init_db.GetMongoDbInstance().await; //// the first argument of this method must be &self in order to have the init_db instance after calling this method, cause self as the first argument will move the instance after calling the related method and we don't have access to any field like init_db.url any more due to moved value error - we must always use & (like &self and &mut self) to borrotw the ownership instead of moving
                        Some( //// putting the Arc-ed db inside the Option
                            Arc::new( //// cloning app_storage to move it between threads
                                Storage{ //// defining db context 
                                    id: Uuid::new_v4(),
                                    db: Some(
                                        Db{
                                            mode: init_db.mode,
                                            instance: Some(mongodb_instance),
                                            engine: init_db.engine,
                                            url: init_db.url,
                                        }
                                    ),
                                }
                            )
                        )
                    },
                    Err(e) => {
                        error!("😕 init db error - {}", e);
                        empty_app_storage //// whatever the error is we have to return and empty app storage instance 
                    }
                }
            } else if $engine.as_str() == "postgres"{
                info!("➔ 🛢️ switching to postgres on address: [{}:{}]", $host, $port);
                let environment = env::var("ENVIRONMENT").expect("⚠️ no environment variable set");                
                if environment == "dev"{
                    format!("{}://{}:{}", $engine, $host, $port)
                } else if environment == "prod"{
                    format!("{}://{}:{}@{}:{}", $engine, $username, $password, $host, $port)
                } else{
                    "".to_string()
                };
        
                // TODO - configure surrealdb instance
                todo!();
            
            } else{
                empty_app_storage
            };

            app_storage //// returning the created app_storage

        }
    };

}

#[macro_export]
macro_rules! passport {
    (
      $req:expr,
      $db:expr,
      $access_levels:expr
    ) 
    => {

        { //// this is required if we want to import modules and use the let statements
            use crate::middlewares;
            use crate::misc::app;
            use hyper::{header, StatusCode, Body, Response};
            
            let res = Response::builder();
            let pass = middlewares::auth::pass($req).await;

            if pass.is_ok(){
                
                let (token_data, req) = pass.unwrap(); //// the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have the borrowed form of the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function
                let _id = token_data.claims._id;
                let username = token_data.claims.username.clone();
                let access_level = token_data.claims.access_level;
                if middlewares::auth::user::exists(Some(&$db.clone()), _id, username.clone(), access_level).await{ //// finding the user with these info extracted from jwt
                    if $access_levels.contains(&access_level){   
                        Some(
                                (
                                    Some(token_data),
                                    Some(req), //// we must send the req object to decode its body for further logics inside the related route
                                    None //// there is no need to return a response object since it'll be fulfilled inside the related route
                                )
                            ) 
                    } else{
                        //////////////////////////////
                        ////// ACCESS DENIED RESPONSE
                        //////////////////////////////
                        let response_body = app::Response::<app::Nill>{
                            data: Some(app::Nill(&[])), //// data is an empty &[u8] array
                            message: ACCESS_DENIED,
                            status: 403,
                        };
                        let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                        let response = Ok(
                                    res
                                        .status(StatusCode::FORBIDDEN)
                                        .header(header::CONTENT_TYPE, "application/json")
                                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                        .unwrap() 
                                );
                        Some(
                                (
                                    None, 
                                    None,
                                    Some(response)
                                )
                            )
                    }
                    
                } else{
                    //////////////////////////////
                    ////// NOT FOUND USER RESPONSE
                    //////////////////////////////
                    let response_body = app::Response::<app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                        data: Some(app::Nill(&[])), //// data is an empty &[u8] array
                        message: DO_SIGNUP, //// document not found in database and the user must do a signup
                        status: 404,
                    };
                    let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                    let response = Ok(
                                    res
                                        .status(StatusCode::NOT_FOUND)
                                        .header(header::CONTENT_TYPE, "application/json")
                                        .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                        .unwrap() 
                                );
                    Some(
                            (
                                None,
                                None,
                                Some(response)
                            )
                        )
                }
            } else{
                ///////////////////////////
                ////// WRONG TOKEN RESPONSE
                ///////////////////////////
                let e = pass.err().unwrap();
                let response_body = app::Response::<app::Nill>{
                    data: Some(app::Nill(&[])), //// data is an empty &[u8] array
                    message: &e.to_string(), //// e is of type String and message must be of type &str thus by taking a reference to the String we can convert or coerce it to &str
                    status: 500,
                };
                let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
                let response = Ok(
                            res
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .header(header::CONTENT_TYPE, "application/json")
                                .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                                .unwrap() 
                        );
                Some(
                        (
                            None,
                            None, //// we can't have req in here since it has been moved into the pass() function and it implements neither Copy nor Clone trait 
                            Some(response)
                        )
                    )
            }
        }
    }
}

/*
    we can define as many as response object 
    since once the scope or method or the match
    arm gets executed the lifetime of the response
    object will be dropped from the ram since rust 
    doesn't have gc :) 
*/
#[macro_export]
macro_rules! resp {
    (
        $data_type:ty, //// ty indicates the type of the data
        $data:expr,
        $msg:expr,
        $code:expr,
        $content_type:expr
    ) => {

        {
            use hyper::{header, StatusCode, Body, Response};
            use crate::misc::app;

            let code = $code.as_u16(); 
            let res = Response::builder();
            let response_body = app::Response::<$data_type>{
                data: Some($data),
                message: $msg,
                status: code,
            };
            let response_body_json = serde_json::to_string(&response_body).unwrap(); //// converting the response body object into json stringify to send using hyper body
            let response = Ok(
                res
                    .status($code)
                    .header(header::CONTENT_TYPE, $content_type)
                    .body(Body::from(response_body_json)) //// the body of the response must be serialized into the utf8 bytes to pass through the socket here is serialized from the json
                    .unwrap() 
            );
            return response;

        }
    }
}

#[macro_export]
macro_rules! contract {

    /*

        contract!{

            NftContract, //// name of the contract
            "wildonion.near", //// the contract owner
            /////////////////////
            //// contract fields
            /////////////////////
            [
                contract_owner: AccountId, 
                deposit_by_owner: HashMap<AccountId, near_sdk::json_types::U128>, 
                contract_balance: near_sdk::json_types::U128
            ]; //// fields
            /////////////////////
            //// contract methods
            /////////////////////
            [ 
                "init" => [ //// array of init methods
                    pub fn init_contract(){
            
                    }
                ],
                "private" => [ //// array of private methods
                    pub fn get_all_deposits(){

                    }
                ],
                "payable" => [ //// array of payable methods
                    pub fn deposit(){
            
                    }
                ],
                "external" => [ //// array of external methods
                    fn get_address_bytes(){

                    }
                ]
            ]

        }

    */

    // event!{
    //     name: "list_owner",
    //     log: [NewOwner, AddDeposit],

    //     // event methods

    //     fn add_owner(){

    //     } 

    //     fn add_deposit(){
            
    //     }
    // }

    // emit!{
    //     event_name
    // }

    (
     $name:ident, $signer:expr, //// ident can be used to pass struct
     [$($fields:ident: $type:ty),*]; 
     [$($method_type:expr => [$($method:item),*]),* ]
    ) 
     
     => {
            #[near_bindgen]
            #[derive(serde::Deserialize, serde::Serialize)]
            pub struct $name{
                $($fields: $type),*
            }

            impl $name{
                        
                // https://stackoverflow.com/questions/64790850/how-do-i-write-a-macro-that-returns-the-implemented-method-of-a-struct-based-on
                // TODO - implement methods here 
                // ...
            }
    }
}