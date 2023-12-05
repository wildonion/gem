

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
    
    CREATE OR REPLACE FUNCTION notify_trigger() RETURNS trigger AS $$
    BEGIN
        PERFORM pg_notify('my_channel', 'update');
        RETURN NEW;
    END;
    $$ LANGUAGE plpgsql;

    CREATE TRIGGER user_update_trigger
    AFTER UPDATE ON users
    FOR EACH ROW EXECUTE PROCEDURE notify_trigger();

    use tokio_postgres::{NoTls, Error};

    #[tokio::main]
    async fn main() -> Result<(), Error> {
        // Connect to the database
        let (client, connection) = tokio_postgres::connect("host=localhost dbname=mydb user=myuser", NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        // Start listening to the channel
        client.execute("LISTEN my_channel", &[]).await?;

        loop {
            client.process_notifications().await;
            while let Some(notification) = client.notifications().try_recv() {
                println!("Got notification: {:?}", notification);
                // Refresh data from database or whatever you need to do
            }
        }
    }


*/

use crate::*;