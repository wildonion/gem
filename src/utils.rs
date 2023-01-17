


use std::io::Write;
use std::sync::Mutex;
use std::sync::{Arc, mpsc::channel as heavy_mpsc, mpsc}; use std::time::{SystemTime, UNIX_EPOCH}; // NOTE - mpsc means multiple thread can access the Arc<Mutex<T>> (use Arc::new(&Arc<Mutex<T>>) to clone the arced and mutexed T which T can also be Receiver<T>) but only one of them can mutate the T out of the Arc by locking on the Mutex
use std::{env, thread, fs}; 
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
use crate::contexts::app;
use serde::{Serialize, Deserialize};
use borsh::{BorshDeserialize, BorshSerialize};
use routerify_multipart::Multipart;
use hyper::{Client as HyperClient, Response, Body, Uri, Server, server::conn::AddrIncoming};
use async_trait::async_trait;
use std::net::SocketAddr;
use routerify::{RouterService, Router};








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
            let sms_response_stream = block_on(client.get(uri)).unwrap(); //// since we can't use .await inside trait methods thus we have to solve the future using block_on() function
            
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
pub fn async_gen_random_idx(idx: usize) -> BoxFuture<'static, usize>{ // NOTE - pub type BoxFuture<'a, T> = Pin<alloc::boxed::Box<dyn Future<Output = T> + Send + 'a>>
    async move{
        if idx <= CHARSET.len(){
            idx
        } else{
            gen_random_idx(random::<u8>() as usize)
        }
    }.boxed() //// wrap the future in a Box, pinning it
}





pub fn gen_random_idx(idx: usize) -> usize{
    if idx < CHARSET.len(){
        idx
    } else{
        gen_random_idx(random::<u8>() as usize)
    }
}





pub async fn upload_asset(path: &str, mut payload: Multipart<'_>, doc_id: &String) -> Option<String>{ //// parsing the incoming file stream into MultipartItem instances - Multipart struct takes a lifetime and we've passed an unnamed lifetime to that
    
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












// ------------------------------ macros
// -----------------------------------------------------------------------------------------
// -----------------------------------------------------------------------------------------
// -----------------------------------------------------------------------------------------

#[macro_export]
macro_rules! db {

    ($name:expr, $engine:expr, $host:expr, $port:expr, $username:expr, $password:expr) => {
                
        { //// this is the key! this curly braces is required to use if let statement, use libs and define let inside macro
            
            use crate::contexts::app::*;
            
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
                info!("➔ 🛢️ switching to mongodb");
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
                let environment = env::var("ENVIRONMENT").expect("⚠️ no environment variable set");                
                if environment == "dev"{
                    format!("{}://{}:{}", $engine, $host, $port)
                } else if environment == "prod"{
                    format!("{}://{}:{}@{}:{}", $engine, $username, $password, $host, $port)
                } else{
                    "".to_string()
                };
        
                // TODO 
                todo!();
            
            } else{
                empty_app_storage
            };

            app_storage //// returning the created app_storage

        }
    };

}