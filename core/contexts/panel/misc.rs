

use crate::*;



// inspire complex macro syntax => inside https://github.com/wildonion/uniXerr/blob/master/infra/valhalla/coiniXerr/src/utils.rs


#[derive(Clone)] //// can't bound Copy trait cause engine and url are String which are heap data structure 
pub struct Db{
    pub mode: Mode,
    pub engine: Option<String>,
    pub url: Option<String>,
    pub instance: Option<Client>,
    pub pool: Option<Pool<ConnectionManager<PgConnection>>>
}

impl Default for Db{
    fn default() -> Db {
        Db{
            mode: self::Mode::Off,
            engine: None,
            url: None,
            instance: None,
            pool: None
        }
    }
}

impl Db{
    
    pub async fn new() -> Result<Db, Box<dyn std::error::Error>>{
        Ok(
            Db{ //// building an instance with generic type C which is the type of the db client instance
                mode: Mode::On, //// 1 means is on 
                engine: None, 
                url: None,
                instance: None,
                pool: None
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
    pub db: Option<Db>, //// we could have no db at all
}

impl Storage{
    pub async fn get_mongodb(&self) -> Option<&Client>{
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().instance.as_ref(), //// return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Client> or Option<&T>
            Mode::Off => None, //// no db is available cause it's off
        }
    }
    pub async fn get_pgdb(&self) -> Option<&Pool<ConnectionManager<PgConnection>>>{
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().pool.as_ref(), //// return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Pool<ConnectionManager<PgConnection>>> or Option<&T>
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
            
            let response = Ok(
                res
                    .json(
                        response_data
                    )
            );
            return response;
        }
    }
}


#[macro_export]
macro_rules! server {
    (
        
        /* ... args here ... */

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
            let redis_password = env::var("REDIS_PASSWORD").expect("⚠️ no redis password variable set");
            let redis_host = std::env::var("REDIS_HOST").expect("⚠️ no redis host variable set");
            let redis_conn_url = format!("redis://{}@{}", redis_password, redis_host);
            let client = redis::Client::open(redis_conn_url.as_str()).unwrap();

            let app_storage = db!{ //// this publicly has exported inside the misc so we can access it here 
                db_name,
                db_engine,
                db_host,
                db_port,
                db_username,
                db_password
            };

            let shared_arced_redis_conn = Data::new(client.clone());
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
                        REDIS SHARED STATE
                    */
                    .app_data(Data::clone(&shared_arced_redis_conn))
                    /* 
                        MONGODB SHARED STATE
                    */
                    .app_data(Data::clone(&shared_storage))
                    .wrap(
                        Cors::default()
                            .allow_any_origin()
                            .allowed_methods(vec!["GET", "POST"])
                            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                            .allowed_header(header::CONTENT_TYPE)
                            .max_age(3600)
                    )
                    .wrap(Logger::default())
                    .wrap(Logger::new("%a %{User-Agent}i %t %P %r %s %b %T %D"))
                    /*
                        DEV PUSH NOTIF REGISTERATION SERIVE 
                    */
                    .service(
                        actix_web::web::scope("/panel/api/dev")
                            .configure(services::init_dev)   
                    )
                    /*
                        ADMIN PUSH NOTIF REGISTERATION SERIVE 
                    */
                    .service(
                        actix_web::web::scope("/panel/api/admin")
                            .configure(services::init_admin)
                    )
                    /*
                        HEALTH SERIVE
                    */
                    .service(
                        actix_web::web::scope("/panel/api/health")
                            .configure(services::init_health)
                    )
                    /*
                        MMQ SERIVE
                    */
                    .service(
                        actix_web::web::scope("/panel/api/mmq")
                            .configure(services::init_mmq)
                    )
                }) //// each thread of the HttpServer instance needs its own app factory 
                .bind((host.as_str(), port))
                .unwrap()
                .workers(10)
                .run()
                .await
        };
    };
}

#[macro_export]
macro_rules! db {

    ($name:expr, $engine:expr, $host:expr, $port:expr, $username:expr, $password:expr) => {
                
        { //// this is the key! this curly braces is required to use if let statement, use libs and define let inside macro
            
            use crate::misc::*;
            
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
                                pool: None
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
                                            pool: None
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
                let db_addr = if environment == "dev"{
                    format!("{}://{}:{}", $engine, $host, $port)
                } else if environment == "prod"{
                    format!("{}://{}:{}@{}:{}/{}", $engine, $username, $password, $host, $port, $name)
                } else{
                    "".to_string()
                };
                match Db::new().await{
                    Ok(mut init_db) => { //// init_db instance must be mutable since we want to mutate its fields
                        init_db.engine = Some($engine);
                        init_db.url = Some(db_addr);
                        let pg_pool = init_db.GetPostgresPool().await; //// the first argument of this method must be &self in order to have the init_db instance after calling this method, cause self as the first argument will move the instance after calling the related method and we don't have access to any field like init_db.url any more due to moved value error - we must always use & (like &self and &mut self) to borrotw the ownership instead of moving
                        Some( //// putting the Arc-ed db inside the Option
                            Arc::new( //// cloning app_storage to move it between threads
                                Storage{ //// defining db context 
                                    id: Uuid::new_v4(),
                                    db: Some(
                                        Db{
                                            mode: init_db.mode,
                                            instance: None,
                                            engine: init_db.engine,
                                            url: init_db.url,
                                            pool: Some(pg_pool)
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
      $token:expr
    ) 
    => {

        { //// this is required if we want to import modules and use the let statements
            
            let host = std::env::var("HOST").expect("⚠️ no host variable set");
            let port = std::env::var("CONSE_PORT").expect("⚠️ no port variable set");
            let check_token_api = format!("{}:{}/auth/check-token", host, port);
            let mut resp: Result<actix_web::HttpResponse, actix_web::Error>;

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
    