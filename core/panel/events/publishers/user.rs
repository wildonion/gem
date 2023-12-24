


use crate::{*, models::users::UserWalletInfoResponse};


#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserNotif{
    wallet_info: UserWalletInfoResponse,
    notifs: Vec<NotifData>,
    updated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct NotifData{
    event_id: String, // event_id is the id of an entity that caused this notif happened
    fired_at: Option<i64>,
    seen: bool,
    action_type: ActionType,
    action_data: serde_json::Value, // we don't know the exact type of action_data, so we've used json value
}

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
    ListNft,
    DelistNft,
    BuyNft,
    UnclaimedGiftCard
}

impl UserNotif{
    fn set(&mut self, notif_data: NotifData) -> Self{
        self.notifs.push(notif_data);
        let user_notif = UserNotif { wallet_info: self.wallet_info.clone(), notifs: self.notifs.clone(), updated_at: self.updated_at };
        UserNotif{
            ..user_notif /* filling all the fields with the user_notif ones */
        }
    }
    fn get(&mut self) -> Self{
        let this = UserNotif { ..Default::default() };
        this
    }
}

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

pub async fn publish_actions(){

    type Method = fn() -> i32;
    fn run(param: impl Fn() -> ActionType, method: Method)
    // bounding generic Method to traits and lifetimes
    where Method: Send + Sync + 'static{}
    fn execute<'f, F>(param: &'f F) -> () 
    // bounding generic F to closure, lifetimes and other traits
    where F: Fn() -> ActionType + Send + Sync + 'static{}

}