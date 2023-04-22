


use std::io::Write;
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

use crate::*;





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





pub async fn activate_discord_bot(discord_token: &str, serenity_shards: u64, gpt: ctx::gpt::chat::Gpt){

    //// each shard is a ws client to the discrod ws server also discord 
    //// requires that there be at least one shard for every 2500 guilds 
    //// (discrod servers) that a bot is on.
    //
    //// data of each bot client must be safe to send between other shards' 
    //// threads means they must be Arc<Mutex<Data>> + Send + Sync + 'static
    //// or an RwLock type also each shard must be Arced and Mutexed to 
    //// be shareable between threads.

    let http = http::Http::new(&discord_token);
    let (owners, _bot_id) = match http.get_current_application_info().await{ //// fetching bot owner and id, application id is the id of the created http channel
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (Some(owners), Some(bot_id.id)),
                Err(why) => {
                    error!("😖 Could not access the bot id: {:?}", why);
                    (None, None)
                },
            }
        },
        Err(why) => {
            error!("😖 could not access discord bot application info: {:?}", why);
            (None, None)
        },
    };
    if owners.is_some(){
        let framework = StandardFramework::new()
                                                .configure(|c| 
                                                    c
                                                        .on_mention(_bot_id)
                                                        .owners(owners.unwrap())
                                                );
        ///// gateway intents are predefined ws events 
        let intents = GatewayIntents::all(); //// all the gateway intents must be on inside the https://discord.com/developers/applications/1092048595605270589/bot the privileged gateway intents section
        let mut bot_client = BotClient::builder(discord_token, intents)
                                                        .framework(framework)
                                                        .event_handler(ctx::bot::handler::Handler)
                                                        .await
                                                        .expect("😖 in creating discord bot client");
        {   
            //// building a new chat GPT instance for our summerization process
            //// it must be Send to be shared and Sync or safe to move it between 
            //// shards' and command handlers' threads 
            let gpt_instance_cloned_mutexed = Arc::new(Mutex::new(gpt.clone())); //// Mutex is inside the tokio::sync
            //// since we want to borrow the bot_client as immutable we must define 
            //// a new scope to do this because if a mutable pointer exists 
            //// an immutable one can't be there otherwise we get this Error:
            //// cannot borrow `bot_client` as mutable because it is also borrowed as immutable
            let mut data = bot_client.data.write().await; //// data of the bot client is of type RwLock which can be written safely in other threads
            data.insert::<ctx::bot::handler::GptBot>(gpt_instance_cloned_mutexed.clone()); //// writing the GPT bot instance into the data variable of the bot client to pass it between shards' threads 
            data.insert::<ctx::bot::handler::ShardManagerContainer>(bot_client.shard_manager.clone()); //// writing a cloned shard manager inside the bot client data
        }
        //// moving the shreable shard (Arc<Mutex<ShardManager>>) 
        //// into tokio green threadpool to check all the shards status
        let shard_manager = bot_client.shard_manager.clone(); //// each shard is an Arced Mutexed data that can be shared between other threads safely
        tokio::spawn(async move{
            tokio::signal::ctrl_c().await.expect("😖 install the plugin CTRL+C signal to the server");
            shard_manager.lock().await.shutdown_all().await; //// once we received the ctrl + c we'll shutdown all shards or ws clients 
            //// we'll print the current statuses of the two shards to the 
            //// terminal every 30 seconds. This includes the ID of the shard, 
            //// the current connection stage, (e.g. "Connecting" or "Connected"), 
            //// and the approximate WebSocket latency (time between when a heartbeat 
            //// is sent to discord and when a heartbeat acknowledgement is received),
            //// note that it may take a minute or more for a latency to be recorded or to
            //// update, depending on how often Discord tells the client to send a heartbeat.
            loop{ //// here we're logging the shard status every 30 seconds
                tokio::time::sleep(Duration::from_secs(30)).await; //// wait for 30 seconds heartbeat of course it depends on the discord ws server of the heartbeat response
                let lock = shard_manager.lock().await;
                let shard_runners = lock.runners.lock().await;
                for (id, runner) in shard_runners.iter(){
                    info!(
                        "🧩 shard with ID {} is {} with a latency of {:?}",
                        id, runner.stage, runner.latency,
                    );
                }
            }
        });
        //// start the bot client with 2 shards or ws clients for listening
        //// for events, there is an ~5 second ratelimit period
        //// between when one shard can start after another.
        if let Err(why) = bot_client.start_shards(serenity_shards).await{
            error!("😖 discord bot client error: {:?}", why);
        }
    }

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
        
                // TODO 
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
      $access:expr
    ) 
    => {

        { //// this is required if we want to import modules and use the let statements
            use crate::middlewares;
            use crate::contexts as ctx;
            use hyper::{header, StatusCode, Body, Response};
            
            let res = Response::builder();
            let db = &$req.data::<Client>().unwrap().to_owned();
            let pass = middlewares::auth::pass($req).await;

            if pass.is_ok(){
                
                let (token_data, req) = pass.unwrap(); //// the decoded token and the request object will be returned from the function call since the Copy and Clone trait is not implemented for the hyper Request and Response object thus we can't have the borrowed form of the req object by passing it into the pass() function therefore it'll be moved and we have to return it from the pass() function
                let _id = token_data.claims._id;
                let username = token_data.claims.username.clone();
                let access_level = token_data.claims.access_level;
                if middlewares::auth::user::exists(Some(&db.clone()), _id, username.clone(), access_level).await{ //// finding the user with these info extracted from jwt
                    if access_level == $access{ //// the passed in access_level must be equals with the one decoded one inside the JWT  
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
                        let response_body = ctx::app::Response::<ctx::app::Nill>{
                            data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
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
                    let response_body = ctx::app::Response::<ctx::app::Nill>{ //// we have to specify a generic type for data field in Response struct which in our case is Nill struct
                        data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
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
                let response_body = ctx::app::Response::<ctx::app::Nill>{
                    data: Some(ctx::app::Nill(&[])), //// data is an empty &[u8] array
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
        };
    }
}

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
            use crate::contexts as ctx;

            let code = $code.as_u16(); 
            let res = Response::builder();
            let response_body = ctx::app::Response::<$data_type>{
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