


use actix::Addr;
use crate::constants::WS_SUBSCRIPTION_INTERVAL;
use crate::{*, models::users::UserWalletInfoResponse};


#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserNotif{
    wallet_info: UserWalletInfoResponse,
    notifs: Vec<NotifData>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SingleUserNotif{
    pub wallet_info: UserWalletInfoResponse,
    pub notif: NotifData
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct NotifData{
    actioner_wallet_info: UserWalletInfoResponse, // it can be the user himself or others caused the event to be happened
    pub fired_at: Option<i64>,
    action_type: ActionType,
    action_data: serde_json::Value, // we don't know the exact type of action_data, so we've used json value
}

// gallery
// collection
// nft
// friend and invitation requests
#[derive(Serialize, Deserialize, Clone, Default)]
pub enum ActionType{
    InvitationRequestFrom,
    FriendRequestFrom,
    LikeNft,
    CommentNft,
    CreateNft,
    #[default]
    MintNft, // mint nft is the default action type
    CreateCollection,
    UpdateCollection,
    CreatePrivateGallery,
    UpdatePrivateGallery,
    ListNft,
    DelistNft,
    BuyNft,
    UnclaimedGiftCard,
    DepositGiftCard
}

impl UserNotif{

    fn set(&mut self, notif_data: NotifData) -> Self{
        self.notifs.push(notif_data);
        let UserNotif{ wallet_info, notifs } = self;
        let notif_isntance = UserNotif{
            ..self.clone() /* filling all the fields with the self ones */
        };
        self.clone()
    }
    fn get(&mut self) -> Self{
        let this = UserNotif { ..Default::default() };
        this
    }
}

// making the NotifExt trait sendable for 
// multithreaded work-stealing executors
// cause we have an async method that may
// gets sovled inside tokio spawn threadpool
#[trait_variant::make(NotifExtSend: Send)] 
pub trait NotifExt: Sync{
    type Data;
    fn set_user_notif(&mut self, notif_data: NotifData) -> Self;
    fn set_user_wallet_info(&mut self, wallet_info: UserWalletInfoResponse) -> Self;
    async fn get_user_notifs(&self) -> Vec<NotifData>;
    fn update_new_slice_notifs(&mut self, notif_slice: Vec<NotifData>);
}

impl NotifExt for UserNotif{
    type Data = Self;

    async fn get_user_notifs(&self) -> Vec<NotifData> {
        self.notifs.clone()
    }

    fn set_user_wallet_info(&mut self, wallet_info: UserWalletInfoResponse) -> Self {
        self.wallet_info = wallet_info;
        self.clone()
    }

    fn set_user_notif(&mut self, new_notif: NotifData) -> Self { // since the set() method of the UserNotif instance is mutable this method must be mutable too
        self.set(new_notif)
    }

    fn update_new_slice_notifs(&mut self, notif_slice: Vec<NotifData>){
        self.notifs = notif_slice;
    }

}

pub async fn publish(
    redis_actor: Addr<RedisActor>,
    channel: &str, 
    stringified_data: &str, // stringified data of SingleUserNotif struct instance
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
        publish new user notif constantly to the related channel to make sure that the subscribers will receive 
        the message at their own time and once 1 subscriber receives the message we'll break the 
        background loop since there is only one redis async subscriber in overall which will begin to 
        subscribing once the user notif listener actor gets started, so in the following we're running an async 
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
                                    
                                    /* if we're here means that ws session received the notif */
                                    info!("🙋 --- [{subs:}] user notif listener subscriber actor has subscribed to topic : {}", cloned_channel);
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