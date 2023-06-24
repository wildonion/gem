


use crate::*;
use crate::constants::CHARSET;


pub fn gen_chars(size: u32) -> String{
    let mut rng = rand::thread_rng();
    (0..size).map(|_|{
        char::from_u32(rng.gen_range(33..126)).unwrap() // generating a char from the random output of type u32 using from_u32() method
    }).collect()
}

pub fn gen_random_number(from: u32, to: u32) -> u32{
    let mut rng = rand::thread_rng(); // we can't share this between threads and across .awaits
    rng.gen_range(from..to)
} 

pub fn gen_random_idx(idx: usize) -> usize{
    if idx < CHARSET.len(){
        idx
    } else{
        gen_random_idx(random::<u8>() as usize)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Keys{
    pub twitter_bearer_token: String,
    pub twitter_access_token: String,
    pub twitter_access_token_secret: String,
    pub twitter_consumer_key: String,
    pub twitter_consumer_secret: String,
    pub twitter_api_key: String,
    pub twitter_api_secret: String
}


#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct TwitterAccounts{
    pub keys: Vec<Keys>
}

// inspire complex macro syntax => inside https://github.com/wildonion/uniXerr/blob/master/infra/valhalla/coiniXerr/src/utils.rs

#[derive(Clone)] // can't bound Copy trait cause engine and url are String which are heap data structure 
pub struct Db{
    pub mode: Mode,
    pub engine: Option<String>,
    pub url: Option<String>,
    pub instance: Option<Client>,
    pub pool: Option<Pool<ConnectionManager<PgConnection>>>,
    pub redis: Option<RedisClient>,
}

impl Default for Db{
    fn default() -> Db {
        Db{
            mode: self::Mode::Off,
            engine: None,
            url: None,
            instance: None,
            pool: None,
            redis: None
        }
    }
}

impl Db{
    
    pub async fn new() -> Result<Db, Box<dyn std::error::Error>>{
        Ok(
            Db{ // building an instance with generic type C which is the type of the db client instance
                mode: Mode::On, // 1 means is on 
                engine: None, 
                url: None,
                instance: None,
                pool: None,
                redis: None,
            }
        )
    }
    
    // don't call a method which has self (not &self) as it's first 
    // param since by call it on the instance the instance will be 
    // dropped from the ram move borrowed form of the type in most 
    // cases unless its pointer is a shared pointer in which we 
    // must deref it using * or clone
    //
    // Client object uses std::sync::Arc internally, so it can safely be 
    // shared across threads or async tasks like tokio::spawn(async move{}) 
    // green threads also it is highly recommended to create a single 
    // Client and persist it for the lifetime of your application.
    pub async fn GetMongoDbInstance(&self) -> Client{ // it'll return an instance of the mongodb client - we set the first argument to &self in order to have the instance of the object later on after calling this method and prevent ownership moving
        Client::with_uri_str(self.url.as_ref().unwrap()).await.unwrap() // building mongodb client instance
    }

    pub async fn GetPostgresPool(&self) -> Pool<ConnectionManager<PgConnection>>{
        let uri = self.url.as_ref().unwrap().as_str();
        let manager = ConnectionManager::<PgConnection>::new(uri);
        let pool = Pool::builder().test_on_check_out(true).build(manager).unwrap();
        pool
    }

}

#[derive(Clone)]
pub struct Storage{
    pub id: Uuid,
    pub db: Option<Db>, // we could have no db at all
}

impl Storage{
    pub async fn get_mongodb(&self) -> Option<&Client>{
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().instance.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Client> or Option<&T>
            Mode::Off => None, // no db is available cause it's off
        }
    }
    pub async fn get_pgdb(&self) -> Option<&Pool<ConnectionManager<PgConnection>>>{ // Pool is an structure which takes a generic M which is bounded to ManageConnection trait
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().pool.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Pool<ConnectionManager<PgConnection>>> or Option<&T>
            Mode::Off => None, // no db is available cause it's off
        }
    }
    pub async fn get_redis(&self) -> Option<&RedisClient>{ /* an in memory data storage */
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().redis.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
            Mode::Off => None, // no db is available cause it's off
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Mode{ // enum uses 8 bytes (usize which is 64 bits on 64 bits arch) tag which is a pointer pointing to the current variant - the total size of this enum is 8 bytes tag + the largest variant size = 8 + 0 = 8 bytes; cause in our case On and Off variant both have 0 size
    On, // zero byte size
    Off, // zero byte size
}


/*
    can't bound the T to ?Sized since 
    T is inside the Option which the size
    of the Option depends on the T at 
    compile time hence the T must be 
    Sized, also we're using a lifetime 
    to use the str slices in message

    in the case of passing &[] data we must 
    specify the type of T and pass the type to 
    the Response signature like Response::<&[Type]>{} 
    since the size of &[] can't be known 
    at compile time hence we must specify the exact
    type of T inside &[]    

*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Response<'m, T>{
    pub data: Option<T>,
    pub message: &'m str, // &str are a slice of String thus they're behind a pointer and every pointer needs a valid lifetime which is 'm in here 
    pub status: u16,
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
        $data_type:ty,
        $data:expr,
        $msg:expr,
        $code:expr,
        $cookie:expr,
    ) => {

        {
            use actix_web::HttpResponse;
            use crate::misc::Response;
            
            let code = $code.as_u16();
            let mut res = HttpResponse::build($code);
            
            let response_data = Response::<$data_type>{
                data: Some($data),
                message: $msg,
                status: code
            };
            
            let resp = if let Some(cookie) = $cookie{
                res
                    .cookie(cookie.clone())
                    .append_header(("cookie", cookie.value()))
                    .json(
                        response_data
                    )
            } else{
                res
                    .json(
                        response_data
                    )
            }; 

            return Ok(resp);
        }
    }
}


#[macro_export]
macro_rules! server {
    (
        
        /* ... setup args here ... */

    ) => {
        
        {

            use std::env;
            use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse, get, ResponseError};
            use actix_web::middleware::Logger;
            use dotenv::dotenv;
            
            env::set_var("RUST_LOG", "trace");
            // env::set_var("RUST_LOG", "actix_web=debug");
            dotenv().expect("⚠️ .env file not found");
            env_logger::init_from_env(Env::default().default_filter_or("info"));
            let host = std::env::var("HOST").expect("⚠️ no host variable set");
            let port = std::env::var("PANEL_PORT").expect("⚠️ no panel port variable set").parse::<u16>().unwrap();
            let db_host = env::var("DB_HOST").expect("⚠️ no db host variable set");
            let db_port = env::var("DB_PORT").expect("⚠️ no db port variable set");
            let db_username = env::var("DB_USERNAME").expect("⚠️ no db username variable set");
            let db_password = env::var("DB_PASSWORD").expect("⚠️ no db password variable set");
            let db_engine = env::var("DB_ENGINE").expect("⚠️ no db engine variable set");
            let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");
            let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");

            let app_storage = db!{ // this publicly has exported inside the misc so we can access it here 
                db_name,
                db_engine,
                db_host,
                db_port,
                db_username,
                db_password
            }.await;

            let shared_storage = Data::new(app_storage.clone());
    
            /*
                the HttpServer::new function takes a factory function that 
                produces an instance of the App, not the App instance itself. 
                This is because each worker thread needs to have 
                its own App instance.

                handle streaming async tasks like socket connections in a none blocking
                manner asyncly and concurrently using tokio::spawn(async move{}) and 
                shared state data between tokio::spawn() green threadpool using jobq channels 
                and clusters using redis and routers' threads using arc, mutex and rwlock 
                also data must be Send + Sync + 'static also handle incoming async 
                events into the server using tokio::select!{} eventloop. 

                we're sharing the db_instance and redis connection state between 
                routers' threads to get the data inside each api also for this the 
                db and redis connection data must be shareable and safe to send 
                between threads which must be bounded to Send + Sync traits 

                since every api or router is an async task that must be handled 
                inside the hyper threads thus the data that we want to use inside 
                of them and share it between other routers must be 
                Arc<Mutex<Data>> + Send + Sync + 'static 

                mongodb and redis connection instances must be only Arc (shareable)
                to share them between threads since we don't want to mutate them 
                in actix routers' threads. 
            */
            HttpServer::new(move ||{
                App::new()
                    /* 
                        APP STORAGE SHARED STATE
                    */
                    .app_data(Data::clone(&shared_storage.clone()))
                    .wrap(Cors::permissive())
                    .wrap(Logger::default())
                    .wrap(Logger::new("%a %{User-Agent}i %t %P %r %s %b %T %D"))
                    /*
                        INIT DEV SERIVE APIs 
                    */
                    .service(
                        actix_web::web::scope("/dev")
                            .configure(services::init_dev)   
                    )
                    /*
                        INIT ADMIN SERIVE APIs
                    */
                    .service(
                        actix_web::web::scope("/admin")
                            .configure(services::init_admin)
                    )
                    /*
                        INIT USER SERIVE APIs 
                    */
                    .service(
                        actix_web::web::scope("/user")
                            .configure(services::init_user)
                    )
                    /*
                        INIT HEALTH SERIVE
                    */
                    .service(
                        actix_web::web::scope("/health")
                            .configure(services::init_health)
                    )
                    /*
                        INIT BOT SERIVE
                    */
                    .service(
                        actix_web::web::scope("/bot")
                            .configure(services::init_bot)
                    )
                    /*
                        INIT SWAGGER UI SERIVES
                    */
                    .service(SwaggerUi::new("/swagger/{_:.*}").urls(vec![
                        (
                            Url::new("admin", "/api-docs/admin.json"),
                            apis::admin::AdminApiDoc::openapi(),
                        ),
                        (
                            Url::new("dev", "/api-docs/dev.json"),
                            apis::dev::DevApiDoc::openapi(),
                        ),
                        (
                            Url::new("user", "/api-docs/user.json"),
                            apis::user::UserApiDoc::openapi(),
                        ),
                        (
                            Url::new("health", "/api-docs/health.json"),
                            apis::health::HealthApiDoc::openapi(),
                        ),
                        (
                            Url::new("bot", "/api-docs/bot.json"),
                            apis::bot::BotApiDoc::openapi(),
                        )
                    ]))
                }) // each thread of the HttpServer instance needs its own app factory 
                .bind((host.as_str(), port))
                .unwrap()
                .workers(10)
                .run()
                .await

        }
    };
}

#[macro_export]
macro_rules! db {

    ($name:expr, $engine:expr, $host:expr, $port:expr, $username:expr, $password:expr) => {
                
        async { // this is the key! this curly braces is required to use if let statement, use libs and define let inside macro
            
            use crate::misc::*;

            /* -=-=-=-=-=-=-=-=-=-=-= REDIS SETUP -=-=-=-=-=-=-=-=-=-=-= */

            let redis_password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());
            let redis_username = env::var("REDIS_USERNAME").unwrap_or("".to_string());
            let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
            let redis_port = std::env::var("REDIS_PORT").unwrap_or("6379".to_string()).parse::<u64>().unwrap();

            let redis_conn_url = if !redis_password.is_empty(){
                format!("redis://:{}@{}:{}", redis_password, redis_host, redis_port)
            } else if !redis_password.is_empty() && !redis_username.is_empty(){
                format!("redis://{}:{}@{}:{}", redis_username, redis_password, redis_host, redis_port)
            } else{
                format!("redis://{}:{}", redis_host, redis_port)
            };

            let client = redis::Client::open(redis_conn_url.as_str()).unwrap();
            
            /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */

            
            let empty_app_storage = Some( // putting the Arc-ed db inside the Option
                Arc::new( // cloning app_storage to move it between threads
                    Storage{ // defining db context 
                        id: Uuid::new_v4(),
                        db: Some(
                            Db{
                                mode: Mode::Off,
                                instance: None,
                                engine: None,
                                url: None,
                                pool: None,
                                redis: None
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
                    Ok(mut init_db) => { // init_db instance must be mutable since we want to mutate its fields
                        init_db.engine = Some($engine);
                        init_db.url = Some(db_addr);
                        let mongodb_instance = init_db.GetMongoDbInstance().await; // the first argument of this method must be &self in order to have the init_db instance after calling this method, cause self as the first argument will move the instance after calling the related method and we don't have access to any field like init_db.url any more due to moved value error - we must always use & (like &self and &mut self) to borrotw the ownership instead of moving
                        Some( // putting the Arc-ed db inside the Option
                            Arc::new( // cloning app_storage to move it between threads
                                Storage{ // defining db context 
                                    id: Uuid::new_v4(),
                                    db: Some(
                                        Db{
                                            mode: init_db.mode,
                                            instance: Some(mongodb_instance),
                                            engine: init_db.engine,
                                            url: init_db.url,
                                            pool: None,
                                            redis: Some(client.clone())
                                        }
                                    ),
                                }
                            )
                        )
                    },
                    Err(e) => {
                        error!("😕 init db error - {}", e);
                        empty_app_storage // whatever the error is we have to return and empty app storage instance 
                    }
                }
            } else if $engine.as_str() == "postgres"{
                info!("➔ 🛢️ switching to postgres on address: [{}:{}]", $host, $port);
                let environment = env::var("ENVIRONMENT").expect("⚠️ no environment variable set");                
                let db_addr = if environment == "dev"{
                    format!("{}://{}:{}", $engine, $host, $port)
                } else if environment == "prod"{
                    format!("{}://{}:{}@{}:{}/{}", $engine, $username, $password, $host, $port, $name)
                } else{
                    "".to_string()
                };
                match Db::new().await{
                    Ok(mut init_db) => { // init_db instance must be mutable since we want to mutate its fields
                        init_db.engine = Some($engine);
                        init_db.url = Some(db_addr);
                        let pg_pool = init_db.GetPostgresPool().await; // the first argument of this method must be &self in order to have the init_db instance after calling this method, cause self as the first argument will move the instance after calling the related method and we don't have access to any field like init_db.url any more due to moved value error - we must always use & (like &self and &mut self) to borrotw the ownership instead of moving
                        Some( // putting the Arc-ed db inside the Option
                            Arc::new( // cloning app_storage to move it between threads
                                Storage{ // defining db context 
                                    id: Uuid::new_v4(),
                                    db: Some(
                                        Db{
                                            mode: init_db.mode,
                                            instance: None,
                                            engine: init_db.engine,
                                            url: init_db.url,
                                            pool: Some(pg_pool),
                                            redis: Some(client.clone())
                                        }
                                    ),
                                }
                            )
                        )
                    },
                    Err(e) => {
                        error!("😕 init db error - {}", e);
                        empty_app_storage // whatever the error is we have to return and empty app storage instance 
                    }
                }
            } else{
                empty_app_storage
            };

            app_storage // returning the created app_storage

        }
    };

}

#[macro_export]
macro_rules! passport {
    (
      $token:expr
    ) 
    => {

        { // this is required if we want to import modules and use the let statements
            
            use std::env;

            let host = env::var("HOST").expect("⚠️ no host variable set");
            let port = env::var("CONSE_PORT").expect("⚠️ no port variable set");
            let check_token_api = format!("{}:{}/auth/check-token", host, port);
            
            let mut response_value: serde_json::Value = reqwest::Client::new()
                        .post(check_token_api.as_str())
                        .header("Authorization", $token)
                        .send()
                        .await.unwrap()
                        .json()
                        .await.unwrap();

            let msg = response_value["message"].take();
            if msg == serde_json::json!("Access Granted"){
                true
            } else{
                false
            }
            
        }
    }
}

#[macro_export]
macro_rules! verify {
    (
      $endpoint:expr,
      $body:expr,
      $task_id:expr,
      $doer_id:expr,
      $connection:expr,
      $redis_client:expr,
      $task_type:expr,
      $tusername:expr,
      $tweet_link:expr
    ) 
    => {

        { // this is required if we want to import modules and use the let statements

            use crate::models::bot::Twitter;

            info!("🤖 sending request to the twitter bot hosted on [{:#?}]", $endpoint);
            let response_value: serde_json::Value = reqwest::Client::new()
                .post($endpoint)
                .json(&$body)
                .send()
                .await.unwrap()
                .json()
                .await.unwrap();

            /* publishing the twitter bot response to the redis pubsub channel */
            info!("📢 publishing twitter bot response to redis pubsub [twitter-bot-response] channel");

            let mut conn = $redis_client.get_async_connection().await.unwrap();   
            let _: Result<_, RedisError> = conn.publish::<String, String, String>("twitter-bot-response".to_string(), response_value.to_string()).await;

            /* I believe that the bot code has some shity response structure :) since I didn't designed it*/

            let data_field = response_value.get("data");
            if data_field.is_some(){
                let status = data_field.unwrap().get("status");
                if status.is_some(){

                    let bool_status = status.unwrap().to_string();
                    if bool_status == "false"{

                        /* twitter error */

                        match diesel::delete(users_tasks
                            .filter(users_tasks::task_id.eq($task_id)))
                            .filter(users_tasks::user_id.eq($doer_id))
                            .execute($connection)
                            {
                                Ok(num_deleted) => {
                                    
                                    if num_deleted > 0{
            
                                        let resp = Response::<&[u8]>{
                                            data: Some(&[]),
                                            message: TASK_NOT_VERIFIED,
                                            status: 406
                                        };
                                        return Ok(
                                            HttpResponse::NotAcceptable().json(resp)
                                        );                                
            
                                    } else{
                                        
                                        let resp = Response::<&[u8]>{
                                            data: Some(&[]),
                                            message: USER_TASK_HAS_ALREADY_BEEN_DELETED,
                                            status: 417
                                        };
                                        return Ok(
                                            HttpResponse::ExpectationFailed().json(resp)
                                        ); 
            
                                    }
                                
                                },
                                Err(e) => {
            
                                    let resp = Response::<&[u8]>{
                                        data: Some(&[]),
                                        message: &e.to_string(),
                                        status: 500
                                    };
                                    return Ok(
                                        HttpResponse::InternalServerError().json(resp)
                                    );
            
                                }
                            }

                    } else{

                        /* task is verified by twitter */

                        match UserTask::find($doer_id, $task_id, $connection).await{
                            false => {

                                /* try to insert into users_tasks since it's done */
                                let res = Twitter::do_task($doer_id, $task_id, $tusername, $task_type, $tweet_link, $connection).await;
                                return res;
                            },
                            _ => {
        
                                /* user task has already been inserted  */
                                let resp = Response::<&[u8]>{
                                    data: Some(&[]),
                                    message: USER_TASK_HAS_ALREADY_BEEN_INSERTED,
                                    status: 302
                                };
                                return Ok(
                                    HttpResponse::Found().json(resp)
                                );
        
                            }
                        }

                    }
                } else{

                    /* twitter rate limit issue */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: TWITTER_RATE_LIMIT,
                        status: 406
                    };
                    return Ok(
                        HttpResponse::NotAcceptable().json(resp)
                    );  
                
                }
            } else{

                /* twitter rate limit issue */

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: TWITTER_RATE_LIMIT,
                    status: 406
                };
                return Ok(
                    HttpResponse::NotAcceptable().json(resp)
                );  
            }
        }
    }
}