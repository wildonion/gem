

/* 

    - static lazy arc mutex rwlock map as an in memory thread safe database
      rusty ltg(&mut,box,trait,macro,generic) and shared state data to make
      them shareable between routes and threads using Arc<Mutex<Db>> + Send + Sync + 'static with mpsc
    - clap for setup, renew, redeploy, publish process
    - pastel sdk (macos walletnode setup)
    - arc will be used to share the pointer of the type between threads 
      safely cause it's a atomically-reference-counted shared pointer
    - impl Job for ClpEvent{} to execute finish time task of clp event in tokio::spawn() in background every 5 seconds
    - workers folder contains jobs with tokio spawn, select and mpsc which will be triggered 
      once the event taken place

    fn get_name() -> String{ String::from("") }
    let callback = |func: fn() -> String|{
        func();
    };
    callback(get_name);

    // use worker patterns like mpsc, tokio::select, tokio::spawn(), 
    // to fetch latest data from db every 5 seconds with a global 
    // mutexed shared state data to be mutated during the checking process
    // and accessible inside other actix routes threads
    // wasm, box pin, impl Trait | &dyn Trait, Send Sync Arc, Weak, Rc, RefCell, Mutex, RwLock

*/

use crate::*;