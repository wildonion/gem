



/*
    
    - actix ws actor event and stream handler/loop using tokio spawn, 
        select, mpsc, mutex and tcp with redis and libp2p pubsub streams
    - event and stream handler to handle the incoming async task like ws messages 
        using actix StreamHandler and tokio tcp 
    - message handler to handle the message type which is going to 
        be sent between other actors
    - ws actor stream and event handlers are like:
        streaming over incoming bytes through the tokio tcp socket 
        to send them as the async task to tokio green threadpool using
        tokio spawn to handle them as an event using tokio select event 
        loop handler


    ------------------------------------------------
    networking(actor, ws, redis pubsub and streams):
    ------------------------------------------------
        event of async task handler, streamer, loop 
        inside std::thread::scope and tokio::spawn based 
        tokio tcp stream or mmq streaming over future 
        bytes using tokio and ws actor and redis pubsub 
        and streams by streaming over incoming bytes 
        inside the tokio gread threadpool and pass them 
        to other threads using tokio::sync::mpsc, actor, 
        select, spawn, mutex, pubsub, tcp stream, hex, serding 
        )to_string vs from utf8)
        tokio::spawn(async move{
            while let Ok(data) = streamer.recv().await{
                /* decode the bytes to a struct; see redis4 repo */
                let decoded;
                sender.send(decoded)
            }
        });


*/


use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; // self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use borsh::{BorshDeserialize, BorshSerialize};
use uuid::Uuid;





#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CollaborationQueue{
    pub players: Vec<Player>, // user pool that can be used to start a match between them
    pub event_id: String, 
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player{
    pub pub_key: String,
    pub rank: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrentMatch{
    pub event_id: String,
    pub players: Vec<Player>,
    pub room_id: String,
    pub is_locked: bool
}


pub async fn race_condition_avoidance(){

    /* ---------------------------------------------------------------------- */
    /* ---------------------- RACE CONDITION AVOIDANCE ---------------------- */
    /*  
                    https://github.com/wildonion/redis4

        race conditions means that two threads want to mutate the data 
        at the same time we have to use mutex so thell the other threads
        wait there is a threads that is trying to mutate this type and 
        will update you once the lock gets freed and in order to avoid blockcing 
        issues in the current thread we have to lock inside a separate thread 
        and mutate the type then send it through the jobq channel to the other 
        threads for reading
    */
    pub type ArcedMutexed = std::sync::Arc<tokio::sync::Mutex<String>>;
    #[derive(Clone)]
    pub struct Data<D: Send + Sync + 'static>{
        /* we're using tokio mutex to avoid blocing issues inside the current thread since it locks asycnly */
        pub actual: D
    }
    let mut data_instance = Data::<ArcedMutexed>{
        actual: std::sync::Arc::new(tokio::sync::
            Mutex::new(
                String::from("a mutexed data")
            )
        ),
    };
    
    println!("data instance actual value before getting mutated >>> [{}]", data_instance.actual.lock().await.to_owned());
    
    /* reading from the channel is a mutable process thus receiver must be mutable */
    let (data_sender, mut data_receiver) = 
        tokio::sync::mpsc::channel::<Data<ArcedMutexed>>(1024);
    /*
        since tokio spawn takes a closure which captures the env vars 
        we have to use the cloned form of those types and pass them into
        the closure scopes so we can use them in later scopes 
    */
    let sender = data_sender.clone();
    tokio::spawn(async move{
        
        let new_string = String::from("an updated mutexed");
        /* 
            we're cloning data_instance and data_instance_cloned.actual to create a 
            longer lifetime value to use the cloned form to mutate, since by sending 
            data_instance_cloned to the channel its lifetime will be dropped and its 
            ownership will be moved because we're borroing the actual field by locking 
            on it so we can't move the data_instance_cloned into the mpsc channel using 
            the sender, in other words we can't move out of the type if it's behind a 
            shared reference we have to either pass a reference or clone the type and 
            work on the cloned form like the followings which we're cloning the actual 
            field to lock on its mutex and send the data_instance_cloned into 
            the downside of the channel
        */
        let data_instance_cloned = data_instance.clone();
        let data_instance_cloned_actual = data_instance_cloned.actual.clone();
        let mut data_string = data_instance_cloned_actual.lock().await; /* lock the mutex to mutate it */
        
        /* 
            mutating the locked mutex is done by dereferencing the guard 
            we're mutating data string inside the actual field in data_instance_cloned
            this will mutate the actual field inside data_instance_cloned 
        */
        *data_string = new_string; /* the actual field of the data_instance_cloned will be mutated */

        if let Err(why) = sender.send(data_instance_cloned).await{
            println!("can't send because {:?}", why.to_string());
        }

    });

    /* receiving asyncly inside other threads to avoid blocking issues on heavy computations */
    tokio::spawn(async move{
        /* receving data asyncly while they're comming to the end of mpsc jobq channle */
        while let Some(data) = data_receiver.recv().await{
            
            let new_data_string = data.actual.lock().await.to_owned();
            println!("data instance actual value after getting mutated >>> [{}]", new_data_string);
    
        }
    });

}


// fire/emit/publish UserNotif events in ws/rpc/zmq server or using redis
// like using emit!(UserNotif{}) macro which emit and fire an
// event through the redis streaming channel to clients 
// sub or listen to UserNotif events in ws/rpc/zmq client
// using an event loop or listener.
// update UserNotif on every data changes through its related api calls
// then fire the updated data event through the ws server so the client
// can subs using ws to the fired event 
#[derive(Serialize, Deserialize, Clone)]
pub struct UserNotif{
    user_id: String,
    notifs: Vec<NotifData>,
    updated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NotifData{
    fired_at: Option<i64>,
    seen: bool,
    topic: String, // json string contains the actual data like fireing the player status (role and state changing) during the game 
}

impl UserNotif{
    fn set(&mut self, notif_data: NotifData) -> Self{
        self.notifs.push(notif_data);
        let user_notif = UserNotif { user_id: self.user_id.clone(), notifs: self.notifs.clone(), updated_at: self.updated_at };
        UserNotif{
            ..user_notif /* filling all the fields with the user_notif ones */
        }
    }
}


// in order to call the NotifExt methods on the
// UserNotif struct the trait must be implemented 
// for the UserNotif struct and imported inside
// where we want to call the methods on the struct
// instance.
pub trait NotifExt{
    type Data;
    fn set_user_notif(&mut self, notif_data: NotifData) -> Self;
    fn get_user_notif(&self) -> Vec<NotifData>;
}

impl NotifExt for UserNotif{
    type Data = Self;

    fn get_user_notif(&self) -> Vec<NotifData> {
        self.notifs.clone()
    }

    fn set_user_notif(&mut self, new_notif: NotifData) -> Self { // since the set() method of the UserNotif instance is mutable this method must be mutable too
        self.set(new_notif)
    }

}

#[derive(Serialize, Deserialize, Clone)]

pub struct Royalty{
    pub wallet_address: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Clone)]

pub struct Nft{
    pub owner: String,
    pub royalties: Vec<Royalty>, // Royalty struct must be public if this field is public since we want to access this field later which contains the Royalty instances
    pub events: Vec<UserNotif>
}