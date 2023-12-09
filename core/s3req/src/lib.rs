




use diesel::r2d2::ConnectionManager;
use redis_async::client::PubsubConnection;
use actix::Addr;
use mongodb::Client;
use actix_redis::RedisActor;
use sqlx::Postgres;
use std::sync::Arc;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use redis::Client as RedisClient;
use uuid::Uuid;


/*  ----------------------
   | shared state storage 
   |----------------------
   | redis
   | redis async
   | redis actor
   | mongodb
   | diesel postgres
   | tokio postgres
   |
*/


#[derive(Clone)] // can't bound Copy trait cause engine and url are String which are heap data structure 
pub struct Db{
    pub mode: Mode,
    pub engine: Option<String>,
    pub url: Option<String>,
    pub instance: Option<Client>,
    pub pool: Option<Pool<ConnectionManager<PgConnection>>>,
    pub redis: Option<RedisClient>,
    pub redis_async_pubsub_conn: Option<Arc<PubsubConnection>>,
    pub redis_actix_actor: Option<Addr<RedisActor>>,
    pub sqlx_pg_listener: Option<Arc<tokio::sync::Mutex<sqlx::postgres::PgListener>>>,
}

impl Default for Db{
    fn default() -> Db {
        Db{
            mode: self::Mode::Off,
            engine: None,
            url: None,
            instance: None,
            pool: None, // pg pool
            redis: None,
            redis_async_pubsub_conn: None,
            redis_actix_actor: None,
            sqlx_pg_listener: None
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
                pool: None, // pg pool
                redis: None,
                redis_async_pubsub_conn: None,
                redis_actix_actor: None,
                sqlx_pg_listener: None
            }
        )
    }
    /* 
        don't call a method which has self (not &self) as it's first 
        param since by call it on the instance the instance will be 
        dropped from the ram move borrowed form of the type in most 
        cases unless its pointer is a shared pointer in which we 
        must deref it using * or clone
        
        Client object uses std::sync::Arc internally, so it can safely be 
        shared across threads or async tasks like tokio::spawn(async move{}) 
        green threads also it is highly recommended to create a single 
        Client and persist it for the lifetime of your application.
    */
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

#[derive(Clone, Default)]
pub struct Storage{
    pub id: Uuid,
    pub db: Option<Db>, // we could have no db at all
}

impl Storage{

    /* 
        since unwrap() takes the ownership of the instance, because 
        it doesn't have &self in its first param, it has self, thus
        we must call as_ref() on the instance before using it to return 
        a reference to the instance to take the ownership of the referenced
        instance by using the unwrap()

        also we've used the lifetime of self param to return a reference to 
        mongodb Client, pg Pool and redis Client objects
    */
    
    pub async fn get_mongodb(&self) -> Option<&Client>{
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().instance.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Client> or Option<&T>
            Mode::Off => None, // no storage is available cause it's off
        }
    }

    pub async fn get_pgdb(&self) -> Option<&Pool<ConnectionManager<PgConnection>>>{ // Pool is an structure which takes a generic M which is bounded to ManageConnection trait - return type is a reference to the Pool which is using the self lifetime
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().pool.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Pool<ConnectionManager<PgConnection>>> or Option<&T>
            Mode::Off => None, // no storage is available cause it's off
        }
    }

    pub async fn get_sqlx_pg_listener(&self) -> Option<Arc<tokio::sync::Mutex<sqlx::postgres::PgListener>>>{
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().sqlx_pg_listener.clone(),
            Mode::Off => None,
        }
    }

    pub fn get_sqlx_pg_listener_none_async(&self) -> Option<Arc<tokio::sync::Mutex<sqlx::postgres::PgListener>>>{
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().sqlx_pg_listener.clone(),
            Mode::Off => None,
        }
    }

    pub fn get_pgdb_sync(&self) -> Option<&Pool<ConnectionManager<PgConnection>>>{ // Pool is an structure which takes a generic M which is bounded to ManageConnection trait - return type is a reference to the Pool which is using the self lifetime
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().pool.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Pool<ConnectionManager<PgConnection>>> or Option<&T>
            Mode::Off => None, // no storage is available cause it's off
        }
    }

    pub async fn get_redis(&self) -> Option<&RedisClient>{ /* an in memory data storage */
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().redis.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
            Mode::Off => None, // no storage is available cause it's off
        }
    }

    pub fn get_redis_sync(&self) -> Option<&RedisClient>{ /* an in memory data storage */
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().redis.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
            Mode::Off => None, // no storage is available cause it's off
        }
    }

    pub async fn get_async_redis_pubsub_conn(&self) -> Option<Arc<PubsubConnection>>{ /* an in memory data storage */
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().redis_async_pubsub_conn.clone(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
            Mode::Off => None, // no storage is available cause it's off
        }
    }

    pub fn get_async_redis_pubsub_conn_sync(&self) -> Option<Arc<PubsubConnection>>{ /* an in memory data storage */
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().redis_async_pubsub_conn.clone(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
            Mode::Off => None, // no storage is available cause it's off
        }
    }

    pub async fn get_redis_actix_actor(&self) -> Option<Addr<RedisActor>>{ /* an in memory data storage */
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().redis_actix_actor.clone(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
            Mode::Off => None, // no storage is available cause it's off
        }
    }

    pub fn get_redis_actix_actor_sync(&self) -> Option<Addr<RedisActor>>{ /* an in memory data storage */
        match self.db.as_ref().unwrap().mode{
            Mode::On => self.db.as_ref().unwrap().redis_actix_actor.clone(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
            Mode::Off => None, // no storage is available cause it's off
        }
    }

}

#[derive(Copy, Clone, Debug)]
pub enum Mode{ // enum uses 8 bytes (usize which is 64 bits on 64 bits arch) tag which is a pointer pointing to the current variant - the total size of this enum is 8 bytes tag + the largest variant size = 8 + 0 = 8 bytes; cause in our case On and Off variant both have 0 size
    On, // zero byte size
    Off, // zero byte size
}


#[macro_export]
macro_rules! storage {

    ($name:expr, $engine:expr, $host:expr, $port:expr, $username:expr, $password:expr) => {
                
        async { // this is the key! this curly braces is required to use if let statement, use libs and define let inside macro
            
            use s3req::{Storage, Mode, Db};
            use sqlx::PgPool;

            /* -=-=-=-=-=-=-=-=-=-=-= REDIS SETUP -=-=-=-=-=-=-=-=-=-=-= */

            let redis_password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());
            let redis_username = env::var("REDIS_USERNAME").unwrap_or("".to_string());
            let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
            let redis_port = std::env::var("REDIS_PORT").unwrap_or("6379".to_string()).parse::<u64>().unwrap();
            let redis_actor_conn_url = format!("{redis_host}:{redis_port}");

            let redis_conn_url = if !redis_password.is_empty(){
                format!("redis://:{}@{}:{}", redis_password, redis_host, redis_port)
            } else if !redis_password.is_empty() && !redis_username.is_empty(){
                format!("redis://{}:{}@{}:{}", redis_username, redis_password, redis_host, redis_port)
            } else{
                format!("redis://{}:{}", redis_host, redis_port)
            };

            /* redis async, none async and actor setup */
            let none_async_redis_client = redis::Client::open(redis_conn_url.as_str()).unwrap();
            let redis_actor = RedisActor::start(redis_actor_conn_url.as_str());
            let mut redis_conn_builder = ConnectionBuilder::new(redis_host, redis_port as u16).unwrap();
            redis_conn_builder.password(redis_password);
            let async_redis_pubsub_conn = Arc::new(redis_conn_builder.pubsub_connect().await.unwrap());
            
            /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */

            let pg_url = std::env::var("DATABASE_URL").unwrap();
            let sqlx_pg_listener = sqlx::postgres::PgListener::connect(&pg_url).await.unwrap();
 
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
                                pool: None, // pg pool
                                redis: None,
                                redis_async_pubsub_conn: None,
                                redis_actix_actor: None,
                                sqlx_pg_listener: None,
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
                                            pool: None, // pg pool
                                            redis: Some(none_async_redis_client.clone()),
                                            redis_async_pubsub_conn: Some(async_redis_pubsub_conn.clone()),
                                            redis_actix_actor: Some(redis_actor.clone()),
                                            sqlx_pg_listener: Some(Arc::new(tokio::sync::Mutex::new(sqlx_pg_listener)))
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
                                            redis: Some(none_async_redis_client.clone()),
                                            redis_async_pubsub_conn: Some(async_redis_pubsub_conn.clone()),
                                            redis_actix_actor: Some(redis_actor.clone()),
                                            sqlx_pg_listener: Some(Arc::new(tokio::sync::Mutex::new(sqlx_pg_listener)))
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