



use crate::events::sse::broadcaster::Event;
use crate::*;
use crate::models::users::LoginInfoRequest;

// use this method to add new sse client
pub async fn sse_client(app_state: web::Data<AppState>) -> PanelHttpResponse{
    // since unwrap() takes the ownership of the isntance and the app_state hence we should clone the 
    // sse_broadcaster to prevent it from moving, as_ref() and as_mut() is not working here
    // DerefMut is not implemented for AppState
    app_state.sse_broadcaster.clone().unwrap().add_client().await 
}

// use this method to broadcast new event
pub async fn broadcast_event(
    app_state: web::Data<AppState>,
    event_info: web::Path<(String, String)>,
) -> PanelHttpResponse{
    
    let topic = event_info.clone().0;
    let event = event_info.clone().1;
    // since unwrap() takes the ownership of the isntance and the app_state hence we should clone the 
    // sse_broadcaster to prevent it from moving, as_ref() and as_mut() is not working here
    // DerefMut is not implemented for AppState
    let get_sse_broadcaster = app_state.sse_broadcaster.clone();
    let mut sse_broadcaster = get_sse_broadcaster.unwrap();
    sse_broadcaster.broadcast(&topic, Event{data: event}).await
}

// start a tcp streamer in the background
pub async fn start_streaming(){

    /* 
        executing a tcp streamer in the background, this is started 
        in the background even after dropping the http connection,
        and waits for the client request, note that we can start a 
        tokio tcp listener from the context of actix_web::main runtime
        but can't start actix stuffs like actors from the context of 
        tokio::main runtime, 
        more about actor worker threadpool in: https://github.com/wildonion/zoomate/blob/main/src/helpers/acter.rs
    */
    tokio::spawn(async move{ // execute the creating process of a tcp listener asyncly inside a tokio threadpool

        let addr = format!(
            "{}:{}", 
            std::env::var("HOST").unwrap(),
            std::env::var("TCP_PORT").unwrap()
        );
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        info!("tcp listener started in the background");

        tokio::spawn(async move{ // execute the accepting process of a tcp listener asyncly inside a tokio threadpool

            while let Ok((mut stream, addr)) = listener.accept().await{

                tokio::spawn(async move{ // execute the reading process from the socket stream asyncly inside a tokio threadpool
                    
                    info!("got new client: {:?}", addr.to_string());
                    let mut buffer = vec![];

                    while match stream.read(&mut buffer).await{
                        Ok(size) if size == 0 => return,
                        Ok(size) => {

                            let received_buffer = &buffer[..size];
                            let datastr = std::str::from_utf8(&received_buffer).unwrap(); // map into str if there is no structurized data
                            let datainfo = serde_json::from_slice::<LoginInfoRequest>(&buffer).unwrap(); // map into some kinda data structure if the received bytes are structurized
                            
                            // do whatever is needed with received data 
                            // ...
                            
                            info!("parsed data: {:?}", datastr);
                            return;
                        },
                        Err(why) => {
                            error!("can't read from stream due to: {}", why.to_string());
                            return;
                        }
                    }
                    {} // this belongs to the while match

                });
            }
        });
         
    });

}

#[macro_export]
macro_rules! bootsteap {
    (
        
        /* ... setup args go here ... */
        $tcp_listener:expr

    ) => {
        
        {

            use std::env;
            use actix_web::{web, App, HttpRequest, middleware, HttpServer, Responder, HttpResponse, get, ResponseError};
            use actix_web::middleware::Logger;
            use dotenv::dotenv;
            use crate::helpers::config::{Env as ConfigEnv, Context};
            use crate::helpers::config::EnvExt;
            use crate::constants::*;
            use crate::events::sse::broadcaster::Broadcaster;
            use crate::events::subscribers::handlers::actors::ws::servers::role::RoleNotifServer;
            use crate::events::subscribers::handlers::actors::ws::servers::mmr::MmrNotifServer;
            use crate::events::subscribers::handlers::actors::ws::servers::chatroomlp::ChatRoomLaunchpadServer;
            use crate::events::subscribers::handlers::actors::notif::user::UserListenerActor;
            use crate::events::subscribers::handlers::actors::notif::action::UserActionActor;
            use crate::events::subscribers::handlers::actors::notif::system::SystemActor;
            use crate::events::subscribers::handlers::actors::notif::clp::ClpEventSchedulerActor;
            use crate::events::subscribers::handlers::actors::notif::balance::UserBalanceActor;
            use crate::events::subscribers::handlers::actors::ci::run::RunAgentActor;
            use crate::events::subscribers::handlers::actors::ci::deploy::DeployAgentActor;
            use crate::apis::admin::AdminComponentActor;
            use crate::apis::user::UserComponentActor;
            use crate::apis::health::HealthComponentActor;
            use crate::apis::public::PublicComponentActor;
            use crate::helpers::server::{sse_client, broadcast_event};

            
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
                
                make sure we're starting all the actors in here and pass the actor isntance to the routers' 
                threadpool in order to move them between different apis, otherwise each actor will be started 
                each time by calling the related websocket route and the their last state will be lost.
                following servers need to be started globally so we can share them between actix threads 
                and push new ws session actor inside subscription routes into their sessions field, also 
                each server is an actor which allow us to communicate with them asyncly and concurrently 
                within the different parts of the app by message sending logic.
            */
            let role_notif_server_instance = RoleNotifServer::new(app_storage.clone()).start();            
            let mmr_notif_server_instance = MmrNotifServer::new(app_storage.clone()).start();
            let chatroomlp_server_instance = ChatRoomLaunchpadServer::new(app_storage.clone()).start();
            let system_actor_instance = SystemActor{updated_users: HashMap::new()}.start();
            let user_listener_instance = UserListenerActor::new(app_storage.clone(), system_actor_instance.clone()).start();
            let users_action_listener_instance = UserActionActor::new(app_storage.clone(), system_actor_instance.clone()).start();
            let clp_event_listener_instance = ClpEventSchedulerActor::new(app_storage.clone()).start();
            let run_actor_instance = RunAgentActor::new(port, std::path::PathBuf::new()).start();
            let deploy_actor_instance = DeployAgentActor::new(port, std::path::PathBuf::new()).start();
            let user_balance_listener_instance = UserBalanceActor::new(app_storage.clone()).start();
            let admin_component_actor = AdminComponentActor::new(vec![], app_storage.clone()).start();
            let user_component_actor = UserComponentActor::new(vec![], app_storage.clone()).start();
            let public_component_actor = PublicComponentActor::new(vec![], app_storage.clone()).start();
            let health_component_actor = HealthComponentActor::new(vec![], app_storage.clone()).start();

            // setting up the whole app state data
            let mut app_state = AppState::init();
            app_state.app_sotrage = app_storage.clone(); 
            app_state.subscriber_actors = Some(
                SubscriberActors{
                    clp_actor: chatroomlp_server_instance.clone(),
                    role_actor: role_notif_server_instance.clone(),
                    mmr_actor: mmr_notif_server_instance.clone(),
                    action_actor: users_action_listener_instance.clone(),
                    system_actor: system_actor_instance.clone(),
                    user_actor: user_listener_instance.clone(),
                    balance_actor: user_balance_listener_instance.clone(),
                    clp_event_checker_actor: clp_event_listener_instance.clone(),
                }
            );
            app_state.component_actors = Some(
                ApiComponentActors{
                    admin_api_actor: admin_component_actor.clone(),
                    user_api_actor: user_component_actor.clone(),
                    health_api_actor: health_component_actor.clone(),
                    public_api_actor: public_component_actor.clone(),
                }
            );
            app_state.sse_broadcaster = Some(Broadcaster::new());
            app_state.agent_actors = Some(
                AgentActors{
                    run_agent_actor: run_actor_instance.clone(),
                    deploy_agent_actor: deploy_actor_instance.clone()
                }
            );
            app_state.config = {
                let env = ConfigEnv::default();
                let ctx_env = env.get_vars();
                std::sync::Arc::new(ctx_env)
            };
            let shared_state_app = Data::new(app_state.clone()); // making the app state as a shareable data 


            /*      
                take note that Actix Web uses multiple single-thread runtimes and data won’t be sent 
                between threads, that's why HttpResponse is not Send and Sync.
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

                        shared_state_app contains all the global data 
                        that will be shared between actix api routers' threads
                    */
                    .app_data(Data::clone(&shared_state_app.clone())) // the whole app state: s3, actors and configs
                    .wrap(Cors::permissive())
                    .wrap(Logger::default())
                    .wrap(Logger::new("%a %{User-Agent}i %t %P %r %s %b %T %D"))
                    .wrap(middleware::Compress::default())
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
                    /*
                        SSE ROUTES
                    */
                    .route("/events{_:/?}", web::get().to(sse_client)) // eg: /events, /events/123, /events/abc -> fetching event
                    .route("/events/{msg}", web::get().to(broadcast_event))
                }) 
                .listen($tcp_listener){ // bind the http server on the passed in tcp listener cause after all http is a tcp based protocol!
                    Ok(server) => {
                        server
                            /* 
                                running our own tcp listener server in a threadpool with 10 spawned 
                                threads to handle incoming connections asyncly and concurrently also
                                we should make the app to be ran constantly so we can monitor the logics 
                                inside any tokio::spawn() or other threads which have been executed 
                                concurrently and asyncly in the background, otherwise we would use some 
                                mpsc channel to send any computational result inside of those threads 
                                into the channel so we can receive it outside of their scopes while 
                                the app is running
                            */
                            .workers(10) 
                            .run() /* actix web http+ws server runs in the same thread that actix has ran */
                            .await
                    },
                    Err(e) => {
        
                        /* custom error handler */
                        use helpers::error::{ErrorKind, ServerError::{ActixWeb, Ws}, PanelError};
                         
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
            
            s /* returning the server which is being ran constantly, concurrently and asyncly in the background threads */

        }
    };
}