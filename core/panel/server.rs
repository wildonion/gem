



use crate::*; /* this includes all things in app.rs specially s3 which contains the storage! macro */



#[macro_export]
macro_rules! server {
    (
        
        /* ... setup args go here ... */
        $tcp_listener:expr

    ) => {
        
        {

            use std::env;
            use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse, get, ResponseError};
            use actix_web::middleware::Logger;
            use dotenv::dotenv;
            use crate::constants::*;
            use crate::events::subscribers::handlers::actors::ws::servers::role::RoleNotifServer;
            use crate::events::subscribers::handlers::actors::ws::servers::mmr::MmrNotifServer;
            use crate::events::subscribers::handlers::actors::ws::servers::ecq::EcqNotifServer;
            use crate::events::subscribers::handlers::actors::ws::servers::chatroomlp::ChatRoomLaunchpadServer;
            use crate::events::subscribers::handlers::actors::notif::pg::PgListenerActor;
            use crate::events::subscribers::handlers::actors::notif::system::SystemActor;

            
            env::set_var("RUST_LOG", "trace");
            // env::set_var("RUST_LOG", "actix_web=debug");
            env_logger::init_from_env(Env::default().default_filter_or("info"));
            let host = std::env::var("HOST").expect("⚠️ no host variable set");
            let port = std::env::var("PANEL_PORT").expect("⚠️ no panel port variable set").parse::<u16>().unwrap();
            let db_host = env::var("DB_HOST").expect("⚠️ no db host variable set");
            let db_port = env::var("DB_PORT").expect("⚠️ no db port variable set");
            let db_username = env::var("DB_USERNAME").expect("⚠️ no db username variable set");
            let db_password = env::var("DB_PASSWORD").expect("⚠️ no db password variable set");
            let db_engine = env::var("DB_ENGINE").expect("⚠️ no db engine variable set");
            let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");

            /* 
                app_sotrage contains the mongodb, postgres and actix_redis, redis 
                and redis_async which can be used to authorize then publish topics,
                response caching and subscribing asyncly to topics respectively
            */
            let app_storage = s3req::storage!{ // this publicly has exported inside the misc so we can access it here 
                db_name,
                db_engine,
                db_host,
                db_port,
                db_username,
                db_password
            }.await;


            /*  
                                        SETTING UP SHARED STATE DATA
                
                make sure we're starting the RoleNotifServer, MmrNotifServer and EcqNotifServer actor in here 
                and pass the actor isntance to the routers' threadpool in order to move them between different
                apis, otherwise each actor will be started each time by calling the related websocket route and
                the their last state will be lost.

                following servers need to be started globally so we can share them between actix threads 
                and push new ws session actor inside subscription routes into their sessions field, also 
                each server is an actor which allow us to communicate with them asyncly and concurrently 
                within the different parts of the app by message sending logic  
            */
            let role_notif_server_instance = RoleNotifServer::new(app_storage.clone()).start();
            let shared_ws_role_notif_server = Data::new(role_notif_server_instance.clone());
            
            let mmr_notif_server_instance = MmrNotifServer::new(app_storage.clone()).start();
            let shared_ws_mmr_notif_server = Data::new(mmr_notif_server_instance.clone());

            let ecq_notif_server_instance = EcqNotifServer::new(app_storage.clone()).start();
            let shared_ws_ecq_notif_server = Data::new(ecq_notif_server_instance.clone());

            let chatroomlp_server_instance = ChatRoomLaunchpadServer::new(app_storage.clone()).start();
            let shared_ws_chatroomlp_server = Data::new(chatroomlp_server_instance.clone());

            /* 
                initializing the pg listener state before starting the server, we'll pass 
                the initialized instance to the server app data as the shared state data 
                so it can be shareable and loadable inside actix routers' threads
            */
            let system_actor_instance = SystemActor{updated_users: HashMap::new()}.start();
            let shared_system_actor_instance = Data::new(system_actor_instance.clone());
            
            let pg_listener_instance = PgListenerActor::new(app_storage.clone(), system_actor_instance.clone()).start();
            let shared_pg_listener_instance = Data::new(pg_listener_instance.clone());


            let shared_storage = Data::new(app_storage.clone());

            /*  
                we can have a global like data by sharing it between different parts of the app
                using jobq channel cause in rust we don't have global concepts to define a mutable 
                global type thus we have to put the type inside the Arc<Mutex<Type>> and share it 
                using jobq channel like mpsc between other parts and in order to mutate it we must 
                lock on the type to acquire the mutex for updating data, something like hashmap 
                std::sync::Arc<tokio::sync::Mutex<HashMap<String, String>>>, that's why we're using
                app_data() method to share mutable global data structures between actix threads and
                apis asyncly, actually actix do this behind the scene for us using jobq channels 
                to avoid deadlocks and race conditions, also since every api or router is an async 
                task that must be handled inside the hyper threads thus the data that we want to 
                use inside of them and share it between other routers must be bounded Send + Sync + 'static
                
                the HttpServer::new function takes a factory function that produces an instance of the App, 
                not the App instance itself. This is because each worker thread needs to have 
                its own App instance.
            */
            info!("➔ 🚀 {} panel HTTP+WebSocket socket server has launched from [{}:{}] at {}", APP_NAME, host, port, chrono::Local::now().naive_local());
            let s = match HttpServer::new(move ||{
                /* 
                    each thread of the HttpServer instance needs its own app factory
                    so its routes can be executed and handled in actix threadpool
                */
                App::new()
                    /* 
                        SHARED STATE DATA
                    */
                    .app_data(Data::clone(&shared_storage.clone()))
                    .app_data(Data::clone(&shared_ws_role_notif_server.clone()))
                    .app_data(Data::clone(&shared_ws_mmr_notif_server.clone()))
                    .app_data(Data::clone(&shared_ws_ecq_notif_server.clone()))
                    .app_data(Data::clone(&shared_ws_chatroomlp_server.clone()))
                    .app_data(Data::clone(&shared_pg_listener_instance.clone()))
                    .app_data(Data::clone(&shared_system_actor_instance.clone()))
                    .wrap(Cors::permissive())
                    .wrap(Logger::default())
                    .wrap(Logger::new("%a %{User-Agent}i %t %P %r %s %b %T %D"))
                    /*
                        INIT WS SERVICE
                    */
                    .service(
                        actix_web::web::scope("/subscribe")
                            .configure(services::init_ws_notif)
                    )
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
                        actix_web::web::scope("/public")
                            .configure(services::init_public)
                    )
                }) 
                .listen($tcp_listener){
                    Ok(server) => {
                        server
                            /* 
                                running our own tcp listener server in a threadpool with 10 spawned 
                                threads to handle incoming connections asyncly and concurrently 
                            */
                            .workers(10) 
                            .run() /* actix web http+ws server runs in the same thread that actix has ran */
                            .await
                    },
                    Err(e) => {
        
                        /* custom error handler */
                        use error::{ErrorKind, ServerError::{ActixWeb, Ws}, PanelError};
                         
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();
        
                        let error_instance = PanelError::new(*SERVER_IO_ERROR_CODE, error_content, ErrorKind::Server(ActixWeb(e)), "HttpServer::new().bind");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
        
                        panic!("panicked at running actix web server at {}", chrono::Local::now());
                        
        
                    }
                };


            /* 
                this can't be reachable unless we hit the ctrl + c since the 
                http server will be built inside multiple threads in which all 
                server instances will be ran constanly in the background, and 
                must be the last thing that can be reachable before sending Ok(())
                from the main function, it's like the app will be halted in this
                section of the code cause anything after those threads rquires 
                that all the threads to be stopped and joined in order to execute 
                the logic after running the http server, which this can be done
                by stopping all of the threads using ctrl + c.
            */
            // info!("➔ 🎛️ starting conse panel on address: [{}:{}]", host, port);
            
            s /* returning the server */

        }
    };
}