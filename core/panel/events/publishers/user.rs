


use crate::constants::WS_SUBSCRIPTION_INTERVAL;
use crate::{*, models::users::UserWalletInfoResponse};


#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserNotif{
    wallet_info: UserWalletInfoResponse,
    notifs: Vec<NotifData>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct NotifData{
    actioner_wallet_info: UserWalletInfoResponse, // it can be the user himself or others caused the event to be happened
    fired_at: Option<i64>,
    seen: bool,
    action_type: ActionType,
    action_data: serde_json::Value, // we don't know the exact type of action_data, so we've used json value
}

// gallery
// collection
// nft
// friend
// invitation requests
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
        UserNotif{
            ..self.clone() /* filling all the fields with the self ones */
        }
    }
    fn get(&mut self) -> Self{
        let this = UserNotif { ..Default::default() };
        this
    }
}

pub trait NotifExt: Send + Sync{
    type Data;
    fn set_user_notif(&mut self, notif_data: NotifData) -> Self;
    async fn get_user_notif(&self) -> Vec<NotifData>;
}

impl NotifExt for UserNotif{
    type Data = Self;

    async fn get_user_notif(&self) -> Vec<NotifData> {
        self.notifs.clone()
    }

    fn set_user_notif(&mut self, new_notif: NotifData) -> Self { // since the set() method of the UserNotif instance is mutable this method must be mutable too
        self.set(new_notif)
    }

}

pub async fn publish_actions(user_info: UserWalletInfoResponse, notif_data: NotifData){


    type Method = fn() -> i32;
    fn run<'lifteim>(param: impl Fn() -> ActionType, method: &'lifteim Method)
    // bounding generic Method to traits and lifetimes
    where Method: Send + Sync + 'static{}
    fn execute<'f, F>(param: &'f mut F) -> () 
    // bounding generic F to closure, lifetimes and other traits
    where F: Fn() -> ActionType + Send + Sync + 'static{}

    trait Interface: Send + Sync + 'static{}
    struct Instance{}
    impl Interface for Instance{}
    impl Interface for (){}
    type BoxedTrait = Box<dyn FnOnce() -> ()>;
    struct Test<F: Send + Sync + 'static + Clone + Default> where F: FnOnce() -> (){
        pub data: F,
        pub another_data: BoxedTrait
    }
    fn trait_as_ret_and_param_type(param: &mut impl FnOnce() -> ()) -> impl FnOnce() -> (){ ||{} }
    fn trait_as_ret_type(instance_type: Instance) -> impl Interface{ instance_type }
    fn trait_as_ret_type_1(instance_type: Instance) -> impl Interface{ () }
    fn trait_as_param_type(param: impl FnOnce() -> ()){}
    

}