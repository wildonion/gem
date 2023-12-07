





use crate::*;

/* 
    pg notif actor is a ds that will start subscribing to postgres event in 
    its interval loop using while let Some()... syntax in the background and realtime
    once it gets started, to notify other parts about tables changes by sending 
    the received event through mpsc channels.
*/
pub struct PgListenerActor{}

impl PgListenerActor{

    /* 
        pg streaming of events handler by subscribing to the event 
        in an interval loop using while let Some()... syntax
    */
    pub async fn subscribe(){

        /* 
        
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
        
        // use redis pubsub pattern
        // use Postgres' NOTIFY/LISTEN to notify app on users table update
        // with worker patterns like mpsc, tokio::select, tokio::spawn(), 
        // to fetch latest data from db every 5 seconds with a global 
        // mutexed shared state data to gets mutated during the checking process
        // and accessible inside other actix routes threads

        // or 

        // get new changes by sending GetUserNewChanges message from 
        // different parts of the app to this actor to get the latest
        // table update as a response of this actor, this can be done
        // by starting the actor in place where we're starting the server
        // then share the actor as a shared state data like Arc<Mutex< 
        // between actix routers threads so we can extract it from 
        // the app_data in each api

    }

}
