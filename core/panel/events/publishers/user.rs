


use actix::Addr;
use crate::constants::WS_SUBSCRIPTION_INTERVAL;
use crate::{*, models::users::UserWalletInfoResponse};
use self::models::users::{UserData, User};
use self::models::users_fans::FriendData;


#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct UserNotif{
    wallet_info: UserWalletInfoResponse,
    notifs: Vec<NotifData>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct SingleUserNotif{
    pub wallet_info: UserWalletInfoResponse,
    pub notif: NotifData
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct NotifData{
    pub actioner_wallet_info: UserWalletInfoResponse, // it can be the user himself or others caused the event to be happened
    pub fired_at: Option<i64>,
    pub action_type: ActionType, // event type
    pub action_data: serde_json::Value, // event data, we don't know the exact type of action_data, so we've used json value
}

// followings are the whole events that might get triggered or fired
// in the whole platform, the size of the enum is the size of its 
// largest variant + usize tag which is a pointer points to the currnt variant
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub enum ActionType{
    InvitationRequestFrom,
    FriendRequestFrom,
    AcceptFriendRequest,
    AcceptInvitationRequest,
    RemoveInvitedFriendFrom,
    LikeNft,
    DislikeNft,
    CommentNft,
    CreateNft,
    #[default]
    MintNft, // mint nft is the default action type
    ListNft,
    DelistNft,
    TransferNft,
    UpdateOnchainNft,
    BuyNft,
    CreateCollection,
    UpdateCollection,
    CreatePrivateGallery,
    UpdatePrivateGallery,
    ExitFromPrivateGalleryRequest,
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
        let this = UserNotif { ..self.clone() };
        this
    }
}

// making the NotifExt trait sendable for 
// multithreaded work-stealing executors
// cause we have an async method that may
// gets sovled inside tokio spawn threadpool
#[trait_variant::make(NotifExtSend: Send)] /* make NotifExt trait sendable so we can call its async method in other threads */
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
        self.get()
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

pub async fn publish_nft_list_event_2_all_nft_owner_friends(
    friends: Vec<FriendData>,
    redis_actor: Addr<RedisActor>,
    channel: &str, 
    nft_owner: User,
    action_data_value: serde_json::Value,
    connection: &mut PooledConnection<ConnectionManager<PgConnection>>
){

    for friend in friends{

        /** -------------------------------------------------------------------- */
        /** ----------------- publish new event data to `on_user_action` channel */
        /** -------------------------------------------------------------------- */
        // the actioner is the nft owner himself 
        let actioner_wallet_info = UserWalletInfoResponse{
            username: nft_owner.clone().username,
            avatar: nft_owner.clone().avatar,
            bio: nft_owner.clone().bio,
            banner: nft_owner.clone().banner,
            mail: nft_owner.clone().mail,
            screen_cid: nft_owner.clone().screen_cid,
            extra: nft_owner.clone().extra,
            stars: nft_owner.clone().stars,
            created_at: nft_owner.clone().created_at.to_string(),
        };

        let friend_info = User::find_by_screen_cid(&friend.screen_cid, connection).await.unwrap();

        // notify his friend about the event 
        let user_wallet_info = UserWalletInfoResponse{
            username: friend_info.username,
            avatar: friend_info.avatar,
            bio: friend_info.bio,
            banner: friend_info.banner,
            mail: friend_info.mail,
            screen_cid: friend_info.screen_cid,
            extra: friend_info.extra,
            stars: friend_info.stars,
            created_at: friend_info.created_at.to_string(),
        };
        let user_notif_info = SingleUserNotif{
            wallet_info: user_wallet_info,
            notif: NotifData{
                actioner_wallet_info,
                fired_at: Some(chrono::Local::now().timestamp()),
                action_type: ActionType::ListNft,
                action_data: action_data_value.clone()
            }
        };
        let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
        self::publish(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;

    }

}