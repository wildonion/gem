



use std::time::Duration;

use serenity::model::prelude::Connection;

use crate::*;
use crate::resp;
use crate::passport;
use crate::constants::*;
use crate::misc::*;


/*
     ------------------------
    |        SCHEMAS
    | ------------------------
    |
    |

*/
#[derive(Serialize, Deserialize, Clone)]
pub struct Dev{
    pub id: u8,
}


/*
     ------------------------
    |          APIS
    | ------------------------
    |
    |

*/
#[get("/reveal-role")]
pub async fn index(
        req: HttpRequest, 
        id: web::Path<u8>, 
        redis_conn: web::Data<RedisConnection>, //// redis shared state data 
        storage: web::Data<Option<Arc<Storage>>> //// db shared state data
    ) -> Result<HttpResponse, actix_web::Error> {

    
    if let Some(header_value) = req.headers().get("Authorization"){
    
        let token = header_value.to_str().unwrap();
        
        /*
            @params: 
                - @request       → actix request object
                - @storage       → instance inside the request object
                - @access levels → vector of access levels
        */
        match passport!{ token }{
            true => {

                //// -------------------------------------------------------------------------------------
                //// ------------------------------- ACCESS GRANTED REGION -------------------------------
                //// -------------------------------------------------------------------------------------

                let id = id.to_owned();
                let data = Dev{id};
                let redis_conn = redis_conn.to_owned();
                let storage = storage.as_ref().to_owned();
                let db = storage.unwrap().get_db().await.unwrap();   

                // 🥑 todo - publish or fire the reveal role topic or event using redis pubsub
                // 🥑 todo - also call the /reveal/roles api of the hyper server                 
                // ...


                /*
                    shared state sharding to decrease the time lock, we can use a shard from 
                    the pool to update the mutex by locking on it inside a thread (either blocking 
                    using std::sync::Mutex or none blocking using tokio::sync::Mutex) and if other 
                    thread wants to use it it can use other shard instead of waiting for the locked 
                    shard to gets freed, we can use try_lock() method to check that the shard is 
                    currently being locked or not also we have to update the whole shards inside 
                    the pool at the end of each mutex free process which is something that will
                    be taken care of by semaphores.
                */

                let shards = 10;
                let (mutex_data_sender, mut mutex_data_receiver) = tokio::sync::mpsc::channel::<HashMap<i32, String>>(shards as usize);
                let (map_shards_sender, mut map_shards_receiver) = tokio::sync::broadcast::channel::<Vec<Arc<tokio::sync::Mutex<HashMap<i32, String>>>>>(shards as usize);
                let send_sync_map = Arc::new(tokio::sync::Mutex::new(HashMap::new())); //// no need to put in Mutex since we don't want to mutate it
                
                let mut map_shards: Vec<Arc<tokio::sync::Mutex<HashMap<i32, String>>>> = vec![];
                let mutex_data_sender = mutex_data_sender.clone();
                
                /*
                        
                    INITIALIZING SHARDS
                
                */
                for _ in 0..shards{
                    map_shards.push(send_sync_map.clone());
                }

                let mut current_data_length = map_shards[0].lock().await.len();


                /*

                    UPDATE THE map_shards RECEIVED FROM THE CHANNEL ASYNCLY  

                    waiting to receive the new shards from the channel 
                    to update the current shard inside the whole app
                    asyncly, since we're inside an eventloop this can 
                    be done at any time inside the app thus we're sure
                    that we'll always use an udpated version of the shards 

                */
                tokio::select!{
                    sent_shards = map_shards_receiver.recv() => {
                        if let Ok(shards) = sent_shards{
                            map_shards = shards;
                            current_data_length = map_shards[0].lock().await.len();
                        }
                    }
                }


                /*

                    FINDING FREE MUTEX TO UPDATE IT 

                    after finding a free mutex we'll update it then send it to the 
                    downside of the mpsc job queue channel in order to update the vector 
                    of shards by selecting the largest mutex data, this must be done asyncly 
                    since we don't know the time of the lock acquisition, it can happen any 
                    time during the app and due to having a shard of mutex data we have to 
                    update the whole shards with the latest data in a none blocking manner
                    since there might be other mutex-es that are in a lock process.    
                
                */
                tokio::spawn(async move{
                    for idx in 0..map_shards.clone().len(){
                        match map_shards[idx].clone().try_lock(){
                            Ok(mut gaurd) => {
                                
                                // udpate the gaurd 
                                let value = format!("value is {}", idx);
                                gaurd.insert(idx as i32, value);

                                // send the mutex to downside of the channel
                                mutex_data_sender.send(gaurd.to_owned()).await.unwrap();

                            },
                            Err(e) => {
                                // use other mutex instead
                                continue;
                            } 
                        }
                    }
                });
                
                /* 
                    
                    UPDATE OTHER MUTEX-es IN A NONE BLOCKING MANNER
                
                    if we receive something in here means that 
                    a mutex found that has the largest data inside
                    of it thus we'll choose this mutex as the proper
                    one to update the whole vector and remove forks.  
                */
                tokio::spawn(async move{
                    tokio::select!{
                        mutex_data = mutex_data_receiver.recv() => {
                            if let Some(largest_data) = mutex_data{
                                info!("🏳️ receiving largest mutex data");
                                
                                // check that this is the largest data
                                if current_data_length < largest_data.len(){
                                    
                                    // update the whole shards with the largest_data
                                    let new_shards = vec![Arc::new(tokio::sync::Mutex::new(largest_data)); shards];

                                    // broadcast the new shards to the channel
                                    map_shards_sender.send(new_shards).unwrap();
                                    
                                } else{

                                    /* MEANS THAT NO MUTEX HAS BEEN MUTATED YET */
                                }

                            } else{
                                
                                /* SOMETHING WENT WRONG IN SENDING TO CHANNEL */
                                
                            }    
                        }
                    }
                });


                resp!{
                    Dev, //// the data type
                    data.clone(), //// response data
                    FETCHED, //// response message
                    StatusCode::OK, //// status code
                }


                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------
                //// -------------------------------------------------------------------------------------

            },
            false => {
                
                resp!{
                    &[u8], //// the date type
                    &[], //// the data itself
                    INVALID_TOKEN, //// response message
                    StatusCode::FORBIDDEN, //// status code
                }
            }
        }

    } else{
        
        resp!{
            &[u8], //// the date type
            &[], //// the data itself
            NOT_AUTH_HEADER, //// response message
            StatusCode::FORBIDDEN, //// status code
        }
    }

}