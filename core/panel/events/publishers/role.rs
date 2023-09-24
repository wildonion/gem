


use actix::Addr;
use mongodb::bson::oid::ObjectId;
use crate::*;


#[derive(Clone, Debug, Serialize, Default, Deserialize)]
pub struct Reveal{
    pub players: Vec<PlayerRoleInfo>,
    pub event_id: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct PlayerRoleInfo{
  pub _id: ObjectId, // ObjectId is the bson type of _id inside the mongodb
  pub username: String,
  pub status: u8,
  pub role_name: Option<String>,
  pub role_id: Option<ObjectId>, // this field can be None at initialization which is the moment that a participant reserve an event
  pub side_id: Option<ObjectId>, // this field can be None at initialization which is the moment that a participant reserve an event
}


impl Reveal{

    pub async fn publish(
        redis_actor: Addr<RedisActor>, 
        channel: &str, 
        stringified_data: &str,
        notif_room: &str
      ){

        let redis_password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());

        /* 
                                    
            --------------------------------
              AUTHORIZING WITH REDIS ACTOR
            --------------------------------

        */

        /* sending command to redis actor to authorize the this ws client */
        let redis_auth_resp = redis_actor
            .send(Command(resp_array!["AUTH", redis_password.as_str()])).await;


        if redis_auth_resp.is_err(){
          error!("🚨 --- redis actix actor mailbox error at {} due to: {}", chrono::Local::now().timestamp_nanos(), redis_auth_resp.unwrap_err());
        }


        /* 

            ----------------------------------------
                    PUBLISHING WITH REDIS ACTOR
            ----------------------------------------
            since each websocket session has a 1 second interval for push notif subscription, we must 
            publish roles constantly to the related channel to make sure that the subscribers will receive 
            the message at their own time and once 1 subscriber receives the message we'll break the 
            background loop since there is only one redis async subscriber in overall which will begin to 
            subscribing once the websocket server gets started, so in the following we're running an async 
            task every 1 second in the background thus we might have a successfull return from inside the 
            api where this method has called but still waiting for a subscriber to subscribe to the published
            event room 

        */
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        let cloned_channel = channel.to_string().clone();
        let cloned_stringified_data = stringified_data.to_string().clone();
        let cloned_notif_room = notif_room.to_string().clone();

        tokio::spawn(async move{

            /* publish until at least one sub subscribes to the topic */
            loop{

                /* tick every 1 second */
                interval.tick().await;
                
                /* publishing revealed roles to the reveal-role-{notif_room:} channel */
                let redis_pub_resp = redis_actor
                  .send(Command(resp_array!["PUBLISH", &cloned_channel, cloned_stringified_data.clone()]))
                  .await
                  .unwrap();

                    match redis_pub_resp{
                        Ok(resp_val) => {

                            match resp_val{
                                RespValue::Integer(subs) => {
                                    
                                    if subs >= 1{
                                        
                                        /* if we're here means that ws session received the notif */
                                        info!("🙋 --- [{subs:}] subscriber has subscribed to event: [{cloned_notif_room:}] to receive roles notif");
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

}