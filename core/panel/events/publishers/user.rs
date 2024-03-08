

/*  -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
           IN-PLATFORM USER DATA PUBLISHER
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
*/


use actix_redis::{RespValue, Command};
use log::info;
use redis_async::resp_array;
use crate::*;
use actix::Addr;


pub async fn emit(
    redis_actor: Addr<RedisActor>, 
    channel: &str, 
    stringified_data: &str,
  ){

    let redis_password = std::env::var("REDIS_PASSWORD").unwrap_or("".to_string());

    /* 
                                
        --------------------------------
          AUTHORIZING WITH REDIS ACTOR
        --------------------------------

    */

    /* sending command to redis actor to authorize the this ws client */
    let redis_auth_resp = redis_actor
        .send(Command(resp_array!["AUTH", redis_password.as_str()])).await;


    if redis_auth_resp.is_err(){
      error!("🚨 --- redis actix actor mailbox error at {} due to: {}", chrono::Local::now().timestamp_nanos_opt().unwrap(), redis_auth_resp.unwrap_err());
    }


    /* 

        ----------------------------------------
                PUBLISHING WITH REDIS ACTOR
        ----------------------------------------
        since each actor notif has a 1 second interval for push notif subscription, we must 
        publish updated user constantly to the related channel to make sure that the subscribers will receive 
        the message at their own time and once 1 subscriber receives the message we'll break the 
        background loop since there is only one redis async subscriber in overall which will begin to 
        subscribing once the pg listener actor gets started, so in the following we're running an async 
        task every 1 second in the background hence we might have a successfull return from inside the 
        api where this method has called but still waiting for a subscriber to subscribe to the published
        topic in the that channel
    */
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    let cloned_channel = channel.to_string().clone();
    let cloned_stringified_data = stringified_data.to_string().clone();

    tokio::spawn(async move{

        /* publish until at least one sub subscribes to the topic */
        loop{

            /* tick every 1 second */
            interval.tick().await;
            
            let redis_pub_resp = redis_actor
              .send(Command(resp_array!["PUBLISH", &cloned_channel, cloned_stringified_data.clone()]))
              .await
              .unwrap();

                match redis_pub_resp{
                    Ok(resp_val) => {

                        match resp_val{
                            RespValue::Integer(subs) => {
                                
                                if subs >= 1{
                                    
                                    /* if we're here means that a subscriber received the notif */
                                    info!("🙋 --- [{subs:}] user listener subscriber actor has subscribed to topic : {}", cloned_channel);
                                    break;
                                    
                                }
                                
                                info!("👤 --- no one has subscribed yet ");

                            },
                            _ => {}
                        }

                    },
                    Err(e) => () /* or {} */
                }
        }

    });

    return;

}