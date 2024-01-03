


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


}