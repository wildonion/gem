



use crate::models::users_tokens::{NewUserTokenRequest, UserToken};
use std::io::Write;
use std::time::{UNIX_EPOCH, SystemTime};
use actix::Addr;
use borsh::{BorshSerialize, BorshDeserialize};
use chrono::Timelike;
use futures_util::TryStreamExt;
use lettre::message::Mailbox;
use models::sys_treasury::SysTreasury;
use crate::*;
use crate::helpers::misc::{Response, gen_random_chars, gen_random_idx, gen_random_number, get_ip_data, Limit, gen_random_chars_0_255};
use crate::models::users_galleries::{UserPrivateGallery, NewUserPrivateGalleryRequest};
use crate::schema::{users, users_tasks, users_mails, users_phones};
use crate::schema::users::dsl::*;
use crate::models::xbot::Twitter;
use crate::constants::*;
use self::events::publishers::action::{ActionType, NotifData, SingleUserNotif};
use super::users_collections::{UserCollection, CollectionOwnerCount};
use super::users_fans::{UserFan, FriendData, FriendOwnerCount};
use super::users_galleries::GalleryOwnerCount;
use super::users_logins::{NewUserLoginRequest, UserLogin};
use super::users_mails::UserMail;
use super::users_nfts::{UserNft, NftOwnerCount};
use super::users_phones::UserPhone;
use super::users_tasks::UserTask;
use chrono::NaiveDateTime;



/* 

    diesel migration generate users ---> create users migration sql files
    diesel migration run            ---> apply sql files to db 
    diesel migration redo           ---> drop tables 

*/
#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct User{
    pub id: i32,
    pub region: Option<String>,
    pub username: String, /* unique */
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub wallet_background: Option<String>,
    pub activity_code: String,
    pub twitter_username: Option<String>, /* unique */
    pub facebook_username: Option<String>, /* unique */
    pub discord_username: Option<String>, /* unique */
    pub identifier: Option<String>, /* unique */
    pub mail: Option<String>, /* unique */
    pub google_id: Option<String>, /* unique */
    pub microsoft_id: Option<String>, /* unique */
    pub is_mail_verified: bool,
    pub is_phone_verified: bool,
    pub phone_number: Option<String>, /* unique */
    pub paypal_id: Option<String>, /* unique */
    pub account_number: Option<String>, /* unique */
    pub device_id: Option<String>, /* unique */
    pub social_id: Option<String>, /* unique */
    pub cid: Option<String>, /* unique */
    pub screen_cid: Option<String>, /* unique */
    pub snowflake_id: Option<i64>, /* unique */
    pub stars: Option<i64>,
    pub user_role: UserRole,
    pub pswd: String,
    pub token_time: Option<i64>,
    pub balance: Option<i64>,
    pub extra: Option<serde_json::Value>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name=users)]
pub struct FetchUser{
    pub id: i32,
    pub region: Option<String>,
    pub username: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub wallet_background: Option<String>,
    pub activity_code: String,
    pub twitter_username: Option<String>,
    pub facebook_username: Option<String>,
    pub discord_username: Option<String>,
    pub identifier: Option<String>,
    pub mail: Option<String>, /* unique */
    pub google_id: Option<String>, /* unique */
    pub microsoft_id: Option<String>, /* unique */
    pub is_mail_verified: bool,
    pub is_phone_verified: bool,
    pub phone_number: Option<String>, /* unique */
    pub paypal_id: Option<String>, /* unique */
    pub account_number: Option<String>, /* unique */
    pub device_id: Option<String>, /* unique */
    pub social_id: Option<String>, /* unique */
    pub cid: Option<String>, /* unique */
    pub screen_cid: Option<String>, /* unique */
    pub snowflake_id: Option<i64>, /* unique */
    pub stars: Option<i64>,
    pub user_role: UserRole,
    pub token_time: Option<i64>,
    pub balance: Option<i64>,
    pub extra: Option<serde_json::Value>,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct UserData{
    pub id: i32,
    pub region: Option<String>,
    pub username: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub wallet_background: Option<String>,
    pub activity_code: String,
    pub twitter_username: Option<String>,
    pub facebook_username: Option<String>,
    pub discord_username: Option<String>,
    pub identifier: Option<String>,
    pub mail: Option<String>, /* unique */
    pub google_id: Option<String>, /* unique */
    pub microsoft_id: Option<String>, /* unique */
    pub is_mail_verified: bool,
    pub is_phone_verified: bool,
    pub phone_number: Option<String>, /* unique */
    pub paypal_id: Option<String>, /* unique */
    pub account_number: Option<String>, /* unique */
    pub device_id: Option<String>, /* unique */
    pub social_id: Option<String>, /* unique */
    pub cid: Option<String>, /* unique */
    pub screen_cid: Option<String>, /* unique */
    pub snowflake_id: Option<i64>, /* unique */
    pub stars: Option<i64>,
    pub user_role: String,
    pub token_time: Option<i64>,
    pub balance: Option<i64>,
    pub last_login: Option<String>,
    pub extra: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct UserWalletInfoResponse{
    pub username: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub banner: Option<String>,
    pub mail: Option<String>, /* unique */
    pub screen_cid: Option<String>, /* keccak256 */
    pub extra: Option<serde_json::Value>,
    pub stars: Option<i64>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct UserWalletInfoResponseWithBalance{
    pub username: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub banner: Option<String>,
    pub mail: Option<String>, /* unique */
    pub screen_cid: Option<String>, /* keccak256 */
    pub extra: Option<serde_json::Value>,
    pub stars: Option<i64>,
    pub balance: Option<i64>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct UserWalletInfoResponseForUserSuggestions{
    pub username: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub banner: Option<String>,
    pub mail: Option<String>, /* unique */
    pub screen_cid: Option<String>, /* keccak256 */
    pub extra: Option<serde_json::Value>,
    pub stars: Option<i64>,
    pub created_at: String,
    pub requested_at: Option<i64>,
    pub is_accepted: Option<bool>
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ForgotPasswordRequest{
    pub mail: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct NewPasswordRequest{
    pub new_password: String,
    pub old_password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct TopUsers{
    pub nfts_info: Vec<NftOwnerCount>,
    pub collections_info: Vec<CollectionOwnerCount>,
    pub private_galleries_infos: Vec<GalleryOwnerCount>,
    pub followers_info: Vec<FriendOwnerCount>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserIdResponse{
    pub id: i32,
    pub region: String,
    pub username: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub wallet_background: Option<String>,
    pub activity_code: String,
    pub twitter_username: Option<String>,
    pub facebook_username: Option<String>,
    pub discord_username: Option<String>,
    pub identifier: Option<String>,
    pub mail: Option<String>, /* unique */
    pub google_id: Option<String>, /* unique */
    pub microsoft_id: Option<String>, /* unique */
    pub is_mail_verified: bool,
    pub is_phone_verified: bool,
    pub phone_number: Option<String>, /* unique */
    pub paypal_id: Option<String>, /* unique */
    pub account_number: Option<String>, /* unique */
    pub device_id: Option<String>, /* unique */
    pub social_id: Option<String>, /* unique */
    pub cid: Option<String>, /* unique */
    pub screen_cid: Option<String>, /* keccak256 */
    pub snowflake_id: Option<i64>, /* unique */
    pub stars: Option<i64>,
    pub signer: Option<String>,
    pub mnemonic: Option<String>,
    pub user_role: String,
    pub token_time: Option<i64>,
    pub balance: Option<i64>,
    pub last_login: Option<String>,
    pub extra: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserChatRoomLaunchpadRequest{
    pub user_cid: String,
    pub chatroomlp_id: i32,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserMailInfos{
    pub ids: Vec<i32>,
    pub body: String,
    pub subject: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UpdateBioRequest{
    pub bio: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UpdateExtraRequest{
    /* 
        it can be any type of object cause we don't know the structure 
        of the frontend it's a json object 
    */
    pub extra: serde_json::Value, 
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NewIdRequest{
    pub username: String,
    pub device_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ChargeWalletRequest{
    pub user_id: i32,
    pub buyer_cid: String,
    pub tokens: i64,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CheckKycRequest{
    pub caller_cid: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserLoginWithGmailRequest{
    pub identifier: String,
    pub gavatar: String,
    pub gid: String,
    pub gusername: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserLoginWithMicrosoftRequest{
    pub identifier: String,
    pub mavatar: String,
    pub mid: String,
    pub musername: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Id{
    pub region: String,
    pub user_id: i32,
    pub device_id: String,
    pub username: String,
    pub new_snowflake_id: Option<i64>,
    pub new_cid: Option<String>, /* pubkey */
    pub screen_cid: Option<String>, /* keccak256 */
    pub signer: Option<String>, /* prvkey */
    pub mnemonic: Option<String>, /* mnemonic */
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CheckUserMailVerificationRequest{
    pub user_mail: String,
    pub verification_code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CheckUserPhoneVerificationRequest{
    pub user_phone: String,
    pub verification_code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LoginInfoRequest{
    pub username: String,
    pub password: String,
    pub device_id: String
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserLoginInfoRequest{
    pub identifier: String,
    pub password: String,
    pub device_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[derive(diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::Userrole"]
pub enum UserRole{
    Admin,
    #[default] /* we've considered the User variant as the default one */
    User,
    Dev
}

#[derive(Insertable)]
#[diesel(table_name=users)]
pub struct NewUser<'l> {
    pub username: &'l str,
    pub activity_code: &'l str,
    pub identifier: &'l str,
    pub user_role: UserRole,
    pub pswd: &'l str,
}

#[derive(Insertable)]
#[diesel(table_name=users)]
pub struct NewGooleUser<'l> {
    pub username: &'l str,
    pub identifier: &'l str,
    pub activity_code: &'l str,
    pub user_role: UserRole,
    pub avatar: Option<&'l str>,
    pub pswd: &'l str,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name=users)]
#[derive(Clone, Debug)]
pub struct EditUserByAdmin<'p>{
    pub user_role: UserRole,
    pub username: &'p str,
    pub identifier: &'p str,
    pub pswd: &'p str
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims{
    pub _id: i32, // mongodb object id
    pub user_role: UserRole,
    pub token_time: i64,
    pub is_refresh: bool,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued timestamp
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUserInfoRequest{
    pub username: String,
    pub identifier: String,
    pub role: String,
    pub password: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditUserByAdminRequest{
    pub user_id: i32,
    pub role: String,
    pub username: String,
    pub identifier: String,
    pub password: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SMSResponse{
    pub r#return: SMSResponseReturn, // use r# to escape reserved keywords to use them as identifiers in rust
    pub entries: Vec<SMSResponseEntries>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SMSResponseReturn{
    pub status: u16,
    pub message: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SMSResponseEntries{
    pub messageid: f64,
    pub message: String,
    pub status: u8,
    pub statustext: String,
    pub sender: String,
    pub receptor: String,
    pub date: i64,
    pub cost: u16, 
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct IpInfoResponse{
    pub ip: String,
    pub city: String,
    pub region: String, 
    pub country: String, 
    pub loc: String,
    pub org: String,
    pub timezone: String,
    
}

/** ------------------------------------ */
/**       GOOGLE OAUTH STRUCTURES        */
/** ------------------------------------ */
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GoogleQueryCode {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub device_id: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GoogleOAuthResponse{
    pub access_token: String,
    pub id_token: String
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GoogleUserResult{
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub locale: String,
}

/* 
    the error part of the following methods is of type Result<actix_web::HttpResponse, actix_web::Error>
    since in case of errors we'll terminate the caller with an error response like return Err(actix_ok_resp); 
    and pass its encoded form (utf8 bytes) directly through the socket to the client 
*/
impl User{

    pub async fn request_google_token(authorization_code: &str)
        -> Result<GoogleOAuthResponse, PanelHttpResponse>{

        let google_client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap();
        let google_client_secret = std::env::var("GOOGLE_OAUTH_CLIENT_SECRET").unwrap();
        let google_redirect_url = std::env::var("GOOGLE_OAUTH_REDIRECT_URL").unwrap();
        let google_oauth2_url = std::env::var("GOOGLE_OAUTH_ACCESS_TOKEN_URL").unwrap();
        
        let client = reqwest::Client::new();
        let params = &[ // array of tuples it can also be a map 
            ("grant_type", "authorization_code"),
            ("redirect_uri", google_redirect_url.as_str()),
            ("client_id", google_client_id.as_str()),
            ("code", authorization_code),
            ("client_secret", google_client_secret.as_str())
        ];

        // first check the status then decode into the desired structure
        let res = client.post(google_oauth2_url).form(params).send().await.unwrap();
        if res.status().is_success(){ // all status from 200 up to 299
            let oauth_response = res.json::<GoogleOAuthResponse>().await.unwrap();
            Ok(oauth_response)
        } else{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: &res.status().to_string(),
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }
    
    }

    pub async fn get_google_user(oauth_response: GoogleOAuthResponse) 
        -> Result<GoogleUserResult, PanelHttpResponse>{
            
        let GoogleOAuthResponse{access_token, id_token} = oauth_response;
        let google_oauth_user_info = std::env::var("GOOGLE_OAUTH_USER_INFO_URL").unwrap();
        let client = reqwest::Client::new();
        let mut url = reqwest::Url::parse(google_oauth_user_info.as_str()).unwrap();
        url.query_pairs_mut().append_pair("alt", "json");
        url.query_pairs_mut().append_pair("access_token", &access_token);

        // first check the status then decode into the desired structure
        let res = client.get(url).bearer_auth(id_token).send().await.unwrap();
        if res.status().is_success(){ // all status from 200 up to 299
            let user_info = res.json::<GoogleUserResult>().await.unwrap();
            Ok(user_info)
        } else{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: &res.status().to_string(),
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

    }

    pub async fn get_user_data_response_with_cookie(&self, device_id_: &str, redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<PanelHttpResponse, PanelHttpResponse>{

        /* generate cookie 🍪 from token time and jwt */
        /* since generate_cookie_and_jwt() takes the ownership of the user instance we must clone it then call this */
        let keys_info = self.clone().generate_cookie_and_jwt().unwrap();
        let cookie_token_time = keys_info.1;
        let jwt = keys_info.2;

        let now = chrono::Local::now().naive_local();
        let updated_user = diesel::update(users.find(self.id))
            .set((last_login.eq(now), token_time.eq(cookie_token_time)))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            .unwrap();

        // updating users_logins table with the latest jwt
        let get_login_info = UserLogin::upsert(
            NewUserLoginRequest{
                user_id: self.id,
                device_id: device_id_.to_string(),
                jwt: {
                    // store the hash of the token time in db in place of jwt
                    let toke_time_hash = format!("{}", cookie_token_time);
                    let mut hasher = Sha256::new();
                    hasher.update(toke_time_hash.as_str());
                    let time_hash = hasher.finalize();
                    hex::encode(time_hash.to_vec())
                },
                last_login: Some(chrono::Local::now().naive_local()),
            }, connection).await;

        let Ok(login_info) = get_login_info else{
            let err_resp = get_login_info.unwrap_err();
            return Err(err_resp);
        };
        
        let user_login_data = UserData{
            id: updated_user.id,
            region: updated_user.region.clone(),
            username: updated_user.username.clone(),
            bio: updated_user.bio.clone(),
            avatar: updated_user.avatar.clone(),
            banner: updated_user.banner.clone(),
            wallet_background: updated_user.wallet_background.clone(),
            activity_code: updated_user.activity_code.clone(),
            twitter_username: updated_user.twitter_username.clone(),
            facebook_username: updated_user.facebook_username.clone(),
            discord_username: updated_user.discord_username.clone(),
            identifier: updated_user.identifier.clone(),
            user_role: {
                match self.user_role.clone(){
                    UserRole::Admin => "Admin".to_string(),
                    UserRole::User => "User".to_string(),
                    _ => "Dev".to_string(),
                }
            },
            token_time: updated_user.token_time,
            balance: updated_user.balance,
            last_login: { 
                if updated_user.last_login.is_some(){
                    Some(updated_user.last_login.unwrap().to_string())
                } else{
                    Some("".to_string())
                }
            },
            created_at: updated_user.created_at.to_string(),
            updated_at: updated_user.updated_at.to_string(),
            mail: updated_user.mail,
            google_id: updated_user.google_id,
            microsoft_id: updated_user.microsoft_id,
            is_mail_verified: updated_user.is_mail_verified,
            is_phone_verified: updated_user.is_phone_verified,
            phone_number: updated_user.phone_number,
            paypal_id: updated_user.paypal_id,
            account_number: updated_user.account_number,
            device_id: updated_user.device_id,
            social_id: updated_user.social_id,
            cid: updated_user.cid,
            screen_cid: updated_user.screen_cid,
            snowflake_id: updated_user.snowflake_id,
            stars: updated_user.stars,
            extra: updated_user.extra,
        };

        /* ----------------------------------------------- */
        /* --------- publish updated user to redis channel */
        /* ----------------------------------------------- */
        /* 
            once the user updates his info we'll publish new updated user to redis channel and in
            other parts we start to subscribe to the new updated user topic then once we receive 
            the new user we'll start updating user fans and user nfts 
        */
        
        let json_stringified_updated_user = serde_json::to_string_pretty(&user_login_data).unwrap();
        events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;

        let resp = Response{
            data: Some(user_login_data),
            message: LOGGEDIN,
            status: 200,
            is_error: false,
        };
        return Ok(
            Ok(
                HttpResponse::Ok()
                    .cookie(keys_info.0.clone())
                    .append_header(("cookie", keys_info.0.value()))
                    .json(
                        resp
                    )
            )
        );
                    
    }

    pub async fn get_user_data_response_with_cookie_and_redirect_header(&self, device_id_: &str, redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection, state: &str) -> Result<PanelHttpResponse, PanelHttpResponse>{

        /* generate cookie 🍪 from token time and jwt */
        /* since generate_cookie_and_jwt() takes the ownership of the user instance we must clone it then call this */
        let keys_info = self.clone().generate_cookie_and_jwt().unwrap();
        let cookie_token_time = keys_info.1;
        let jwt = keys_info.2;

        let now = chrono::Local::now().naive_local();
        let updated_user = diesel::update(users.find(self.id))
            .set((last_login.eq(now), token_time.eq(cookie_token_time)))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            .unwrap();

        // updating users_logins table with the latest jwt
        let get_login_info = UserLogin::upsert(
            NewUserLoginRequest{
                user_id: self.id,
                device_id: device_id_.to_string(),
                jwt: {
                    // store the hash of the token time in db in place of jwt
                    let toke_time_hash = format!("{}", cookie_token_time);
                    let mut hasher = Sha256::new();
                    hasher.update(toke_time_hash.as_str());
                    let time_hash = hasher.finalize();
                    hex::encode(time_hash.to_vec())
                },
                last_login: Some(chrono::Local::now().naive_local()),
            }, connection).await;

        let Ok(login_info) = get_login_info else{
            let err_resp = get_login_info.unwrap_err();
            return Err(err_resp);
        };
        
        let user_login_data = UserData{
            id: updated_user.id,
            region: updated_user.region.clone(),
            username: updated_user.username.clone(),
            bio: updated_user.bio.clone(),
            avatar: updated_user.avatar.clone(),
            banner: updated_user.banner.clone(),
            wallet_background: updated_user.wallet_background.clone(),
            activity_code: updated_user.activity_code.clone(),
            twitter_username: updated_user.twitter_username.clone(),
            facebook_username: updated_user.facebook_username.clone(),
            discord_username: updated_user.discord_username.clone(),
            identifier: updated_user.identifier.clone(),
            user_role: {
                match self.user_role.clone(){
                    UserRole::Admin => "Admin".to_string(),
                    UserRole::User => "User".to_string(),
                    _ => "Dev".to_string(),
                }
            },
            token_time: updated_user.token_time,
            balance: updated_user.balance,
            last_login: { 
                if updated_user.last_login.is_some(){
                    Some(updated_user.last_login.unwrap().to_string())
                } else{
                    Some("".to_string())
                }
            },
            created_at: updated_user.created_at.to_string(),
            updated_at: updated_user.updated_at.to_string(),
            mail: updated_user.mail,
            google_id: updated_user.google_id,
            microsoft_id: updated_user.microsoft_id,
            is_mail_verified: updated_user.is_mail_verified,
            is_phone_verified: updated_user.is_phone_verified,
            phone_number: updated_user.phone_number,
            paypal_id: updated_user.paypal_id,
            account_number: updated_user.account_number,
            device_id: updated_user.device_id,
            social_id: updated_user.social_id,
            cid: updated_user.cid,
            screen_cid: updated_user.screen_cid,
            snowflake_id: updated_user.snowflake_id,
            stars: updated_user.stars,
            extra: updated_user.extra,
        };

        /* ----------------------------------------------- */
        /* --------- publish updated user to redis channel */
        /* ----------------------------------------------- */
        /* 
            once the user updates his info we'll publish new updated user to redis channel and in
            other parts we start to subscribe to the new updated user topic then once we receive 
            the new user we'll start updating user fans and user nfts 
        */
        
        let json_stringified_updated_user = serde_json::to_string_pretty(&user_login_data).unwrap();
        events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;

        let resp = Response{
            data: Some(user_login_data),
            message: LOGGEDIN,
            status: 200,
            is_error: false,
        };
        let redirect_url = format!("{}?data={}", state, jwt);
        return Ok(
            Ok(
                HttpResponse::Ok()
                    .cookie(keys_info.0.clone())
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .append_header(("cookie", keys_info.0.value()))
                    .append_header((actix_web::http::header::LOCATION, redirect_url))
                    .json(
                        "Redirecting..."
                    )
            )
        );
                    
    }

    pub fn decode_token(token: &str) -> Result<TokenData<JWTClaims>, jsonwebtoken::errors::Error>{
        let encoding_key = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let decoded_token = decode::<JWTClaims>(token, &DecodingKey::from_secret(encoding_key.as_bytes()), &Validation::new(Algorithm::HS512));
        decoded_token
    }

    pub const SCHEMA_NAME: &'static str = "User";
    pub const fn get_schema_name() -> &'static str{ Self::SCHEMA_NAME }

    pub async fn passport(req: HttpRequest, pass_role: Option<UserRole>, connection: &mut DbPoolConnection) -> Result<JWTClaims, PanelHttpResponse>{

        let mut jwt_token = ""; 

        let Some(authen_header) = req.headers().get("Authorization") else{

            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: NOT_FOUND_COOKIE_VALUE_OR_JWT,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );

        };

        let auth_header_value = authen_header.to_str().unwrap();
        if auth_header_value.starts_with("bearer") ||
            auth_header_value.starts_with("Bearer"){

            jwt_token = auth_header_value[6..auth_header_value.len()].trim();

        }


        /* decoding the jwt */
        let token_result = User::decode_token(jwt_token);
        
        match token_result{
            Ok(token) => {

                /* cookie time is not expired yet */
                let token_data = token.claims;
                let _id = token_data._id;
                let role = token_data.user_role.clone();
                let _token_time = token_data.token_time; /* if a user do a login this will be reset and the last JWT will be invalid */
                let exp_time = token_data.exp;

                // ------------------------------------------------------------------
                // the exp_time must not be greater than now so a JWT be a valid one
                // ------------------------------------------------------------------
                if Utc::now().timestamp_nanos_opt().unwrap() > exp_time{
                    let resp = Response{
                        data: Some(_id.to_owned()),
                        message: EXPIRED_JWT,
                        status: 406,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    );
                } 

                /* fetch user info based on the data inside jwt */ 
                let single_user = users
                    .filter(id.eq(_id))
                    .first::<User>(connection);

                if single_user.is_err(){
                    let resp = Response{
                        data: Some(_id.to_owned()),
                        message: USER_NOT_FOUND,
                        status: 404,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::NotFound().json(resp))
                    );
                }

                let user = single_user.unwrap();

                /* 
                    check that the user is authorized with the 
                    passed in role and the one inside the jwt
                    since some of the routes require role guard 
                */
                if pass_role.is_some(){
                    if user.user_role != pass_role.unwrap() &&
                        user.user_role != role{
                        let resp = Response{
                            data: Some(_id.to_owned()),
                            message: ACCESS_DENIED,
                            status: 403,
                            is_error: true
                        };
                        return Err(
                            Ok(HttpResponse::Forbidden().json(resp))
                        );
                    } 
                }

                //// -----------------------------
                //// JWT PER DEVICE AUTHENTICATION
                //// -----------------------------
                /*
                    on each login we’ll generate a new token time that will be stored 
                    in jwt itself and in a separate table along with its device id so 
                    if a user wants to to logout we’ll set the token time related to 
                    the passed in device id to 0 and in the next request we’ll check 
                    the token time inside the jwt with the one inside that table if we 
                    found a match means user has not logged out yet otherwise he did a 
                    logout and the current jwt is invalid, note that in all these logic 
                    the hash of the token time will be inside the table  
                */
                let get_user_login_infos = UserLogin::find_by_user_id(_id, connection).await;
                let Ok(user_login_infos) = get_user_login_infos else{
                    let err_resp = get_user_login_infos.unwrap_err();
                    return Err(err_resp);
                };

                let mut found_token_in_db = String::from("");
                for login_info in user_login_infos{
                    // since there are jwt per each device thus if we found 
                    // a jwt means that this user didn't logout from that 
                    // device yet
                    let toke_time_hash = format!("{}", _token_time);
                    let mut hasher = Sha256::new();
                    hasher.update(toke_time_hash.as_str());
                    let time_hash = hasher.finalize();
                    let token_time_hash_hex = hex::encode(time_hash.to_vec());

                    if login_info.jwt == token_time_hash_hex{
                        /* returning token data, if we're here means that nothing went wrong */
                        found_token_in_db = login_info.jwt;
                    } 
                }
                // we didn't found jwt in db cause user might has logged out 
                // from his device related to this jwt
                if found_token_in_db.is_empty(){
                    let resp = Response{
                        data: Some(_id.to_owned()),
                        message: INVALID_JWT, /* comple the user to login again to set a new token time in his/her jwt */
                        status: 403,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::Forbidden().json(resp))
                    );
                    
                }

                return Ok(token_data);

            },
            Err(e) => {
                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: &e.to_string(),
                    status: 500,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::InternalServerError().json(resp))
                );
            }
        }

    }

    fn generate_tokens(&self, _token_time: i64) -> (
        Result<String, jsonwebtoken::errors::Error>,
        Result<String, jsonwebtoken::errors::Error>
    ){

        let encoding_key = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let now = Utc::now();
        let access_exp_time = now + chrono::Duration::days(30); // logout every month
        
        // -------------------------------------------------
        //    access token payload, will be used to login 
        // ------------------------------------------------
        let access_token_payload = JWTClaims{
            _id: self.id,
            /* 
                if a user role is changed by the admin, user must logout 
                then login again since the jwt must be updated with new role
            */
            user_role: self.user_role.clone(),
            token_time: _token_time,
            exp: access_exp_time.timestamp_nanos_opt().unwrap(),
            iat: now.timestamp_nanos_opt().unwrap(),
            is_refresh: false,
        };

        let access_token = encode(
            &Header::new(Algorithm::HS512), 
            &access_token_payload, 
            &EncodingKey::from_secret(encoding_key.as_bytes())
        );
        
        // ---------------------------------------------------------------------
        //    refresh token payload, will be used to generate new set of tokens 
        // ---------------------------------------------------------------------
        let refresh_exp_time = access_exp_time + chrono::Duration::minutes(10);
        let refresh_token_payload = JWTClaims{
            _id: self.id,
            /* 
                if a user role is changed by the admin, user must logout 
                then login again since the jwt must be updated with new role
            */
            user_role: self.user_role.clone(),
            token_time: 0,
            exp: refresh_exp_time.timestamp_nanos_opt().unwrap(),
            iat: Utc::now().timestamp_nanos_opt().unwrap(),
            is_refresh: true,
        };

        let refresh_token = encode(
            &Header::new(Algorithm::HS512), 
            &refresh_token_payload, 
            &EncodingKey::from_secret(encoding_key.as_bytes())
        );

        (access_token, refresh_token)
    
    }

    /* >-----------------------------------------------------------------------------------------------------------
        since self is not behind & thus the Cookie can't use the lifetime of the self reference hence we 
        must specify the 'static lifetime for the Cookie also the reason that the self is not behind a pointer
        is because this function returns a Cookie instance which takes a valid lifetime in which we can't return
        it from the the caller space of this method since rust says can't returns a value referencing data owned by 
        the current function means that the returned cookie instance from here to the caller space has a reference 
        to the instance of User struct in which we can't return the cookie instance from the caller scope to other 
        scopes in other words we can't return reference to a data which is owned by the current function. 
    */
    pub fn generate_cookie_and_jwt(self) -> Option<(Cookie<'static>, i64, String, String)>{

        /*
            since cookie can be stored inside the request object thus for peers on the same network 
            which have an equal ip address they share a same cookie thus we'll face the bug of which 
            every user can be every user in which they can see other peer's jwt info inside their browser
            which allows them to be inside each other panel!
            
            let time_hash = walletreq::exports::get_sha256_from(&time_hash_now_now_str);
        */
        let time_hash_now = chrono::Local::now().timestamp_nanos_opt().unwrap();
        let time_hash_now_now_str = format!("{}", time_hash_now);
        let mut hasher = Sha256::new();
        hasher.update(time_hash_now_now_str.as_str());
        let time_hash = hasher.finalize();

        /* every 2 chars is 1 byte thus in sha256 we have 32 bytes elements which is 64 chars in hex */
        let time_hash_hex_string = time_hash
                                        .into_iter()
                                        /* mapping each byte into its corresponding hex char */
                                        .map(|byte| format!("{:02x}", byte))
                                        .collect::<String>();
        
        // let time_hash_hex_string = hex::encode(&time_hash);

        /* if we're here means that the password was correct */
        let get_tokens = self.generate_tokens(time_hash_now);
        let access_token = get_tokens.0.unwrap();
        let refresh_token = get_tokens.1.unwrap();
        
        let cookie_value = format!("/accesstoken={access_token:}&accesstoken_time={time_hash_hex_string:}&refreshtoken={refresh_token:}");
        let mut cookie = Cookie::build("jwt", cookie_value)
                                    .same_site(cookie::SameSite::Strict)
                                    .secure(true)
                                    .finish();
        let cookie_exp_days = env::var("COOKIE_EXPIRATION_DAYS").expect("⚠️ no cookie exporation days variable set").parse::<i64>().unwrap();
        let mut now = OffsetDateTime::now_utc();
        now += Duration::days(cookie_exp_days);
        cookie.set_expires(now); /* will be invalid 30 days from now on */

        Some((cookie, time_hash_now, access_token, refresh_token))

    }

    fn check_cookie_time_hash(&self, cookie_time_hash: &str) -> bool{
        
        let Some(utt) = self.token_time else{
            return false;
        };

        let utt_string = format!("{}", utt);
        let mut hasher = Sha256::new();
        hasher.update(utt_string.as_str());
        let time_hash = hasher.finalize();

        /* every 2 chars is 1 byte thus in sha256 we have 32 bytes elements which is 64 chars in hex */
        let time_hash_hex_string = time_hash
                                        .into_iter()
                                        .map(|byte| format!("{:02x}", byte))
                                        .collect::<String>();
        
        // let time_hash_hex_string = hex::encode(&time_hash);

        if time_hash_hex_string == cookie_time_hash{
            true
        } else{
            false
        }

    }

    pub fn hash_pswd(password: &str) -> Result<String, argon2::Error>{ /* argon2 as the kdf */
        let salt = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let salt_bytes = salt.as_bytes();
        let password_bytes = password.as_bytes();
        argon2::hash_encoded(password_bytes, salt_bytes, &argon2::Config::default())
    }

    pub fn verify_pswd(&self, raw_pswd: &str) -> Result<bool, argon2::Error>{ /* argon2 as the kdf */
        let password_bytes = raw_pswd.as_bytes();
        Ok(argon2::verify_encoded(&self.pswd, password_bytes).unwrap())
    }

    pub async fn find_by_username(user_name: &str, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(username.eq(user_name.to_string()))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(user_name),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn find_by_username_or_mail_or_scid(recipient_info: &str, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(
                username.eq(recipient_info.to_string().to_lowercase())
                .or(mail.eq(recipient_info.to_string().to_lowercase()))
                .or(screen_cid.eq(recipient_info.to_string().to_lowercase()))
            )
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(recipient_info),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn fetch_wallet_by_username_or_mail_or_scid_or_cid(user_info: &str, 
        connection: &mut PooledConnection<ConnectionManager<PgConnection>>) -> Result<UserWalletInfoResponse, PanelHttpResponse>{

        let single_user = users
            .filter(
                username.eq(user_info.to_string())
                .or(mail.eq(user_info.to_string()))
                .or(cid.eq(user_info.to_string()))
                .or(screen_cid.eq(user_info.to_string()))
            )
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(user_info),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(
            UserWalletInfoResponse{ 
                username: user.username, 
                avatar: user.avatar,
                mail: user.mail, 
                screen_cid: user.screen_cid, 
                stars: user.stars, 
                created_at: user.created_at.to_string(),
                bio: user.bio,
                banner: user.banner, 
                extra: user.extra,
            }
        )

    }

    pub async fn fetch_wallet_by_username_or_mail_or_scid(user_info: &str, 
        connection: &mut DbPoolConnection) -> Result<UserWalletInfoResponse, PanelHttpResponse>{

        let single_user = users
            .filter(
                username.eq(user_info.to_string())
                .or(mail.eq(user_info.to_string()))
                .or(screen_cid.eq(user_info.to_string()))
            )
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(user_info),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(
            UserWalletInfoResponse{ 
                username: user.username, 
                avatar: user.avatar,
                mail: user.mail, 
                screen_cid: user.screen_cid, 
                stars: user.stars, 
                created_at: user.created_at.to_string(),
                bio: user.bio,
                banner: user.banner, 
                extra: user.extra,
            }
        )

    }

    pub async fn get_top_users(connection: &mut DbPoolConnection, 
        redis_client: RedisClient,
        limit: web::Query<Limit>) 
        -> Result<TopUsers, PanelHttpResponse>{

        let get_users = User::get_all(connection, limit.clone()).await;
        let Ok(all_users) = get_users else{
            let err_resp = get_users.unwrap_err();
            return Err(err_resp);
        };
            
        let get_owners_with_most_nfts = UserNft::get_owners_with_lots_of_nfts(all_users.clone(), connection).await;
        let Ok(owners_with_most_nfts) = get_owners_with_most_nfts else{
            let err_resp = get_owners_with_most_nfts.unwrap_err();
            return Err(err_resp);
        };

        let get_owners_with_most_collections = UserCollection::get_owners_with_lots_of_collections(all_users.clone(), connection).await;
        let Ok(owners_with_most_collections) = get_owners_with_most_collections else{
            let err_resp = get_owners_with_most_collections.unwrap_err();
            return Err(err_resp);
        };

        let get_owners_with_most_private_galleries = UserPrivateGallery::get_owners_with_lots_of_galleries(all_users.clone(), redis_client.clone(), connection).await;
        let Ok(owners_with_most_private_galleries) = get_owners_with_most_private_galleries else{
            let err_resp = get_owners_with_most_private_galleries.unwrap_err();
            return Err(err_resp);
        };

        let get_owners_with_most_followers = UserFan::get_owners_with_lots_of_followers(all_users.clone(), limit.clone(), connection).await;
        let Ok(owners_with_most_followers) = get_owners_with_most_followers else{
            let err_resp = get_owners_with_most_followers.unwrap_err();
            return Err(err_resp);
        };

        Ok(
            TopUsers{
                nfts_info: owners_with_most_nfts,
                collections_info: owners_with_most_collections,
                private_galleries_infos: owners_with_most_private_galleries,
                followers_info: owners_with_most_followers,
            }
        ) 

    }

    pub async fn fetch_all_users_wallet_info(limit: web::Query<Limit>, 
        connection: &mut DbPoolConnection) 
        -> Result<Vec<UserWalletInfoResponseWithBalance>, PanelHttpResponse>{

            let from = limit.from.unwrap_or(0);
            let to = limit.to.unwrap_or(10);
    
            if to < from {
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_QUERY_LIMIT,
                    status: 406,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                )
            }
            
            match users
                .order(created_at.desc())
                .offset(from)
                .limit((to - from) + 1)
                .load::<User>(connection)
            {
                Ok(all_users) => {

                    Ok(
                        all_users
                        .into_iter()
                        .map(|u|{

                            UserWalletInfoResponseWithBalance{
                                username: u.username,
                                avatar: u.avatar,
                                bio: u.bio,
                                banner: u.banner,
                                mail: u.mail,
                                screen_cid: u.screen_cid,
                                stars: u.stars,
                                created_at: u.created_at.to_string(),
                                balance: u.balance,
                                extra: u.extra,
                            }

                        })
                        .collect::<Vec<UserWalletInfoResponseWithBalance>>()
                    )

                },
                Err(e) => {

                    let resp_err = &e.to_string();
    
    
                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::fetch_all_users_wallet_info");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
    
                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
    
                }
            }


    }

    pub async fn suggest_user_to_owner(limit: web::Query<Limit>, owner_screen_cid: &str,
        connection: &mut DbPoolConnection) 
        -> Result<Vec<UserWalletInfoResponseForUserSuggestions>, PanelHttpResponse>{

            let from = limit.from.unwrap_or(0) as usize;
            let to = limit.to.unwrap_or(10) as usize;
    
            if to < from {
                let resp = Response::<'_, &[u8]>{
                    data: Some(&[]),
                    message: INVALID_QUERY_LIMIT,
                    status: 406,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                )
            }
            
            match users
                .order(created_at.desc())
                .load::<User>(connection)
            {
                Ok(all_users) => {

                    let mut suggestions = vec![];
                    for user in all_users{

                        // might be admin, dev or the user didn't create wallet yet
                        // also don't suggest the user to himself
                        if user.screen_cid.is_none() ||
                            (user.screen_cid.is_some() && user.screen_cid.as_ref().unwrap() == owner_screen_cid){
                                continue;
                            }

                        // get all friends data of user, push to suggestions
                        let get_user_fan_data = UserFan::get_user_fans_data_for(user.screen_cid.as_ref().unwrap(), connection).await;
                        if get_user_fan_data.is_ok(){
                            let user_friends = get_user_fan_data.as_ref().unwrap();
                            let friends_data = user_friends.clone().construct_friends_data(connection);
                            let decoded_friends_data = if friends_data.is_some(){
                                serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
                            } else{
                                vec![]
                            };

                            let mut requested_at: Option<i64> = None;
                            let mut is_accepted: Option<bool> = None;
                            for friend in decoded_friends_data{
                                if friend.screen_cid == owner_screen_cid{
                                    requested_at = Some(friend.requested_at);
                                    is_accepted = Some(friend.is_accepted);
                                    break;
                                }
                            }

                            suggestions.push(
                                UserWalletInfoResponseForUserSuggestions{
                                    username: user.clone().username,
                                    avatar: user.clone().avatar,
                                    bio: user.clone().bio,
                                    banner: user.clone().banner,
                                    mail: user.clone().mail,
                                    screen_cid: user.clone().screen_cid,
                                    stars: user.clone().stars,
                                    created_at: user.clone().created_at.to_string(),
                                    extra: user.clone().extra,
                                    requested_at, // to know whether owner_screen_cid has sent a request to friend or not
                                    is_accepted // to know whether the user.screen_cid has accepted the request of owner_screen_cid or not
                                }
                            )
                            
                        } else{
                            suggestions.push(
                                UserWalletInfoResponseForUserSuggestions{
                                    username: user.clone().username,
                                    avatar: user.clone().avatar,
                                    bio: user.clone().bio,
                                    banner: user.clone().banner,
                                    mail: user.clone().mail,
                                    screen_cid: user.clone().screen_cid,
                                    stars: user.clone().stars,
                                    created_at: user.clone().created_at.to_string(),
                                    extra: user.clone().extra,
                                    requested_at: None, // to know whether owner_screen_cid has sent a request to friend or not
                                    is_accepted: None // to know whether the user.screen_cid has accepted the request of owner_screen_cid or not
                                }
                            )
                        }

                    }

                    suggestions.sort_by(|s1, s2|{

                        let s1_created_at = NaiveDateTime
                            ::parse_from_str(&s1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                            .unwrap();

                        let s2_created_at = NaiveDateTime
                            ::parse_from_str(&s2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                            .unwrap();

                        s2_created_at.cmp(&s1_created_at)
        
                    });
                    

                    let sliced = if from < suggestions.len(){
                        if suggestions.len() > to{
                            let data = &suggestions[from..to+1];
                            data.to_vec()
                        } else{
                            let data = &suggestions[from..suggestions.len()];
                            data.to_vec()
                        }
                    } else{
                        vec![]
                    };


                    Ok(
                        sliced
                    )


                },
                Err(e) => {

                    let resp_err = &e.to_string();
    
    
                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::suggest_user_to_owner");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
    
                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
    
                }
            }


    }

    pub async fn suggest_user_to_owner_without_limit(owner_screen_cid: &str,
        connection: &mut DbPoolConnection) 
        -> Result<Vec<UserWalletInfoResponseForUserSuggestions>, PanelHttpResponse>{
            
            match users
                .order(created_at.desc())
                .load::<User>(connection)
            {
                Ok(all_users) => {

                    let mut suggestions = vec![];
                    for user in all_users{

                        // might be admin, dev or the user didn't create wallet yet
                        // also don't suggest the user to himself
                        if user.screen_cid.is_none() ||
                            (user.screen_cid.is_some() && user.screen_cid.as_ref().unwrap() == owner_screen_cid){
                                continue;
                            }

                        // get all friends data of user, push to suggestions
                        let get_user_fan_data = UserFan::get_user_fans_data_for(user.screen_cid.as_ref().unwrap(), connection).await;
                        if get_user_fan_data.is_ok(){
                            let user_friends = get_user_fan_data.as_ref().unwrap();
                            let friends_data = user_friends.clone().construct_friends_data(connection);
                            let decoded_friends_data = if friends_data.is_some(){
                                serde_json::from_value::<Vec<FriendData>>(friends_data.clone().unwrap()).unwrap()
                            } else{
                                vec![]
                            };

                            let mut requested_at: Option<i64> = None;
                            let mut is_accepted: Option<bool> = None;
                            for friend in decoded_friends_data{
                                if friend.screen_cid == owner_screen_cid{
                                    requested_at = Some(friend.requested_at);
                                    is_accepted = Some(friend.is_accepted);
                                    break;
                                }
                            }

                            suggestions.push(
                                UserWalletInfoResponseForUserSuggestions{
                                    username: user.clone().username,
                                    avatar: user.clone().avatar,
                                    bio: user.clone().bio,
                                    banner: user.clone().banner,
                                    mail: user.clone().mail,
                                    screen_cid: user.clone().screen_cid,
                                    stars: user.clone().stars,
                                    created_at: user.clone().created_at.to_string(),
                                    extra: user.clone().extra,
                                    requested_at, // to know whether owner_screen_cid has sent a request to friend or not
                                    is_accepted // to know whether the user.screen_cid has accepted the request of owner_screen_cid or not
                                }
                            )
                            
                        } else{
                            suggestions.push(
                                UserWalletInfoResponseForUserSuggestions{
                                    username: user.clone().username,
                                    avatar: user.clone().avatar,
                                    bio: user.clone().bio,
                                    banner: user.clone().banner,
                                    mail: user.clone().mail,
                                    screen_cid: user.clone().screen_cid,
                                    stars: user.clone().stars,
                                    created_at: user.clone().created_at.to_string(),
                                    extra: user.clone().extra,
                                    requested_at: None, // to know whether owner_screen_cid has sent a request to friend or not
                                    is_accepted: None // to know whether the user.screen_cid has accepted the request of owner_screen_cid or not
                                }
                            )
                        }

                    }

                    suggestions.sort_by(|s1, s2|{

                        let s1_created_at = NaiveDateTime
                            ::parse_from_str(&s1.created_at, "%Y-%m-%d %H:%M:%S%.f")
                            .unwrap();

                        let s2_created_at = NaiveDateTime
                            ::parse_from_str(&s2.created_at, "%Y-%m-%d %H:%M:%S%.f")
                            .unwrap();

                        s2_created_at.cmp(&s1_created_at)
        
                    });

                    Ok(
                        suggestions
                    )

                },
                Err(e) => {

                    let resp_err = &e.to_string();
    
    
                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::suggest_user_to_owner");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
    
                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );
    
                }
            }


    }

    pub async fn find_by_mail(user_mail: &str, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(mail.eq(user_mail.to_string()))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(user_mail),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn find_by_phone(user_phone: &str, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(phone_number.eq(user_phone.to_string()))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(user_phone),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn find_by_identifier(identifier_login: &str, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(identifier.eq(identifier_login.to_string()))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(identifier_login),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn find_by_identifier_or_mail(identifier_login: &str, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(
                identifier.eq(identifier_login.to_string())
                .or(mail.eq(identifier_login.to_string()))
            )
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(identifier_login),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(
            user
        )

    }

    pub async fn find_by_id(doer_id: i32, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(users::id.eq(doer_id))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(doer_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub fn find_by_id_none_async(doer_id: i32, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(users::id.eq(doer_id))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(doer_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn find_by_screen_cid(user_screen_cid: &str, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(users::screen_cid.eq(user_screen_cid))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(user_screen_cid),
                message: RECIPIENT_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub fn find_by_screen_cid_none_async(user_screen_cid: &str, connection: &mut DbPoolConnection) -> Result<Self, PanelHttpResponse>{

        let single_user = users
            .filter(users::screen_cid.eq(user_screen_cid))
            .first::<User>(connection);
                        
        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(user_screen_cid),
                message: RECIPIENT_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        Ok(user)

    }

    pub async fn insert(identifier_login: String, connection: &mut DbPoolConnection) -> Result<(UserData, Cookie), PanelHttpResponse>{

        let random_chars = gen_random_chars(gen_random_number(5, 11));
        let random_code: String = (0..5).map(|_|{
            let idx = gen_random_idx(random::<u8>() as usize); // idx is one byte cause it's of type u8
            CHARSET[idx] as char // CHARSET is of type utf8 bytes thus we can index it which it's length is 10 bytes (0-9)
        }).collect();

        let new_user = NewUser{
            username: &identifier_login.to_lowercase(), /* first insert the username is the identifier address */
            activity_code: &random_code,
            identifier: &identifier_login.to_lowercase(),
            user_role: UserRole::User,
            pswd: "",
        };
        
        match diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result::<User>(connection)
            {
                Ok(fetched_user) => {

                    let user_login_data = UserData{
                        id: fetched_user.id,
                        region: fetched_user.region.clone(),
                        username: fetched_user.username.clone(),
                        bio: fetched_user.bio.clone(),
                        avatar: fetched_user.avatar.clone(),
                        banner: fetched_user.banner.clone(),
                        wallet_background: fetched_user.wallet_background.clone(),
                        activity_code: fetched_user.activity_code.clone(),
                        twitter_username: fetched_user.twitter_username.clone(),
                        facebook_username: fetched_user.facebook_username.clone(),
                        discord_username: fetched_user.discord_username.clone(),
                        identifier: fetched_user.identifier.clone(),
                        user_role: {
                            match fetched_user.user_role.clone(){
                                UserRole::Admin => "Admin".to_string(),
                                UserRole::User => "User".to_string(),
                                _ => "Dev".to_string(),
                            }
                        },
                        token_time: fetched_user.token_time,
                        balance: fetched_user.balance,
                        last_login: { 
                            if fetched_user.last_login.is_some(){
                                Some(fetched_user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: fetched_user.created_at.to_string(),
                        updated_at: fetched_user.updated_at.to_string(),
                        mail: fetched_user.clone().mail,
                        google_id: fetched_user.clone().google_id,
                        microsoft_id: fetched_user.clone().microsoft_id,
                        is_mail_verified: fetched_user.is_mail_verified,
                        is_phone_verified: fetched_user.is_phone_verified,
                        phone_number: fetched_user.clone().phone_number,
                        paypal_id: fetched_user.clone().paypal_id,
                        account_number: fetched_user.clone().account_number,
                        device_id: fetched_user.clone().device_id,
                        social_id: fetched_user.clone().social_id,
                        cid: fetched_user.clone().cid,
                        screen_cid: fetched_user.clone().screen_cid,
                        snowflake_id: fetched_user.snowflake_id,
                        stars: fetched_user.stars,
                        extra: fetched_user.clone().extra,
                    };

                    /* generate cookie 🍪 from token time and jwt */
                    /* since generate_cookie_and_jwt() takes the ownership of the user instance we must clone it then call this */
                    /* generate_cookie_and_jwt() returns a Cookie instance with a 'static lifetime which allows us to return it from here*/
                    let Some(cookie_info) = fetched_user.clone().generate_cookie_and_jwt() else{
                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: CANT_GENERATE_COOKIE,
                            status: 500,
                            is_error: true,
                        };
                        return Err(
                            Ok(HttpResponse::InternalServerError().json(resp))
                        );
                    };
                    
                    let cookie = cookie_info.0;
                    let cookie_token_time = cookie_info.1;

                    /* update the login token time */
                    let now = chrono::Local::now().naive_local();
                    let updated_user = diesel::update(users.find(fetched_user.id))
                        .set((last_login.eq(now), token_time.eq(cookie_token_time)))
                        .execute(connection)
                        .unwrap();
                    
                    Ok((user_login_data, cookie))

                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::insert");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }
    
    }

    pub async fn insert_by_identifier_password(identifier_login: String, password: String, connection: &mut DbPoolConnection) -> Result<(UserData, Cookie), PanelHttpResponse>{

        let random_chars = gen_random_chars(gen_random_number(5, 11));
        let random_code: String = (0..5).map(|_|{
            let idx = gen_random_idx(random::<u8>() as usize); // idx is one byte cause it's of type u8
            CHARSET[idx] as char // CHARSET is of type utf8 bytes thus we can index it which it's length is 10 bytes (0-9)
        }).collect();

        if !helpers::misc::is_password_valid(&password){
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: REGEX_PASSWORD_ISSUE,
                status: 406,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        };

        let pass = User::hash_pswd(password.as_str()).unwrap();
        let new_user = NewUser{
            /* 
                we'll fill the username with the current timestamp because 
                otherwise if a user logged in to the app with a username 
                as the identifier, the username field also will be the same 
                as the identifier field, this is actually a bug cause if a
                user wants to create a cid we're checking that if the cid 
                field is not empty then we'll insert the passed in username 
                along with other fields into the db and since the username 
                is already registered with the identifier, database says
                that this username is already exists regardless of who is 
                creating the cid, cause we don't check the username and cid
                against the new incomng request in cid api.
            */
            username: &chrono::Local::now().timestamp_nanos_opt().unwrap().to_string(),
            activity_code: &random_code,
            identifier: &identifier_login.to_lowercase(),
            user_role: UserRole::User,
            pswd: &pass
        };
        
        match diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result::<User>(connection)
            {
                Ok(fetched_user) => {

                    let user_login_data = UserData{
                        id: fetched_user.id,
                        region: fetched_user.region.clone(),
                        username: fetched_user.username.clone(),
                        bio: fetched_user.bio.clone(),
                        avatar: fetched_user.avatar.clone(),
                        banner: fetched_user.banner.clone(),
                        wallet_background: fetched_user.wallet_background.clone(),
                        activity_code: fetched_user.activity_code.clone(),
                        twitter_username: fetched_user.twitter_username.clone(),
                        facebook_username: fetched_user.facebook_username.clone(),
                        discord_username: fetched_user.discord_username.clone(),
                        identifier: fetched_user.identifier.clone(),
                        user_role: {
                            match fetched_user.user_role.clone(){
                                UserRole::Admin => "Admin".to_string(),
                                UserRole::User => "User".to_string(),
                                _ => "Dev".to_string(),
                            }
                        },
                        token_time: fetched_user.token_time,
                        balance: fetched_user.balance,
                        last_login: { 
                            if fetched_user.last_login.is_some(){
                                Some(fetched_user.last_login.unwrap().to_string())
                            } else{
                                Some("".to_string())
                            }
                        },
                        created_at: fetched_user.created_at.to_string(),
                        updated_at: fetched_user.updated_at.to_string(),
                        mail: fetched_user.clone().mail,
                        google_id: fetched_user.clone().google_id,
                        microsoft_id: fetched_user.clone().microsoft_id,
                        is_mail_verified: fetched_user.is_mail_verified,
                        is_phone_verified: fetched_user.is_phone_verified,
                        phone_number: fetched_user.clone().phone_number,
                        paypal_id: fetched_user.clone().paypal_id,
                        account_number: fetched_user.clone().account_number,
                        device_id: fetched_user.clone().device_id,
                        social_id: fetched_user.clone().social_id,
                        cid: fetched_user.clone().cid,
                        screen_cid: fetched_user.clone().screen_cid,
                        snowflake_id: fetched_user.snowflake_id,
                        stars: fetched_user.stars,
                        extra: fetched_user.clone().extra,
                    };

                    /* generate cookie 🍪 from token time and jwt */
                    /* since generate_cookie_and_jwt() takes the ownership of the user instance we must clone it then call this */
                    /* generate_cookie_and_jwt() returns a Cookie instance with a 'static lifetime which allows us to return it from here*/
                    let Some(cookie_info) = fetched_user.clone().generate_cookie_and_jwt() else{
                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: CANT_GENERATE_COOKIE,
                            status: 500,
                            is_error: true,
                        };
                        return Err(
                            Ok(HttpResponse::InternalServerError().json(resp))
                        );
                    };
                    
                    let cookie = cookie_info.0;
                    let cookie_token_time = cookie_info.1;

                    /* update the login token time */
                    let now = chrono::Local::now().naive_local();
                    let updated_user = diesel::update(users.find(fetched_user.id))
                        .set((last_login.eq(now), token_time.eq(cookie_token_time)))
                        .execute(connection)
                        .unwrap();
                    
                    Ok((user_login_data, cookie))

                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::insert_by_identifier_password");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }
    
    }

    pub async fn insert_new_google_user(user: GoogleUserResult, 
            connection: &mut DbPoolConnection,
            redis_client: &RedisClient
        ) -> Result<User, PanelHttpResponse>{

        let new_user = NewGooleUser{
            username: user.name.as_str(),
            identifier: &user.email, // we should insert this into the identifier since mail field must be verified later by the user himself
            user_role: UserRole::User,
            avatar: Some(&user.picture),
            activity_code: "",
            pswd: "",
        };

        match diesel::insert_into(users::table)
            .values(&new_user)
            .execute(connection)
            {
                Ok(affected_row) => {
                    
                    if affected_row >= 1{

                        let user = User::find_by_identifier(&user.email, connection).await.unwrap();
                        Ok(user)
                    } else{
                        Ok(User::default())
                    }
                
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::insert_new_google_user");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn update_user_with_google_info(user: GoogleUserResult, user_id: i32, 
        redis_actor: Addr<RedisActor>, connection: &mut DbPoolConnection,
        redis_client: &RedisClient
    ) -> Result<User, PanelHttpResponse>{

    
        let get_user = User::find_by_id(user_id, connection).await;
        let Ok(user_info) = get_user else{
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };

        // we must check these on every google login update since
        // user might have not updated the username field yet and 
        // updated the avatar with his own picture
        let updated_avatar = if user_info.avatar.is_some() && !user_info.clone().avatar.unwrap().is_empty(){
            user_info.avatar.unwrap()
        } else{
            user.clone().picture
        };

        let updated_username = if !user_info.username.is_empty(){
            user_info.username
        } else{
            user.clone().name
        };

        match diesel::update(users.find(user_id))
            .set(
                    (   // when we're trying to update a user we're pretty sure 
                        // that this user is already loggedin with google and 
                        // his mail is verified thus we update both identifier 
                        // and mail fields 
                        username.eq(updated_username),
                        identifier.eq(&user.email),
                        mail.eq(&user.email),
                        avatar.eq(updated_avatar)

                    )
            )
            .returning(User::as_returning())
            .get_result(connection)
        {
            
            Ok(updated_user) => {
                
                /* ----------------------------------------------- */
                /* --------- publish updated user to redis channel */
                /* ----------------------------------------------- */
                /* 
                    once the user updates his info we'll publish new updated user to redis channel and in
                    other parts we start to subscribe to the new updated user topic then once we receive 
                    the new user we'll start updating user fans and user nfts 
                */
                let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;
                
                Ok(updated_user)
            
            },
            Err(e) => {

                let resp_err = &e.to_string();


                /* custom error handler */
                use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();  
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_user_with_google_info");
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: resp_err,
                    status: 500,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::InternalServerError().json(resp))
                );

            }
        }

    }

    pub async fn insert_new_user(user: NewUserInfoRequest, 
        connection: &mut DbPoolConnection,
        redis_client: &RedisClient
        ) -> Result<usize, PanelHttpResponse>{

        let hash_pswd = User::hash_pswd(user.password.as_str()).unwrap();
        let u_name = user.username.as_str();
        let identifier_login = user.identifier.as_str();
        let uname = if u_name == ""{
            chrono::Local::now().timestamp_nanos_opt().unwrap().to_string()
        } else{
            u_name.to_string()
        };

        let user = NewUser{
            username: &uname,
            activity_code: "",
            identifier: {
                if identifier_login == ""{
                    &uname
                } else{
                    identifier_login
                }
            },
            user_role: match user.role.as_str(){
                "Admin" => UserRole::Admin,
                "Dev" => UserRole::Dev,
                _ => UserRole::User
            },
            pswd: hash_pswd.as_str()
        };

        match diesel::insert_into(users::table)
            .values(&user)
            .execute(connection)
            {
                Ok(affected_row) => Ok(affected_row),
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::insert_new_user");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn update_bio(
        bio_owner_id: i32, 
        new_bio: &str, 
        redis_client: redis::Client,
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


        let Ok(user) = User::find_by_id(bio_owner_id, connection).await else{
            let resp = Response{
                data: Some(bio_owner_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };


        match diesel::update(users.find(user.id))
            .set(bio.eq(new_bio.to_lowercase()))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {
                    
                    /* ----------------------------------------------- */
                    /* --------- publish updated user to redis channel */
                    /* ----------------------------------------------- */
                    /* 
                        once the user updates his info we'll publish new updated user to redis channel and in
                        other parts we start to subscribe to the new updated user topic then once we receive 
                        the new user we'll start updating user fans and user nfts 
                    */
                    let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                    events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;

                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            region: updated_user.region.clone(),
                            username: updated_user.clone().username, 
                            bio: updated_user.bio.clone(),
                            avatar: updated_user.avatar.clone(),
                            banner: updated_user.banner.clone(),
                            wallet_background: updated_user.wallet_background.clone(),
                            activity_code: updated_user.clone().activity_code, 
                            twitter_username: updated_user.clone().twitter_username, 
                            facebook_username: updated_user.clone().facebook_username, 
                            discord_username: updated_user.clone().discord_username, 
                            identifier: updated_user.clone().identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            balance: updated_user.balance,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            mail: updated_user.clone().mail,
                            google_id: updated_user.clone().google_id,
                            microsoft_id: updated_user.clone().microsoft_id,
                            is_mail_verified: updated_user.is_mail_verified,
                            is_phone_verified: updated_user.is_phone_verified,
                            phone_number: updated_user.clone().phone_number,
                            paypal_id: updated_user.clone().paypal_id,
                            account_number: updated_user.clone().account_number,
                            device_id: updated_user.clone().device_id,
                            social_id: updated_user.clone().social_id,
                            cid: updated_user.clone().cid,
                            screen_cid: updated_user.clone().screen_cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars,
                            extra: updated_user.clone().extra,
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_bio");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn update_extra(
        extra_owner_id: i32, 
        new_extra: serde_json::Value, 
        redis_client: redis::Client,
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


        let Ok(user) = User::find_by_id(extra_owner_id, connection).await else{
            let resp = Response{
                data: Some(extra_owner_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };


        match diesel::update(users.find(user.id))
            .set(extra.eq(new_extra))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {
                    
                    /* ----------------------------------------------- */
                    /* --------- publish updated user to redis channel */
                    /* ----------------------------------------------- */
                    /* 
                        once the user updates his info we'll publish new updated user to redis channel and in
                        other parts we start to subscribe to the new updated user topic then once we receive 
                        the new user we'll start updating user fans and user nfts 
                    */
                    let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                    events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;

                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            region: updated_user.region.clone(),
                            username: updated_user.clone().username, 
                            bio: updated_user.bio.clone(),
                            avatar: updated_user.avatar.clone(),
                            banner: updated_user.banner.clone(),
                            wallet_background: updated_user.wallet_background.clone(),
                            activity_code: updated_user.clone().activity_code, 
                            twitter_username: updated_user.clone().twitter_username, 
                            facebook_username: updated_user.clone().facebook_username, 
                            discord_username: updated_user.clone().discord_username, 
                            identifier: updated_user.clone().identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            balance: updated_user.balance,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            mail: updated_user.clone().mail,
                            google_id: updated_user.clone().google_id,
                            microsoft_id: updated_user.clone().microsoft_id,
                            is_mail_verified: updated_user.is_mail_verified,
                            is_phone_verified: updated_user.is_phone_verified,
                            phone_number: updated_user.clone().phone_number,
                            paypal_id: updated_user.clone().paypal_id,
                            account_number: updated_user.clone().account_number,
                            device_id: updated_user.clone().device_id,
                            social_id: updated_user.clone().social_id,
                            cid: updated_user.clone().cid,
                            screen_cid: updated_user.clone().screen_cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars,
                            extra: updated_user.clone().extra,
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_extra");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn update_wallet_back(
        wallet_owner_id: i32, 
        mut img: Multipart, 
        redis_client: redis::Client,
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{
        
            
        let Ok(user) = User::find_by_id(wallet_owner_id, connection).await else{
            let resp = Response{
                data: Some(wallet_owner_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        let img = std::sync::Arc::new(tokio::sync::Mutex::new(img));
        let get_wallet_img_path = multipartreq::store_file(
            WALLET_BACK_UPLOAD_PATH, &format!("{}", wallet_owner_id), 
            "walletback", 
            img).await;
        let Ok(wallet_img_path) = get_wallet_img_path else{

            let err_res = get_wallet_img_path.unwrap_err();
            return Err(err_res);
        };

        /* update the avatar field in db */
        match diesel::update(users.find(user.id))
            .set(wallet_background.eq(wallet_img_path))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {
                    
                    /* ----------------------------------------------- */
                    /* --------- publish updated user to redis channel */
                    /* ----------------------------------------------- */
                    /* 
                        once the user updates his info we'll publish new updated user to redis channel and in
                        other parts we start to subscribe to the new updated user topic then once we receive 
                        the new user we'll start updating user fans and user nfts 
                    */
                    
                    let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                    events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;

                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            region: updated_user.region.clone(),
                            username: updated_user.clone().username, 
                            bio: updated_user.bio.clone(),
                            avatar: updated_user.avatar.clone(),
                            banner: updated_user.banner.clone(),
                            wallet_background: updated_user.wallet_background.clone(),
                            activity_code: updated_user.clone().activity_code, 
                            twitter_username: updated_user.clone().twitter_username, 
                            facebook_username: updated_user.clone().facebook_username, 
                            discord_username: updated_user.clone().discord_username, 
                            identifier: updated_user.clone().identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            balance: updated_user.balance,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            mail: updated_user.clone().mail,
                            google_id: updated_user.clone().google_id,
                            microsoft_id: updated_user.clone().microsoft_id,
                            is_mail_verified: updated_user.is_mail_verified,
                            is_phone_verified: updated_user.is_phone_verified,
                            phone_number: updated_user.clone().phone_number,
                            paypal_id: updated_user.clone().paypal_id,
                            account_number: updated_user.clone().account_number,
                            device_id: updated_user.clone().device_id,
                            social_id: updated_user.clone().social_id,
                            cid: updated_user.clone().cid,
                            screen_cid: updated_user.clone().screen_cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars,
                            extra: updated_user.clone().extra,
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_wallet_back");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }
    
    }

    pub async fn update_avatar(
        avatar_owner_id: i32, 
        mut img: Multipart, 
        redis_client: redis::Client,
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


        let Ok(user) = User::find_by_id(avatar_owner_id, connection).await else{
            let resp = Response{
                data: Some(avatar_owner_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        let img = std::sync::Arc::new(tokio::sync::Mutex::new(img));
        let get_avatar_img_path = multipartreq::store_file(
            AVATAR_UPLOAD_PATH, &format!("{}", avatar_owner_id), 
            "avatar", 
            img).await;
        let Ok(avatar_img_path) = get_avatar_img_path else{

            let err_res = get_avatar_img_path.unwrap_err();
            return Err(err_res);
        };

        /* update the avatar field in db */
        match diesel::update(users.find(user.id))
            .set(avatar.eq(avatar_img_path))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {

                    /* ----------------------------------------------- */
                    /* --------- publish updated user to redis channel */
                    /* ----------------------------------------------- */
                    /* 
                        once the user updates his info we'll publish new updated user to redis channel and in
                        other parts we start to subscribe to the new updated user topic then once we receive 
                        the new user we'll start updating user fans and user nfts 
                    */
                    
                    let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                    events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;

                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            region: updated_user.region.clone(),
                            username: updated_user.clone().username, 
                            bio: updated_user.bio.clone(),
                            avatar: updated_user.avatar.clone(),
                            banner: updated_user.banner.clone(),
                            wallet_background: updated_user.wallet_background.clone(),
                            activity_code: updated_user.clone().activity_code, 
                            twitter_username: updated_user.clone().twitter_username, 
                            facebook_username: updated_user.clone().facebook_username, 
                            discord_username: updated_user.clone().discord_username, 
                            identifier: updated_user.clone().identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            balance: updated_user.balance,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            mail: updated_user.clone().mail,
                            google_id: updated_user.clone().google_id,
                            microsoft_id: updated_user.clone().microsoft_id,
                            is_mail_verified: updated_user.is_mail_verified,
                            is_phone_verified: updated_user.is_phone_verified,
                            phone_number: updated_user.clone().phone_number,
                            paypal_id: updated_user.clone().paypal_id,
                            account_number: updated_user.clone().account_number,
                            device_id: updated_user.clone().device_id,
                            social_id: updated_user.clone().social_id,
                            cid: updated_user.clone().cid,
                            screen_cid: updated_user.clone().screen_cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars,
                            extra: updated_user.clone().extra,
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_avatar");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn update_banner(
        banner_owner_id: i32, 
        mut img: Multipart, 
        redis_client: redis::Client,
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


        let Ok(user) = User::find_by_id(banner_owner_id, connection).await else{
            let resp = Response{
                data: Some(banner_owner_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        let img = std::sync::Arc::new(tokio::sync::Mutex::new(img));
        let get_banner_img_path = multipartreq::store_file(
            BANNER_UPLOAD_PATH, &format!("{}", banner_owner_id), 
            "banner", 
            img).await;
        let Ok(banner_img_path) = get_banner_img_path else{

            let err_res = get_banner_img_path.unwrap_err();
            return Err(err_res);
        };

        /* update the avatar field in db */
        match diesel::update(users.find(user.id))
            .set(banner.eq(banner_img_path))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {

                    /* ----------------------------------------------- */
                    /* --------- publish updated user to redis channel */
                    /* ----------------------------------------------- */
                    /* 
                        once the user updates his info we'll publish new updated user to redis channel and in
                        other parts we start to subscribe to the new updated user topic then once we receive 
                        the new user we'll start updating user fans and user nfts 
                    */
                    
                    let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                    events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;

                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            region: updated_user.region.clone(),
                            username: updated_user.clone().username, 
                            bio: updated_user.bio.clone(),
                            avatar: updated_user.avatar.clone(),
                            banner: updated_user.banner.clone(),
                            wallet_background: updated_user.wallet_background.clone(),
                            activity_code: updated_user.clone().activity_code, 
                            twitter_username: updated_user.clone().twitter_username, 
                            facebook_username: updated_user.clone().facebook_username, 
                            discord_username: updated_user.clone().discord_username, 
                            identifier: updated_user.clone().identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            balance: updated_user.balance,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            mail: updated_user.clone().mail,
                            google_id: updated_user.clone().google_id,
                            microsoft_id: updated_user.clone().microsoft_id,
                            is_mail_verified: updated_user.is_mail_verified,
                            is_phone_verified: updated_user.is_phone_verified,
                            phone_number: updated_user.clone().phone_number,
                            paypal_id: updated_user.clone().paypal_id,
                            account_number: updated_user.clone().account_number,
                            device_id: updated_user.clone().device_id,
                            social_id: updated_user.clone().social_id,
                            cid: updated_user.clone().cid,
                            screen_cid: updated_user.clone().screen_cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars,
                            extra: updated_user.clone().extra,
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_banner");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn edit_by_admin(new_user: EditUserByAdminRequest, redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{

        /* fetch user info based on the data inside jwt */ 
        let single_user = users
            .filter(users::id.eq(new_user.user_id.to_owned()))
            .first::<User>(connection);

        let Ok(user) = single_user else{
            let resp = Response{
                data: Some(new_user.user_id.to_owned()),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };
        
        /* if the passed in password was some then we must updated the password */
        let password = if let Some(password) = &new_user.password{ // borrowing the user to prevent from moving

            /* we can pass &str to the method by borrowing the String since String will be coerced into &str at compile time */
            User::hash_pswd(password).unwrap()

        } else{
            
            /* if the passed in password was none then we must use the old one */
            user.pswd

        };


        let _username = if new_user.username != "".to_string(){
            &new_user.username
        } else{
            let resp = Response{
                data: Some(new_user.user_id.to_owned()),
                message: USERNAME_CANT_BE_EMPTY,
                status: 406,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        let _identifier = if new_user.identifier != "".to_string(){
            &new_user.identifier
        } else{
            let resp = Response{
                data: Some(new_user.user_id.to_owned()),
                message: WALLET_CANT_BE_EMPTY,
                status: 406,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };
        
        match diesel::update(users.find(new_user.user_id.to_owned()))
            .set(EditUserByAdmin{
                user_role: {
                    let role = new_user.role.as_str(); 
                    match role{
                        "User" => UserRole::User,
                        "Admin" => UserRole::Admin,
                        _ => UserRole::Dev
                    }
                },
                /* 
                    pswd, username and identifier is of type &str thus by borrowing these 
                    field from new_user instance we can convert them into &str 
                */
                pswd: &password,
                username: &_username,
                identifier: &_identifier
            })
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {

                    /* ----------------------------------------------- */
                    /* --------- publish updated user to redis channel */
                    /* ----------------------------------------------- */
                    /* 
                        once the user updates his info we'll publish new updated user to redis channel and in
                        other parts we start to subscribe to the new updated user topic then once we receive 
                        the new user we'll start updating user fans and user nfts 
                    */
                    
                    let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                    events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;
                    
                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            region: updated_user.region.clone(),
                            username: updated_user.clone().username, 
                            bio: updated_user.bio.clone(),
                            avatar: updated_user.avatar.clone(),
                            banner: updated_user.banner.clone(),
                            wallet_background: updated_user.wallet_background.clone(),
                            activity_code: updated_user.clone().activity_code, 
                            twitter_username: updated_user.clone().twitter_username, 
                            facebook_username: updated_user.clone().facebook_username, 
                            discord_username: updated_user.clone().discord_username, 
                            identifier: updated_user.clone().identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            balance: updated_user.balance,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            mail: updated_user.clone().mail,
                            google_id: updated_user.clone().google_id,
                            microsoft_id: updated_user.clone().microsoft_id,
                            is_mail_verified: updated_user.is_mail_verified,
                            is_phone_verified: updated_user.is_phone_verified,
                            phone_number: updated_user.clone().phone_number,
                            paypal_id: updated_user.clone().paypal_id,
                            account_number: updated_user.clone().account_number,
                            device_id: updated_user.clone().device_id,
                            social_id: updated_user.clone().social_id,
                            cid: updated_user.clone().cid,
                            screen_cid: updated_user.clone().screen_cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars,
                            extra: updated_user.clone().extra,
                        }
                    )
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::edit_by_admin");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn delete_by_admin(doer_id: i32, connection: &mut DbPoolConnection) -> Result<usize, PanelHttpResponse>{

        /* we must first delete from users_tasks */
        
        match UserTask::delete_by_doer(doer_id, connection).await {
            Ok(users_tasks_rows_deleted) => {

                match diesel::delete(users.filter(users::id.eq(doer_id.to_owned())))
                    .execute(connection)
                    {
                        Ok(mut num_deleted) => {
                            
                            /* also delete any users record if there was any */

                            num_deleted += users_tasks_rows_deleted;

                            Ok(num_deleted)
                        
                        },
                        Err(e) => {

                            let resp_err = &e.to_string();


                            /* custom error handler */
                            use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                             
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();  
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::delete_by_admin");
                            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                            let resp = Response::<&[u8]>{
                                data: Some(&[]),
                                message: resp_err,
                                status: 500,
                                is_error: true,
                            };
                            return Err(
                                Ok(HttpResponse::InternalServerError().json(resp))
                            );

                        }
                    }

            },
            Err(e) => {
                
                return Err(e);
            }
        }

    
    }

    pub async fn get_all(connection: &mut DbPoolConnection, limit: web::Query<Limit>) -> Result<Vec<UserData>, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }
        
        match users
            .order(created_at.desc())
            .offset(from)
            .limit((to - from) + 1)
            .load::<User>(connection)
        {
            Ok(all_users) => {
                Ok(
                    all_users
                        .into_iter()
                        .map(|u| UserData { 
                            id: u.id, 
                            region: u.region.clone(),
                            username: u.clone().username, 
                            bio: u.bio.clone(),
                            avatar: u.avatar.clone(),
                            banner: u.banner.clone(),
                            wallet_background: u.wallet_background.clone(),
                            activity_code: u.clone().activity_code, 
                            twitter_username: u.clone().twitter_username, 
                            facebook_username: u.clone().facebook_username, 
                            discord_username: u.clone().discord_username, 
                            identifier: u.clone().identifier, 
                            user_role: {
                                match u.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: u.token_time,
                            balance: u.balance,
                            last_login: { 
                                if u.last_login.is_some(){
                                    Some(u.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: u.created_at.to_string(),
                            updated_at: u.updated_at.to_string(),
                            mail: u.clone().mail,
                            google_id: u.clone().google_id,
                            microsoft_id: u.clone().microsoft_id,
                            is_mail_verified: u.clone().is_mail_verified,
                            is_phone_verified: u.clone().is_phone_verified,
                            phone_number: u.clone().phone_number,
                            paypal_id: u.clone().paypal_id,
                            account_number: u.clone().account_number,
                            device_id: u.clone().device_id,
                            social_id: u.clone().social_id,
                            cid: u.clone().cid,
                            screen_cid: u.clone().screen_cid,
                            snowflake_id: u.snowflake_id,
                            stars: u.stars,
                            extra: u.clone().extra,
                        })
                        .collect::<Vec<UserData>>()
                )
            },
            Err(e) => {

                let resp_err = &e.to_string();


                /* custom error handler */
                use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                 
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();  
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::get_all");
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: resp_err,
                    status: 500,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::InternalServerError().json(resp))
                );

            }
        }

    }

    pub async fn get_all_balance_greater_than_100(connection: &mut DbPoolConnection, limit: web::Query<Limit>) -> Result<Vec<UserData>, PanelHttpResponse>{

        let from = limit.from.unwrap_or(0);
        let to = limit.to.unwrap_or(10);

        if to < from {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INVALID_QUERY_LIMIT,
                status: 406,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            )
        }
        
        match users
            .order(created_at.desc())
            .order(balance.desc())
            .offset(from)
            .limit((to - from) + 1)
            .filter(balance.ge(100))
            .load::<User>(connection)
        {
            Ok(all_users) => {
                Ok(
                    all_users
                        .into_iter()
                        .map(|u| UserData { 
                            id: u.id, 
                            region: u.region.clone(),
                            username: u.clone().username, 
                            bio: u.bio.clone(),
                            avatar: u.avatar.clone(),
                            banner: u.banner.clone(),
                            wallet_background: u.wallet_background.clone(),
                            activity_code: u.clone().activity_code, 
                            twitter_username: u.clone().twitter_username, 
                            facebook_username: u.clone().facebook_username, 
                            discord_username: u.clone().discord_username, 
                            identifier: u.clone().identifier, 
                            user_role: {
                                match u.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: u.token_time,
                            balance: u.balance,
                            last_login: { 
                                if u.last_login.is_some(){
                                    Some(u.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: u.created_at.to_string(),
                            updated_at: u.updated_at.to_string(),
                            mail: u.clone().mail,
                            google_id: u.clone().google_id,
                            microsoft_id: u.clone().microsoft_id,
                            is_mail_verified: u.clone().is_mail_verified,
                            is_phone_verified: u.clone().is_phone_verified,
                            phone_number: u.clone().phone_number,
                            paypal_id: u.clone().paypal_id,
                            account_number: u.clone().account_number,
                            device_id: u.clone().device_id,
                            social_id: u.clone().social_id,
                            cid: u.clone().cid,
                            screen_cid: u.clone().screen_cid,
                            snowflake_id: u.snowflake_id,
                            stars: u.stars,
                            extra: u.clone().extra,
                        })
                        .collect::<Vec<UserData>>()
                )
            },
            Err(e) => {

                let resp_err = &e.to_string();


                /* custom error handler */
                use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                 
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();  
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::get_all_balance_greater_than_100");
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: resp_err,
                    status: 500,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::InternalServerError().json(resp))
                );

            }
        }

    }

    pub async fn get_all_without_limit(connection: &mut DbPoolConnection) -> Result<Vec<UserData>, PanelHttpResponse>{
        
        match users
            .load::<User>(connection)
        {
            Ok(all_users) => {
                Ok(
                    all_users
                        .into_iter()
                        .map(|u| UserData { 
                            id: u.id, 
                            region: u.region.clone(),
                            username: u.clone().username, 
                            bio: u.bio.clone(),
                            avatar: u.avatar.clone(),
                            banner: u.banner.clone(),
                            wallet_background: u.wallet_background.clone(),
                            activity_code: u.clone().activity_code, 
                            twitter_username: u.clone().twitter_username, 
                            facebook_username: u.clone().facebook_username, 
                            discord_username: u.clone().discord_username, 
                            identifier: u.clone().identifier, 
                            user_role: {
                                match u.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: u.token_time,
                            balance: u.balance,
                            last_login: { 
                                if u.last_login.is_some(){
                                    Some(u.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: u.created_at.to_string(),
                            updated_at: u.updated_at.to_string(),
                            mail: u.clone().mail,
                            google_id: u.clone().google_id,
                            microsoft_id: u.clone().microsoft_id,
                            is_mail_verified: u.clone().is_mail_verified,
                            is_phone_verified: u.clone().is_phone_verified,
                            phone_number: u.clone().phone_number,
                            paypal_id: u.clone().paypal_id,
                            account_number: u.clone().account_number,
                            device_id: u.clone().device_id,
                            social_id: u.clone().social_id,
                            cid: u.clone().cid,
                            screen_cid: u.clone().screen_cid,
                            snowflake_id: u.snowflake_id,
                            stars: u.stars,
                            extra: u.clone().extra,
                        })
                        .collect::<Vec<UserData>>()
                )
            },
            Err(e) => {

                let resp_err = &e.to_string();


                /* custom error handler */
                use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                 
                let error_content = &e.to_string();
                let error_content = error_content.as_bytes().to_vec();  
                let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::get_all_without_limit");
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: resp_err,
                    status: 500,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::InternalServerError().json(resp))
                );

            }
        }

    }

    pub async fn logout(who: i32, _token_time: i64, redis_client: redis::Client, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<(), PanelHttpResponse>{

        match diesel::update(users.find(who))
            .set(token_time.eq(0))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {

                    // also remove jwt from the users_logins table
                    let get_user_login_infos = UserLogin::find_by_user_id(who, connection).await;
                    let Ok(user_login_infos) = get_user_login_infos else{
                        let err_resp = get_user_login_infos.unwrap_err();
                        return Err(err_resp);
                    };

                    // also make the jwt field empty in db
                    let toke_time_hash = format!("{}", _token_time);
                    let mut hasher = Sha256::new();
                    hasher.update(toke_time_hash.as_str());
                    let time_hash = hasher.finalize();
                    let token_time_hash_hex = hex::encode(time_hash.to_vec());
                    for login_info in user_login_infos{
                        if login_info.jwt == token_time_hash_hex{
                            let get_updated_user_login = UserLogin::remove_jwt(login_info.id, connection).await;
                            let Ok(updated_user_login) = get_updated_user_login else{
                                let err_resp = get_updated_user_login.unwrap_err();
                                return Err(err_resp);
                            };
                        }
                    }
                    
                    /* ----------------------------------------------- */
                    /* --------- publish updated user to redis channel */
                    /* ----------------------------------------------- */
                    /* 
                        once the user updates his info we'll publish new updated user to redis channel and in
                        other parts we start to subscribe to the new updated user topic then once we receive 
                        the new user we'll start updating user fans and user nfts 
                    */
                    
                    let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                    events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;
                    
                    Ok(())
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::logout");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn update_balance(owner_id: i32, tx_type: &str, treasury_type: &str,
        new_balance: i64, redis_client: RedisClient, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) 
        -> Result<UserData, PanelHttpResponse>{

        let mut redis_conn = redis_client.get_async_connection().await.unwrap();

        let Ok(user) = User::find_by_id(owner_id, connection).await else{
            let resp = Response{
                data: Some(owner_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        if new_balance < 0 {
            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: INSUFFICIENT_FUNDS,
                status: 406,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

        // ========================================================================
        // ======================== system treasury calculations ==================
        // ========================================================================
        use crate::models::sys_treasury::SysTreasuryRequest;
        use crate::schema::sys_treasury::dsl::sys_treasury;

        // insert into sys_treasury 1 % of the tx 
        // new balance is the updated new balance considered by 1 %
        let is_key_there: bool = redis_conn.exists("sys_treasury").await.unwrap();
        
        let (sys_treasury_, pure_balance) = if tx_type.starts_with("Airdrop") {
            let mut splitted_tx_type = tx_type.split("|");
            let invar = splitted_tx_type.next().unwrap();
            let unvar = splitted_tx_type.next().unwrap();
            (SysTreasuryRequest{
                airdrop: unvar.parse::<i64>().unwrap(),
                debit: unvar.parse::<i64>().unwrap(),
                paid_to: owner_id,
                current_networth: if is_key_there{
                    let get_sys_trs_instance: String = redis_conn.get("sys_treasury").await.unwrap();
                    let mut sys_trs_instance = serde_json::from_str::<SysTreasuryRequest>(&get_sys_trs_instance).unwrap();
                    sys_trs_instance.current_networth -= unvar.parse::<i64>().unwrap();
                    sys_trs_instance.current_networth
                } else{
                    0 - unvar.parse::<i64>().unwrap()
                }
            }, new_balance)
        } else if tx_type.starts_with("Error"){
            
            (
                SysTreasuryRequest{
                    airdrop: 0,
                    debit: 0,
                    paid_to: owner_id,
                    current_networth: if is_key_there{
                        let get_sys_trs_instance: String = redis_conn.get("sys_treasury").await.unwrap();
                        let mut sys_trs_instance = serde_json::from_str::<SysTreasuryRequest>(&get_sys_trs_instance).unwrap();
                        sys_trs_instance.current_networth
                    } else{
                        0
                    }
                },
                new_balance
            )

        } else{
            let nb = new_balance as f64;
            let percent = percentage::Percentage::from_decimal(1.0);
            let profit = percent.apply_to(nb);
            let pure_balance = (nb - profit.round()) as i64;
            
            let sys_treasury_ = SysTreasuryRequest{
                airdrop: 0,
                debit: 0,
                paid_to: owner_id,
                current_networth: if is_key_there{
                    let get_sys_trs_instance: String = redis_conn.get("sys_treasury").await.unwrap();
                    let mut sys_trs_instance = serde_json::from_str::<SysTreasuryRequest>(&get_sys_trs_instance).unwrap();
                    sys_trs_instance.current_networth += profit.round() as i64;
                    sys_trs_instance.current_networth
                } else{
                    profit.round() as i64
                }
            };
            (sys_treasury_, pure_balance)
        };


        let new_sys_instance_stringified = serde_json::to_string(&sys_treasury_).unwrap();
        let _: () = redis_conn.set("sys_treasury", &new_sys_instance_stringified).await.unwrap();

        diesel::insert_into(sys_treasury)
            .values(&sys_treasury_)
            .returning(SysTreasury::as_returning())
            .get_result::<SysTreasury>(connection);
        // ========================================================================
        // ========================================================================
        // ========================================================================


        // ======================================================================
        // ======================== user treasury calculations ==================
        // ======================================================================
        use crate::models::user_treasury::{UserTreasuryRequest, UserTreasury};
        use crate::schema::user_treasury::dsl::user_treasury;
        let user_treasury_ = UserTreasuryRequest{
            user_id: owner_id,
            done_at: chrono::Local::now().timestamp(),
            amount: pure_balance,
            tx_type: tx_type.to_string(),
            treasury_type: treasury_type.to_string(),
        };
        diesel::insert_into(user_treasury)
            .values(&user_treasury_)
            .returning(UserTreasury::as_returning())
            .get_result::<UserTreasury>(connection);

        // ======================================================================
        // ======================================================================
        // ======================================================================

        match diesel::update(users.find(user.id))
            .set(balance.eq(pure_balance))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {

                    /* ----------------------------------------------- */
                    /* --------- publish updated user to redis channel */
                    /* ----------------------------------------------- */
                    /* 
                        once the user updates his info we'll publish new updated user to redis channel and in
                        other parts we start to subscribe to the new updated user topic then once we receive 
                        the new user we'll start updating user fans and user nfts 
                    */
                    
                    let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                    events::publishers::user::emit(redis_actor.clone(), "on_user_update", &json_stringified_updated_user).await;

                    /** -------------------------------------------------------------------- */
                    /** ----------------- publish new event data to `on_user_action` channel */
                    /** -------------------------------------------------------------------- */
                    // if the actioner is the user himself we'll notify user with something like:
                    // u've just done that action!
                    let actioner_wallet_info = UserWalletInfoResponse{
                        username: updated_user.clone().username,
                        avatar: updated_user.clone().avatar,
                        bio: updated_user.clone().bio,
                        banner: updated_user.clone().banner,
                        mail: updated_user.clone().mail,
                        screen_cid: updated_user.clone().screen_cid,
                        extra: updated_user.clone().extra,
                        stars: updated_user.clone().stars,
                        created_at: updated_user.clone().created_at.to_string(),
                    };
                    let user_wallet_info = UserWalletInfoResponse{
                        username: updated_user.clone().username,
                        avatar: updated_user.clone().avatar,
                        bio: updated_user.clone().bio,
                        banner: updated_user.clone().banner,
                        mail: updated_user.clone().mail,
                        screen_cid: updated_user.clone().screen_cid,
                        extra: updated_user.clone().extra,
                        stars: updated_user.clone().stars,
                        created_at: updated_user.clone().created_at.to_string(),
                    };
                    let user_notif_info = SingleUserNotif{
                        wallet_info: user_wallet_info,
                        notif: NotifData{
                            actioner_wallet_info,
                            fired_at: Some(chrono::Local::now().timestamp()),
                            action_type: ActionType::UpdateBalance,
                            action_data: serde_json::to_value(updated_user.clone()).unwrap()
                        }
                    };
                    let stringified_user_notif_info = serde_json::to_string_pretty(&user_notif_info).unwrap();
                    events::publishers::action::emit(redis_actor.clone(), "on_user_action", &stringified_user_notif_info).await;


                    // insert into users_balance table, tracking the income and outcome of
                    // each user let us to debug quickly and prevent frauds
                    UserToken::insert(
                        NewUserTokenRequest{
                            user_id: user.id,
                            current_balance: updated_user.balance.unwrap_or(0),
                            last_balance: user.balance.unwrap_or(0),
                        },
                        connection
                    ).await;

                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            region: updated_user.region.clone(),
                            username: updated_user.clone().username, 
                            bio: updated_user.bio.clone(),
                            avatar: updated_user.avatar.clone(),
                            banner: updated_user.banner.clone(),
                            wallet_background: updated_user.wallet_background.clone(),
                            activity_code: updated_user.clone().activity_code, 
                            twitter_username: updated_user.clone().twitter_username, 
                            facebook_username: updated_user.clone().facebook_username, 
                            discord_username: updated_user.clone().discord_username, 
                            identifier: updated_user.clone().identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            balance: updated_user.balance,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            mail: updated_user.clone().mail,
                            google_id: updated_user.clone().google_id,
                            microsoft_id: updated_user.clone().microsoft_id,
                            is_mail_verified: updated_user.is_mail_verified,
                            is_phone_verified: updated_user.is_phone_verified,
                            phone_number: updated_user.clone().phone_number,
                            paypal_id: updated_user.clone().paypal_id,
                            account_number: updated_user.clone().account_number,
                            device_id: updated_user.clone().device_id,
                            social_id: updated_user.clone().social_id,
                            cid: updated_user.clone().cid,
                            screen_cid: updated_user.clone().screen_cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars,
                            extra: updated_user.clone().extra,
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_balance");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

    pub async fn update_social_account(
        social_owner_id: i32, 
        account_name: &str, 
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


            let Ok(user) = User::find_by_id(social_owner_id, connection).await else{
                let resp = Response{
                    data: Some(social_owner_id),
                    message: USER_NOT_FOUND,
                    status: 404,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
            };

            let bot_endpoint = env::var("XBOT_ENDPOINT").expect("⚠️ no twitter bot endpoint key variable set");
            let new_twitter = Twitter::new(Some(bot_endpoint)).await;
            let Ok(bot) =  new_twitter else{
                return Err(new_twitter.unwrap_err());
            };

            let tusername = if account_name.is_empty(){
                "".to_string()
            } else{
                if account_name.contains("@"){
                    account_name.replace("@", "")
                } else{
                    account_name.to_string()
                }
            };

            let is_user_verified = bot.verify_user_with_xbot(&tusername, connection).await;
            let Ok(is_verified) = is_user_verified else{
                return Err(is_user_verified.unwrap_err());
            };

            if is_verified{
                
                match diesel::update(users.find(user.id))
                    .set(twitter_username.eq(tusername.to_lowercase()))
                    .returning(FetchUser::as_returning())
                    .get_result(connection)
                    {
                        Ok(updated_user) => {
                            Ok(
                                UserData { 
                                    id: updated_user.id, 
                                    region: updated_user.region.clone(),
                                    username: updated_user.clone().username, 
                                    bio: updated_user.bio.clone(),
                                    avatar: updated_user.avatar.clone(),
                                    banner: updated_user.banner.clone(),
                                    wallet_background: updated_user.wallet_background.clone(),
                                    activity_code: updated_user.clone().activity_code, 
                                    twitter_username: updated_user.clone().twitter_username, 
                                    facebook_username: updated_user.clone().facebook_username, 
                                    discord_username: updated_user.clone().discord_username, 
                                    identifier: updated_user.clone().identifier, 
                                    user_role: {
                                        match updated_user.user_role.clone(){
                                            UserRole::Admin => "Admin".to_string(),
                                            UserRole::User => "User".to_string(),
                                            _ => "Dev".to_string(),
                                        }
                                    },
                                    token_time: updated_user.token_time,
                                    balance: updated_user.balance,
                                    last_login: { 
                                        if updated_user.last_login.is_some(){
                                            Some(updated_user.last_login.unwrap().to_string())
                                        } else{
                                            Some("".to_string())
                                        }
                                    },
                                    created_at: updated_user.created_at.to_string(),
                                    updated_at: updated_user.updated_at.to_string(),
                                    mail: updated_user.clone().mail,
                                    google_id: updated_user.clone().google_id,
                                    microsoft_id: updated_user.clone().microsoft_id,
                                    is_mail_verified: updated_user.is_mail_verified,
                                    is_phone_verified: updated_user.is_phone_verified,
                                    phone_number: updated_user.clone().phone_number,
                                    paypal_id: updated_user.clone().paypal_id,
                                    account_number: updated_user.clone().account_number,
                                    device_id: updated_user.clone().device_id,
                                    social_id: updated_user.clone().social_id,
                                    cid: updated_user.clone().cid,
                                    screen_cid: updated_user.clone().screen_cid,
                                    snowflake_id: updated_user.snowflake_id,
                                    stars: updated_user.stars,
                                    extra: updated_user.clone().extra,
                                }
                            )
                        },
                        Err(e) => {
                            
                            let resp_err = &e.to_string();


                            /* custom error handler */
                            use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                             
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();  
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_social_account");
                            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                            let resp = Response::<&[u8]>{
                                data: Some(&[]),
                                message: resp_err,
                                status: 500,
                                is_error: true,
                            };
                            return Err(
                                Ok(HttpResponse::InternalServerError().json(resp))
                            );
    
                        }
                    }
            } else{

                let resp = Response{
                    data: Some(social_owner_id),
                    message: TWITTER_USER_IS_NOT_VALID,
                    status: 406,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                );

            }


    }

    pub async fn update_mail(
        mail_owner_id: i32, 
        new_mail: &str, 
        redis_client: RedisClient,
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


            let Ok(user) = User::find_by_id(mail_owner_id, connection).await else{
                let resp = Response{
                    data: Some(mail_owner_id),
                    message: USER_NOT_FOUND,
                    status: 404,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
            };


            match diesel::update(users.find(user.id))
                .set(mail.eq(new_mail.to_lowercase()))
                .returning(FetchUser::as_returning())
                .get_result(connection)
                {
                    Ok(updated_user) => {

                        /* ----------------------------------------------- */
                        /* --------- publish updated user to redis channel */
                        /* ----------------------------------------------- */
                        /* 
                            once the user updates his info we'll publish new updated user to redis channel and in
                            other parts we start to subscribe to the new updated user topic then once we receive 
                            the new user we'll start updating user fans and user nfts 
                        */
                        
                        let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                        events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;

                        Ok(
                            UserData { 
                                id: updated_user.id, 
                                region: updated_user.region.clone(),
                                username: updated_user.clone().username, 
                                bio: updated_user.bio.clone(),
                                avatar: updated_user.avatar.clone(),
                                banner: updated_user.banner.clone(),
                                wallet_background: updated_user.wallet_background.clone(),
                                activity_code: updated_user.clone().activity_code, 
                                twitter_username: updated_user.clone().twitter_username, 
                                facebook_username: updated_user.clone().facebook_username, 
                                discord_username: updated_user.clone().discord_username, 
                                identifier: updated_user.clone().identifier, 
                                user_role: {
                                    match updated_user.user_role.clone(){
                                        UserRole::Admin => "Admin".to_string(),
                                        UserRole::User => "User".to_string(),
                                        _ => "Dev".to_string(),
                                    }
                                },
                                token_time: updated_user.token_time,
                                balance: updated_user.balance,
                                last_login: { 
                                    if updated_user.last_login.is_some(){
                                        Some(updated_user.last_login.unwrap().to_string())
                                    } else{
                                        Some("".to_string())
                                    }
                                },
                                created_at: updated_user.created_at.to_string(),
                                updated_at: updated_user.updated_at.to_string(),
                                mail: updated_user.clone().mail,
                                google_id: updated_user.clone().google_id,
                                microsoft_id: updated_user.clone().microsoft_id,
                                is_mail_verified: updated_user.is_mail_verified,
                                is_phone_verified: updated_user.is_phone_verified,
                                phone_number: updated_user.clone().phone_number,
                                paypal_id: updated_user.clone().paypal_id,
                                account_number: updated_user.clone().account_number,
                                device_id: updated_user.clone().device_id,
                                social_id: updated_user.clone().social_id,
                                cid: updated_user.clone().cid,
                                screen_cid: updated_user.clone().screen_cid,
                                snowflake_id: updated_user.snowflake_id,
                                stars: updated_user.stars,
                                extra: updated_user.clone().extra,
                            }
                        )
                    },
                    Err(e) => {
                        
                        let resp_err = &e.to_string();


                        /* custom error handler */
                        use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_mail");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: resp_err,
                            status: 500,
                            is_error: true,
                        };
                        return Err(
                            Ok(HttpResponse::InternalServerError().json(resp))
                        );

                    }
                }



    }

    pub async fn update_phone(
        phone_owner_id: i32, 
        new_phone: &str, 
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


            let Ok(user) = User::find_by_id(phone_owner_id, connection).await else{
                let resp = Response{
                    data: Some(phone_owner_id),
                    message: USER_NOT_FOUND,
                    status: 404,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
            };

            let new_balance = if user.balance.is_none(){5} else{user.balance.unwrap() + 5};
            match diesel::update(users.find(user.id))
                .set(
                    (
                        phone_number.eq(new_phone),
                        balance.eq(new_balance)
                    )
                    
                )
                .returning(FetchUser::as_returning())
                .get_result(connection)
                {
                    Ok(updated_user) => {

                        /* ----------------------------------------------- */
                        /* --------- publish updated user to redis channel */
                        /* ----------------------------------------------- */
                        /* 
                            once the user updates his info we'll publish new updated user to redis channel and in
                            other parts we start to subscribe to the new updated user topic then once we receive 
                            the new user we'll start updating user fans and user nfts 
                        */
                        
                        let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                        events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;


                        Ok(
                            UserData { 
                                id: updated_user.id, 
                                region: updated_user.region.clone(),
                                username: updated_user.clone().username,
                                bio: updated_user.bio.clone(),
                                avatar: updated_user.avatar.clone(),
                                banner: updated_user.banner.clone(), 
                                wallet_background: updated_user.wallet_background.clone(), 
                                activity_code: updated_user.clone().activity_code, 
                                twitter_username: updated_user.clone().twitter_username, 
                                facebook_username: updated_user.clone().facebook_username, 
                                discord_username: updated_user.clone().discord_username, 
                                identifier: updated_user.clone().identifier, 
                                user_role: {
                                    match updated_user.user_role.clone(){
                                        UserRole::Admin => "Admin".to_string(),
                                        UserRole::User => "User".to_string(),
                                        _ => "Dev".to_string(),
                                    }
                                },
                                token_time: updated_user.token_time,
                                balance: updated_user.balance,
                                last_login: { 
                                    if updated_user.last_login.is_some(){
                                        Some(updated_user.last_login.unwrap().to_string())
                                    } else{
                                        Some("".to_string())
                                    }
                                },
                                created_at: updated_user.created_at.to_string(),
                                updated_at: updated_user.updated_at.to_string(),
                                mail: updated_user.clone().mail,
                                google_id: updated_user.clone().google_id,
                                microsoft_id: updated_user.clone().microsoft_id,
                                is_mail_verified: updated_user.is_mail_verified,
                                is_phone_verified: updated_user.is_phone_verified,
                                phone_number: updated_user.clone().phone_number,
                                paypal_id: updated_user.clone().paypal_id,
                                account_number: updated_user.clone().account_number,
                                device_id: updated_user.clone().device_id,
                                social_id: updated_user.clone().social_id,
                                cid: updated_user.clone().cid,
                                screen_cid: updated_user.clone().screen_cid,
                                snowflake_id: updated_user.snowflake_id,
                                stars: updated_user.stars,
                                extra: updated_user.clone().extra,
                            }
                        )
                    },
                    Err(e) => {
                        
                        let resp_err = &e.to_string();


                        /* custom error handler */
                        use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::update_phone");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: resp_err,
                            status: 500,
                            is_error: true,
                        };
                        return Err(
                            Ok(HttpResponse::InternalServerError().json(resp))
                        );

                    }
                }



    }

    pub async fn send_phone_verification_code_to(phone_owner_id: i32, user_phone: String, user_ip: String, redis_actor: Addr<RedisActor>, 
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{

        let get_single_user = User::find_by_id(phone_owner_id, connection).await;
        let Ok(single_user) = get_single_user else{
            let resp = Response{
                data: Some(phone_owner_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        //----- don't uncomment it since the user might not enter the code 
        //----- but server has saved his phone in db so he must request again for a new code
        // let get_same_user = User::find_by_phone(&user_phone, connection).await;
        // if get_same_user.is_ok(){

        //     let resp = Response{
        //         data: Some(user_phone),
        //         message: PHONE_EXISTS,
        //         status: 302,
        //         is_error: true,
        //     };
        //     return Err(
        //         Ok(HttpResponse::Found().json(resp))
        //     );
        // }

        /* if the passed in mail was the one inside the db, means it has already been verified */
        if single_user.phone_number.is_some() && 
            single_user.phone_number.unwrap() == user_phone &&
            /* 
                is_mail_verified also must be true since user might 
                entered an expired code which we won't update this 
                field thus he must enter a new code by calling this api
                and if this field isn't set to true we must allow him 
                to get the code otherwise means that his mail is already
                verified.
            */ 
            single_user.is_phone_verified{
            let resp = Response{
                data: Some(phone_owner_id),
                message: ALREADY_VERIFIED_PHONE,
                status: 302,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::Found().json(resp))
            );
        }

        /* 
            if we're here means that the user is trying to verify a new phone so 
            we have to set the is_phone_verified to false, we'll set this to true
            once the user sent the code back to the server
        */
        if single_user.is_phone_verified{

            let res = diesel::update(users.find(phone_owner_id))
                .set(is_phone_verified.eq(false))
                .returning(FetchUser::as_returning())
                .get_result(connection);
        }


        let random_code: String = (0..6).map(|_|{
            let idx = gen_random_idx(random::<u8>() as usize); // idx is one byte cause it's of type u8
            CHARSET[idx] as char // CHARSET is of type slice of utf8 bytes thus we can index it which it's length is 10 bytes (0-9)
        }).collect();

        /* get the user region using the api call */
        let u_country = get_ip_data(user_ip.clone()).await.country.as_str().to_lowercase();

        
        let get_two_mins_later = phonereq::send_code(
            APP_NAME, 
            &random_code, 
            OTP_PROVIDER_DIDNT_SEND_CODE, 
            phone_owner_id, 
            &u_country, 
            &user_phone
        ).await;
        let Ok(two_mins_later) = get_two_mins_later else{
            let err_resp = get_two_mins_later.unwrap_err();
            return Err(err_resp);
        };


        let save_phone_res = UserPhone::save(&user_phone, phone_owner_id, random_code, two_mins_later, connection).await;
        let Ok(_) = save_phone_res else{

            let resp_err = save_phone_res.unwrap_err();
            return Err(resp_err);
        };

        /* if we're here means code has been sent successfully */
        match User::update_phone(phone_owner_id, &user_phone, redis_actor, connection).await{
            Ok(user_data) => Ok(user_data),
            Err(e) => Err(e)
        }

    }

    pub async fn check_phone_verification_code(check_user_verification_request: CheckUserPhoneVerificationRequest, receiver_id: i32, redis_actor: Addr<RedisActor>, 
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{
            

        let get_single_user = User::find_by_id(receiver_id, connection).await;
        let Ok(single_user) = get_single_user else{
            let resp = Response{
                data: Some(receiver_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        if single_user.is_phone_verified{
            
            let resp = Response{
                data: Some(receiver_id),
                message: ALREADY_VERIFIED_PHONE,
                status: 302,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::Found().json(resp))
            );

        }


        let single_user_phone = {
            use crate::schema::users_phones::dsl::*;
            use crate::schema::users_phones;
            let single_user_phone = users_phones
                .filter(users_phones::user_id.eq(receiver_id))
                .filter(users_phones::phone.eq(check_user_verification_request.clone().user_phone))
                .filter(users_phones::code.eq(check_user_verification_request.clone().verification_code))
                .first::<UserPhone>(connection);
            single_user_phone

        };
                        
        let Ok(user_phone) = single_user_phone else{
            let resp = Response{
                data: Some(receiver_id),
                message: NO_PHONE_FOR_THIS_USER,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        let exp_code = user_phone.exp;

        /* calculate the naive datetime from the passed in exp milli timestamp */
        let now = Utc::now();
        let given_time = chrono::DateTime::<Utc>::from_naive_utc_and_offset(
            chrono::NaiveDateTime::from_timestamp_millis(exp_code).unwrap(),
            Utc,
        );

        /* calculate the datetime diff between now and the exp time */
        let duration = now.signed_duration_since(given_time);

        /* code must not be expired */
        if duration.num_minutes() >= 2{ /* make sure that the time code is in not older than 2 mins since we've made a request */
            /* delete the record, user must request the code again */
            let del_res = diesel::delete(users_phones::table
                .filter(users_phones::user_id.eq(receiver_id)))
                .filter(users_phones::phone.eq(check_user_verification_request.clone().user_phone))
                .filter(users_phones::code.eq(check_user_verification_request.clone().verification_code))
                .execute(connection);

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: EXPIRED_PHONE_CODE,
                status: 406,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );

        }

        /* update vat field */
        let save_phonw_res = UserPhone::update_vat(user_phone.id, now.timestamp_millis(), connection).await;
        let Ok(_) = save_phonw_res else{

            let resp_err = save_phonw_res.unwrap_err();
            return Err(resp_err);
        };

        /* delete all the records ralated to the receiver_id with vat 0 */
        let del_res = diesel::delete(users_phones::table
            .filter(users_phones::user_id.eq(receiver_id)))
            .filter(users_phones::vat.eq(0))
            .execute(connection);
        
        /* update is_mail_verified field */
        match User::verify_phone(receiver_id, redis_actor, connection).await{
            Ok(user_data) => Ok(user_data),
            Err(e) => Err(e)
        }


    }

    pub async fn verify_phone(
        phone_owner_id: i32, 
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


            let Ok(user) = User::find_by_id(phone_owner_id, connection).await else{
                let resp = Response{
                    data: Some(phone_owner_id),
                    message: USER_NOT_FOUND,
                    status: 404,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
            };


            match diesel::update(users.find(user.id))
                .set(is_phone_verified.eq(true))
                .returning(FetchUser::as_returning())
                .get_result(connection)
                {
                    Ok(updated_user) => {

                        /* ----------------------------------------------- */
                        /* --------- publish updated user to redis channel */
                        /* ----------------------------------------------- */
                        /* 
                            once the user updates his info we'll publish new updated user to redis channel and in
                            other parts we start to subscribe to the new updated user topic then once we receive 
                            the new user we'll start updating user fans and user nfts 
                        */
                        
                        let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                        events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;


                        Ok(
                            UserData { 
                                id: updated_user.id, 
                                region: updated_user.region.clone(),
                                username: updated_user.clone().username, 
                                bio: updated_user.bio.clone(),
                                avatar: updated_user.avatar.clone(),
                                banner: updated_user.banner.clone(),
                                wallet_background: updated_user.wallet_background.clone(),
                                activity_code: updated_user.clone().activity_code, 
                                twitter_username: updated_user.clone().twitter_username, 
                                facebook_username: updated_user.clone().facebook_username, 
                                discord_username: updated_user.clone().discord_username, 
                                identifier: updated_user.clone().identifier, 
                                user_role: {
                                    match updated_user.user_role.clone(){
                                        UserRole::Admin => "Admin".to_string(),
                                        UserRole::User => "User".to_string(),
                                        _ => "Dev".to_string(),
                                    }
                                },
                                token_time: updated_user.token_time,
                                balance: updated_user.balance,
                                last_login: { 
                                    if updated_user.last_login.is_some(){
                                        Some(updated_user.last_login.unwrap().to_string())
                                    } else{
                                        Some("".to_string())
                                    }
                                },
                                created_at: updated_user.created_at.to_string(),
                                updated_at: updated_user.updated_at.to_string(),
                                mail: updated_user.clone().mail,
                                google_id: updated_user.clone().google_id,
                                microsoft_id: updated_user.clone().microsoft_id,
                                is_mail_verified: updated_user.is_mail_verified,
                                is_phone_verified: updated_user.is_phone_verified,
                                phone_number: updated_user.clone().phone_number,
                                paypal_id: updated_user.clone().paypal_id,
                                account_number: updated_user.clone().account_number,
                                device_id: updated_user.clone().device_id,
                                social_id: updated_user.clone().social_id,
                                cid: updated_user.clone().cid,
                                screen_cid: updated_user.clone().screen_cid,
                                snowflake_id: updated_user.snowflake_id,
                                stars: updated_user.stars,
                                extra: updated_user.clone().extra,
                            }
                        )
                    },
                    Err(e) => {
                        
                        let resp_err = &e.to_string();


                        /* custom error handler */
                        use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::verify_phone");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: resp_err,
                            status: 500,
                            is_error: true,
                        };
                        return Err(
                            Ok(HttpResponse::InternalServerError().json(resp))
                        );

                    }
                }



    }

    pub async fn verify_mail(
        mail_owner_id: i32, 
        redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


            let Ok(user) = User::find_by_id(mail_owner_id, connection).await else{
                let resp = Response{
                    data: Some(mail_owner_id),
                    message: USER_NOT_FOUND,
                    status: 404,
                    is_error: true,
                };
                return Err(
                    Ok(HttpResponse::NotFound().json(resp))
                );
            };

            let new_balance = if user.balance.is_none(){5} else{user.balance.unwrap() + 5};
            match diesel::update(users.find(user.id))
                .set(
                    (
                        is_mail_verified.eq(true),
                        balance.eq(new_balance)
                    )
                )
                .returning(FetchUser::as_returning())
                .get_result(connection)
                {
                    Ok(updated_user) => {

                        /* ----------------------------------------------- */
                        /* --------- publish updated user to redis channel */
                        /* ----------------------------------------------- */
                        /* 
                            once the user updates his info we'll publish new updated user to redis channel and in
                            other parts we start to subscribe to the new updated user topic then once we receive 
                            the new user we'll start updating user fans and user nfts 
                        */
                        
                        let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                        events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;


                        Ok(
                            UserData { 
                                id: updated_user.id, 
                                region: updated_user.region.clone(),
                                username: updated_user.clone().username, 
                                bio: updated_user.bio.clone(),
                                avatar: updated_user.avatar.clone(),
                                banner: updated_user.banner.clone(),
                                wallet_background: updated_user.wallet_background.clone(),
                                activity_code: updated_user.clone().activity_code, 
                                twitter_username: updated_user.clone().twitter_username, 
                                facebook_username: updated_user.clone().facebook_username, 
                                discord_username: updated_user.clone().discord_username, 
                                identifier: updated_user.clone().identifier, 
                                user_role: {
                                    match updated_user.user_role.clone(){
                                        UserRole::Admin => "Admin".to_string(),
                                        UserRole::User => "User".to_string(),
                                        _ => "Dev".to_string(),
                                    }
                                },
                                token_time: updated_user.token_time,
                                balance: updated_user.balance,
                                last_login: { 
                                    if updated_user.last_login.is_some(){
                                        Some(updated_user.last_login.unwrap().to_string())
                                    } else{
                                        Some("".to_string())
                                    }
                                },
                                created_at: updated_user.created_at.to_string(),
                                updated_at: updated_user.updated_at.to_string(),
                                mail: updated_user.clone().mail,
                                google_id: updated_user.clone().google_id,
                                microsoft_id: updated_user.clone().microsoft_id,
                                is_mail_verified: updated_user.is_mail_verified,
                                is_phone_verified: updated_user.is_phone_verified,
                                phone_number: updated_user.clone().phone_number,
                                paypal_id: updated_user.clone().paypal_id,
                                account_number: updated_user.clone().account_number,
                                device_id: updated_user.clone().device_id,
                                social_id: updated_user.clone().social_id,
                                cid: updated_user.clone().cid,
                                screen_cid: updated_user.clone().screen_cid,
                                snowflake_id: updated_user.snowflake_id,
                                stars: updated_user.stars,
                                extra: updated_user.clone().extra,
                            }
                        )
                    },
                    Err(e) => {
                        
                        let resp_err = &e.to_string();


                        /* custom error handler */
                        use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                            
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();  
                        let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::verify_mail");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        let resp = Response::<&[u8]>{
                            data: Some(&[]),
                            message: resp_err,
                            status: 500,
                            is_error: true,
                        };
                        return Err(
                            Ok(HttpResponse::InternalServerError().json(resp))
                        );

                    }
                }



    }

    pub async fn reset_password(forgot_pswd: ForgotPasswordRequest, 
        connection: &mut DbPoolConnection) 
        -> Result<UserData, PanelHttpResponse>{

        let get_user = Self::find_by_mail(&forgot_pswd.mail, connection).await;
        let Ok(user) = get_user else{
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };


        if !user.is_mail_verified{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: NOT_VERIFIED_MAIL,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }


        let random_chars = helpers::misc::gen_random_passwd(8);
        let hashed_password = Self::hash_pswd(&random_chars).unwrap();

        /* sending mail */
        let sent_pswd = mailreq::send_reset_pass_mail(APP_NAME, user.id, &forgot_pswd.mail, &random_chars).await;
        let Ok(_) = sent_pswd else{
            let err_resp = sent_pswd.unwrap_err();
            return Err(err_resp);
        };

        // updating user 
        match diesel::update(users.find(user.id))
            .set(pswd.eq(hashed_password))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {
                    
                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            region: updated_user.region.clone(),
                            username: updated_user.clone().username, 
                            bio: updated_user.bio.clone(),
                            avatar: updated_user.avatar.clone(),
                            banner: updated_user.banner.clone(),
                            wallet_background: updated_user.wallet_background.clone(),
                            activity_code: updated_user.clone().activity_code, 
                            twitter_username: updated_user.clone().twitter_username, 
                            facebook_username: updated_user.clone().facebook_username, 
                            discord_username: updated_user.clone().discord_username, 
                            identifier: updated_user.clone().identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            balance: updated_user.balance,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            mail: updated_user.clone().mail,
                            google_id: updated_user.clone().google_id,
                            microsoft_id: updated_user.clone().microsoft_id,
                            is_mail_verified: updated_user.is_mail_verified,
                            is_phone_verified: updated_user.is_phone_verified,
                            phone_number: updated_user.clone().phone_number,
                            paypal_id: updated_user.clone().paypal_id,
                            account_number: updated_user.clone().account_number,
                            device_id: updated_user.clone().device_id,
                            social_id: updated_user.clone().social_id,
                            cid: updated_user.clone().cid,
                            screen_cid: updated_user.clone().screen_cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars,
                            extra: updated_user.clone().extra,
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::reset_password");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }
        

    }

    pub async fn update_password(owner_id: i32, new_pass_request: NewPasswordRequest, 
        connection: &mut DbPoolConnection) 
        -> Result<UserData, PanelHttpResponse>{

        let get_user = Self::find_by_id(owner_id, connection).await;
        let Ok(user) = get_user else{
            let err_resp = get_user.unwrap_err();
            return Err(err_resp);
        };


        if user.mail.is_none() || (user.mail.is_some() && !user.is_mail_verified){
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: NOT_VERIFIED_MAIL,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

        let is_old_password_correct = user.verify_pswd(&new_pass_request.old_password).unwrap();
        if !is_old_password_correct{
            let resp = Response::<&[u8]>{
                data: Some(&[]),
                message: OLD_PASSWORD_INCORRECT,
                status: 406,
                is_error: true
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );
        }

        let hashed_password = Self::hash_pswd(&new_pass_request.new_password).unwrap();

        /* sending mail */
        let sent_pswd = mailreq::send_new_pass_mail(APP_NAME, user.id, &user.mail.unwrap(), &new_pass_request.new_password).await;
        let Ok(_) = sent_pswd else{
            let err_resp = sent_pswd.unwrap_err();
            return Err(err_resp);
        };

        // updating user 
        match diesel::update(users.find(user.id))
            .set(pswd.eq(hashed_password))
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {
                    
                    Ok(
                        UserData { 
                            id: updated_user.id, 
                            region: updated_user.region.clone(),
                            username: updated_user.clone().username, 
                            bio: updated_user.bio.clone(),
                            avatar: updated_user.avatar.clone(),
                            banner: updated_user.banner.clone(),
                            wallet_background: updated_user.wallet_background.clone(),
                            activity_code: updated_user.clone().activity_code, 
                            twitter_username: updated_user.clone().twitter_username, 
                            facebook_username: updated_user.clone().facebook_username, 
                            discord_username: updated_user.clone().discord_username, 
                            identifier: updated_user.clone().identifier, 
                            user_role: {
                                match updated_user.user_role.clone(){
                                    UserRole::Admin => "Admin".to_string(),
                                    UserRole::User => "User".to_string(),
                                    _ => "Dev".to_string(),
                                }
                            },
                            token_time: updated_user.token_time,
                            balance: updated_user.balance,
                            last_login: { 
                                if updated_user.last_login.is_some(){
                                    Some(updated_user.last_login.unwrap().to_string())
                                } else{
                                    Some("".to_string())
                                }
                            },
                            created_at: updated_user.created_at.to_string(),
                            updated_at: updated_user.updated_at.to_string(),
                            mail: updated_user.clone().mail,
                            google_id: updated_user.clone().google_id,
                            microsoft_id: updated_user.clone().microsoft_id,
                            is_mail_verified: updated_user.is_mail_verified,
                            is_phone_verified: updated_user.is_phone_verified,
                            phone_number: updated_user.clone().phone_number,
                            paypal_id: updated_user.clone().paypal_id,
                            account_number: updated_user.clone().account_number,
                            device_id: updated_user.clone().device_id,
                            social_id: updated_user.clone().social_id,
                            cid: updated_user.clone().cid,
                            screen_cid: updated_user.clone().screen_cid,
                            snowflake_id: updated_user.snowflake_id,
                            stars: updated_user.stars,
                            extra: updated_user.clone().extra,
                        }
                    )
                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();

                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                        
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::updated_password");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }
        

    }

    pub async fn send_mail_verification_code_to(mail_owner_id: i32, user_mail: String, redis_client: RedisClient, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{


        let get_single_user = User::find_by_id(mail_owner_id, connection).await;
        let Ok(single_user) = get_single_user else{
            let resp = Response{
                data: Some(mail_owner_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        let get_same_user_by_mail = User::find_by_mail(&user_mail, connection).await;
        let get_same_user_by_identifier = User::find_by_identifier(&user_mail, connection).await;

        if get_same_user_by_mail.is_err() && get_same_user_by_identifier.is_err(){

            let resp = Response{
                data: Some(user_mail),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );   
        }

        if (get_same_user_by_mail.is_ok() && mail_owner_id != get_same_user_by_mail.unwrap().id) || 
            (get_same_user_by_identifier.is_ok() && mail_owner_id != get_same_user_by_identifier.unwrap().id){

            let resp = Response{
                data: Some(user_mail),
                message: MAIL_OWNER_IS_NOT_CALLER,
                status: 403,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::Forbidden().json(resp))
            );
        }

        //----- don't uncomment it since the user might not enter the code 
        //----- but server has saved his mail in db so he must request again for a new code
        // let get_same_user = User::find_by_mail(&user_mail, connection).await;
        // if get_same_user.is_ok(){

        //     let resp = Response{
        //         data: Some(user_mail),
        //         message: MAIL_EXISTS,
        //         status: 302,
        //         is_error: true,
        //     };
        //     return Err(
        //         Ok(HttpResponse::Found().json(resp))
        //     );
        // }


        /* if the passed in mail was the one inside the db, means it has already been verified */
        if single_user.mail.is_some() && 
            single_user.mail.unwrap() == user_mail &&
            /* 
                is_mail_verified also must be true since user might 
                entered an expired code which we won't update this 
                field thus he must enter a new code by calling this api
                and if this field isn't set to true we must allow him 
                to get the code otherwise means that his mail is already
                verified.
            */ 
            single_user.is_mail_verified{
            let resp = Response{
                data: Some(mail_owner_id),
                message: ALREADY_VERIFIED_MAIL,
                status: 302,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::Found().json(resp))
            );
        }

        /* 
            if we're here means that the user is trying to verify a new mail so 
            we have to set the is_mail_verified to false, we'll set this to true
            once the user sent the code back to the server
        */
        if single_user.is_mail_verified{

            let res = diesel::update(users.find(mail_owner_id))
                .set(is_mail_verified.eq(false))
                .returning(FetchUser::as_returning())
                .get_result(connection);
        }


        let random_code: String = (0..8).map(|_|{
            let idx = gen_random_idx(random::<u8>() as usize); // idx is one byte cause it's of type u8
            CHARSET[idx] as char // CHARSET is of type slice of utf8 bytes thus we can index it which it's length is 10 bytes (0-9)
        }).collect();

        /* sending mail */
        let get_five_mins_later = mailreq::send_mail(APP_NAME, mail_owner_id, &user_mail, &random_code).await;
        let Ok(five_mins_later) = get_five_mins_later else{
            let err_resp = get_five_mins_later.unwrap_err();
            return Err(err_resp);
        };

        let save_mail_res = UserMail::save(&user_mail, mail_owner_id, random_code, five_mins_later, connection).await;
        let Ok(_) = save_mail_res else{

            let resp_err = save_mail_res.unwrap_err();
            return Err(resp_err);
        };

        /* if we're here means code has been sent successfully */
        match User::update_mail(mail_owner_id, &user_mail,  redis_client, redis_actor, connection).await{
            Ok(user_data) => Ok(user_data),
            Err(e) => Err(e)
        }

    }

    pub async fn check_mail_verification_code(check_user_verification_request: CheckUserMailVerificationRequest, receiver_id: i32, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserData, PanelHttpResponse>{
            

        let get_single_user = User::find_by_id(receiver_id, connection).await;
        let Ok(single_user) = get_single_user else{
            let resp = Response{
                data: Some(receiver_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        if single_user.is_mail_verified{
            
            let resp = Response{
                data: Some(receiver_id),
                message: ALREADY_VERIFIED_MAIL,
                status: 302,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::Found().json(resp))
            );

        }


        let single_user_mail = {
            use crate::schema::users_mails::dsl::*;
            use crate::schema::users_mails;
            let single_user_mail = users_mails
                .filter(users_mails::user_id.eq(receiver_id))
                .filter(users_mails::mail.eq(check_user_verification_request.clone().user_mail))
                .filter(users_mails::code.eq(check_user_verification_request.clone().verification_code))
                .first::<UserMail>(connection);
            single_user_mail

        };
                        
        let Ok(user_mail) = single_user_mail else{
            let resp = Response{
                data: Some(receiver_id),
                message: NO_MAIL_FOR_THIS_USER,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        let exp_code = user_mail.exp;

        /* calculate the naive datetime from the passed in exp milli timestamp */
        let now = Utc::now();
        let given_time = chrono::DateTime::<Utc>::from_naive_utc_and_offset(
            chrono::NaiveDateTime::from_timestamp_millis(exp_code).unwrap(),
            Utc,
        );

        /* calculate the datetime diff between now and the exp time */
        let duration = now.signed_duration_since(given_time);

        /* code must not be expired */
        if duration.num_minutes() >= 5{ /* make sure that the time code is in not older than 5 mins since we've made a request */
            /* delete the record, user must request the code again */
            let del_res = diesel::delete(users_mails::table
                .filter(users_mails::user_id.eq(receiver_id)))
                .filter(users_mails::mail.eq(check_user_verification_request.clone().user_mail))
                .filter(users_mails::code.eq(check_user_verification_request.clone().verification_code))
                .execute(connection);

            let resp = Response::<'_, &[u8]>{
                data: Some(&[]),
                message: EXPIRED_MAIL_CODE,
                status: 406,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotAcceptable().json(resp))
            );

        }

        /* update vat field */
        let save_mail_res = UserMail::update_vat(user_mail.id, now.timestamp_millis(), connection).await;
        let Ok(_) = save_mail_res else{

            let resp_err = save_mail_res.unwrap_err();
            return Err(resp_err);
        };

        /* delete all the records ralated to the receiver_id with vat 0 */
        let del_res = diesel::delete(users_mails::table
            .filter(users_mails::user_id.eq(receiver_id)))
            .filter(users_mails::vat.eq(0))
            .execute(connection);
        
        /* update is_mail_verified field */
        match User::verify_mail(receiver_id, redis_actor, connection).await{
            Ok(user_data) => Ok(user_data),
            Err(e) => Err(e)
        }


    }

    pub async fn delete_wallet_by_admin(u_id: i32, connection: &mut DbPoolConnection) -> Result<FetchUser, PanelHttpResponse>{

        let wscid: Option<String> = None;
        let wcid: Option<String> = None;
        let wregion: Option<String> = None;

        let Ok(user) = User::find_by_id(u_id, connection).await else{
            let resp = Response{
                data: Some(u_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );
        };

        let user_screen_cid = user.screen_cid.unwrap_or_default();
        let user_cid = user.cid.unwrap_or_default();

        match diesel::update(users.find(u_id))
            .set(
                (
                    region.eq(wregion),
                    screen_cid.eq(wscid),
                    cid.eq(wcid),
                    username.eq(String::from("")),
                )
            )
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {
                    
                    use crate::models::users_deposits::UserDeposit;
                    use crate::models::users_withdrawals::UserWithdrawal;
                    let dels_col_info = UserCollection::delete_all_collections_by_owner_screen_cid(&user_screen_cid, connection).await;
                    let dels_deposits_info_cid = UserDeposit::delete_all_by_cid(&user_cid, connection).await;
                    let dels_deposits_info_scid = UserDeposit::delete_all_by_scid(&user_screen_cid, connection).await;
                    let dels_user_fan_info = UserFan::delete_by_screen_cid(&user_screen_cid, connection).await;
                    let dels_user_gal_info = UserPrivateGallery::delete_by_screen_cid(&user_screen_cid, connection).await;
                    let dels_user_gal_info = UserWithdrawal::delete_by_cid(&user_cid, connection).await;
                    let dels_inv_frd_from_gal = UserPrivateGallery::remove_scid_from_invited_friends(&user_screen_cid, connection).await;

                    Ok(updated_user)
                },
                Err(e) => {

                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                     
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "User::delete_wallet_by_admin");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    }

}


impl Id{

    pub async fn new_or_update(id_: NewIdRequest, id_owner: i32, id_username: String, user_ip: String, redis_client: RedisClient, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<Id, PanelHttpResponse>{

        let Ok(user) = User::find_by_id(id_owner, connection).await else{
            let resp = Response{
                data: Some(id_owner),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            ); 
        };

        

        let u_country = get_ip_data(user_ip.clone()).await.country.as_str().to_lowercase();

        match user.cid{
            /* we'll be here only if the old_cid is not an empty string */
            Some(old_cid) if !old_cid.is_empty() => { 

                /* updating other fields except cid and snowflake id */
                match diesel::update(users.find(id_owner))
                    .set(
                (
                        // update only region and username
                            region.eq(u_country),
                            username.eq(id_.username.clone().to_lowercase()), // username must be unique and case sensitive
                        )
                    )
                    .returning(FetchUser::as_returning())
                    .get_result(connection)
                    {
                        Ok(updated_user) => {

                            /* ----------------------------------------------- */
                            /* --------- publish updated user to redis channel */
                            /* ----------------------------------------------- */
                            /* 
                                once the user updates his info we'll publish new updated user to redis channel and in
                                other parts we start to subscribe to the new updated user topic then once we receive 
                                the new user we'll start updating user fans and user nfts 
                            */
                            
                            let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                            events::publishers::user::emit(redis_actor, "on_user_update", &json_stringified_updated_user).await;

                            let user_data = UserData { 
                                id: updated_user.id, 
                                region: updated_user.region.clone(),
                                username: updated_user.clone().username, 
                                bio: updated_user.bio.clone(),
                                avatar: updated_user.avatar.clone(),
                                banner: updated_user.banner.clone(),
                                wallet_background: updated_user.wallet_background.clone(),
                                activity_code: updated_user.clone().activity_code, 
                                twitter_username: updated_user.clone().twitter_username, 
                                facebook_username: updated_user.clone().facebook_username, 
                                discord_username: updated_user.clone().discord_username, 
                                identifier: updated_user.clone().identifier, 
                                user_role: {
                                    match updated_user.user_role.clone(){
                                        UserRole::Admin => "Admin".to_string(),
                                        UserRole::User => "User".to_string(),
                                        _ => "Dev".to_string(),
                                    }
                                },
                                token_time: updated_user.token_time,
                                balance: updated_user.balance,
                                last_login: { 
                                    if updated_user.last_login.is_some(){
                                        Some(updated_user.last_login.unwrap().to_string())
                                    } else{
                                        Some("".to_string())
                                    }
                                },
                                created_at: updated_user.created_at.to_string(),
                                updated_at: updated_user.updated_at.to_string(),
                                mail: updated_user.clone().mail,
                                google_id: updated_user.clone().google_id,
                                microsoft_id: updated_user.clone().microsoft_id,
                                is_mail_verified: updated_user.is_mail_verified,
                                is_phone_verified: updated_user.is_phone_verified,
                                phone_number: updated_user.clone().phone_number,
                                paypal_id: updated_user.clone().paypal_id,
                                account_number: updated_user.clone().account_number,
                                device_id: updated_user.clone().device_id,
                                social_id: updated_user.clone().social_id,
                                cid: updated_user.clone().cid,
                                screen_cid: updated_user.clone().screen_cid,
                                snowflake_id: updated_user.snowflake_id,
                                stars: updated_user.stars,
                                extra: updated_user.extra,
                            };

                            let resp = Response{
                                data: Some(user_data),
                                message: CID_RECORD_UPDATED,
                                status: 302,
                                is_error: false
                            };
                            return Err(
                                Ok(HttpResponse::Found().json(resp))
                            ); 
                        },
                        Err(e) => {
                            
                            let resp_err = &e.to_string();

                            /* custom error handler */
                            use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                                
                            let error_content = &e.to_string();
                            let error_content = error_content.as_bytes().to_vec();  
                            let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Id::new_or_update");
                            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                            let resp = Response::<&[u8]>{
                                data: Some(&[]),
                                message: resp_err,
                                status: 500,
                                is_error: true,
                            };
                            return Err(
                                Ok(HttpResponse::InternalServerError().json(resp))
                            );

                        }
                    }

            },
            _ => {

                /* --------------------------------------------------------------------------- */
                //   generating keypair using wallexerr, signing and verification using web3
                /* --------------------------------------------------------------------------- */
                /* we'll create a new private gallery if the verification was ok */
                let wallet = walletreq::evm::get_wallet();
                let data_to_be_signed = serde_json::json!({
                    "owner_cid": wallet.secp256k1_public_address.as_ref().unwrap(),
                    "gal_name": format!("{} first private room at time {}", id_username, chrono::Local::now().to_string()),
                    "gal_description": format!("{} first private room at time {}", id_username, chrono::Local::now().to_string()),
                    "extra": None::<Option<serde_json::Value>> // serde needs to know the exact type of extra which can be any json value data
                });

                let sign_res = walletreq::evm::sign(
                    wallet.clone(), 
                    data_to_be_signed.to_string().as_str()
                ).await;

                let signed_data = sign_res.clone().0;

                info!("sig :::: {}", hex::encode(&signed_data.signature.0));
                info!("v :::: {}", signed_data.v);
                info!("r :::: {}", hex::encode(&signed_data.r.0));
                info!("s :::: {}", hex::encode(&signed_data.s.0));
                info!("hash data :::: {}", sign_res.1);

                let verification_res = walletreq::evm::verify_signature(
                    wallet.secp256k1_public_address.as_ref().unwrap().to_string(),
                    hex::encode(&signed_data.signature.0).as_str(),
                    sign_res.1.as_str()
                ).await;
                
                if verification_res.is_ok(){
                    
                    info!("✅ valid signature");

                } else{ 
                    
                    // can't create private gallery then :(
                    error!("🔴 invalid signature");
                    /* terminate the caller with invalid signature */
                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: INVALID_SIGNATURE,
                        status: 406,
                        is_error: true
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    );
                }
                /* ------------------------------------------------------------ */

                /* generating snowflake id */
                let machine_id = std::env::var("MACHINE_ID").unwrap_or("1".to_string()).parse::<i32>().unwrap();
                let node_id = std::env::var("NODE_ID").unwrap_or("1".to_string()).parse::<i32>().unwrap();
                let mut id_generator_generator = SnowflakeIdGenerator::new(machine_id, node_id);
                let new_snowflake_id = id_generator_generator.real_time_generate();
                let new_snowflake_id = Some(new_snowflake_id);

                Ok(
                    Id{ 
                        user_id: id_owner,
                        region: get_ip_data(user_ip).await.country.as_str().to_lowercase(), // never trust user input
                        username: id_username, 
                        device_id: id_.device_id, 
                        new_snowflake_id,
                        new_cid: wallet.secp256k1_public_key, /* secp256k1 */
                        screen_cid: wallet.secp256k1_public_address, /* secp256k1 */
                        signer: wallet.secp256k1_secret_key, /* secp256k1 */
                        mnemonic: wallet.secp256k1_mnemonic,
                    }
                )

            }
        } 

    }

    pub async fn save(&mut self, redis_client: RedisClient, redis_actor: Addr<RedisActor>,
        connection: &mut DbPoolConnection) -> Result<UserIdResponse, PanelHttpResponse>{
        
        let get_user = User::find_by_id(self.user_id, connection).await;
        let Ok(user) = get_user else{
            let resp = Response{
                data: Some(self.user_id),
                message: USER_NOT_FOUND,
                status: 404,
                is_error: true,
            };
            return Err(
                Ok(HttpResponse::NotFound().json(resp))
            );  
        };

        match diesel::update(users.find(self.user_id))
            .set(
                (   
                    /* 
                        can't return heap data of type String we must clone them or use their 
                        borrowed form or return the static version of their slice like &'static str
                    */
                    username.eq(self.username.clone().to_lowercase()), // username must be unique and case sensitive
                    region.eq(self.region.clone()),
                    device_id.eq(self.device_id.clone()),
                    cid.eq(self.new_cid.clone().unwrap()),
                    screen_cid.eq(self.screen_cid.clone().unwrap()),
                    snowflake_id.eq(self.new_snowflake_id),
                )
            )
            .returning(FetchUser::as_returning())
            .get_result(connection)
            {
                Ok(updated_user) => {

                    /* ----------------------------------------------- */
                    /* --------- publish updated user to redis channel */
                    /* ----------------------------------------------- */
                    /* 
                        once the user updates his info we'll publish new updated user to redis channel and in
                        other parts we start to subscribe to the new updated user topic then once we receive 
                        the new user we'll start updating user fans and user nfts 
                    */
                    
                    let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user).unwrap();
                    events::publishers::user::emit(redis_actor.clone(), "on_user_update", &json_stringified_updated_user).await;

                    let new_balance = if user.balance.is_none(){5} else{user.balance.unwrap() + 5};
                    match User::update_balance(self.user_id, "Airdrop|5", "credit",
                        new_balance, redis_client.clone(), redis_actor.clone(), connection).await{

                        Ok(updated_user_data) => {

                            /* ----------------------------------------------- */
                            /* --------- publish updated user to redis channel */
                            /* ----------------------------------------------- */
                            /* 
                                once the user updates his info we'll publish new updated user to redis channel and in
                                other parts we start to subscribe to the new updated user topic then once we receive 
                                the new user we'll start updating user fans and user nfts 
                            */
                            
                            let json_stringified_updated_user = serde_json::to_string_pretty(&updated_user_data).unwrap();
                            events::publishers::user::emit(redis_actor.clone(), "on_user_update", &json_stringified_updated_user).await;

                            Ok(
                                UserIdResponse { 
                                    id: updated_user_data.id, 
                                    region: updated_user_data.clone().region.unwrap(),
                                    username: updated_user_data.clone().username, 
                                    activity_code: updated_user_data.clone().activity_code, 
                                    twitter_username: updated_user_data.clone().twitter_username, 
                                    facebook_username: updated_user_data.clone().facebook_username, 
                                    discord_username: updated_user_data.clone().discord_username, 
                                    identifier: updated_user_data.clone().identifier, 
                                    user_role: updated_user_data.user_role.clone(),
                                    token_time: updated_user_data.clone().token_time,
                                    balance: updated_user_data.clone().balance,
                                    last_login: if updated_user_data.clone().last_login.is_some(){
                                            Some(updated_user_data.clone().last_login.unwrap().to_string())
                                        } else{
                                            Some("".to_string())
                                        }
                                    ,
                                    created_at: updated_user_data.clone().created_at.to_string(),
                                    updated_at: updated_user_data.clone().updated_at.to_string(),
                                    mail: updated_user_data.clone().mail,
                                    google_id: updated_user_data.clone().google_id,
                                    microsoft_id: updated_user_data.clone().microsoft_id,
                                    is_mail_verified: updated_user_data.clone().is_mail_verified,
                                    is_phone_verified: updated_user_data.clone().is_phone_verified,
                                    phone_number: updated_user_data.clone().phone_number,
                                    paypal_id: updated_user_data.clone().paypal_id,
                                    account_number: updated_user_data.clone().account_number,
                                    device_id: updated_user_data.clone().device_id,
                                    social_id: updated_user_data.clone().social_id,
                                    cid: updated_user_data.clone().cid,
                                    screen_cid: self.screen_cid.clone(),
                                    signer: self.signer.clone(),
                                    mnemonic: self.mnemonic.clone(),
                                    snowflake_id: updated_user_data.clone().snowflake_id,
                                    stars: updated_user_data.clone().stars,
                                    bio: updated_user_data.clone().bio,
                                    avatar: updated_user_data.clone().avatar,
                                    banner: updated_user_data.clone().banner,
                                    wallet_background: updated_user_data.clone().wallet_background,
                                    extra: updated_user_data.clone().extra,
                                }
                            )
                            

                        },
                        Err(resp) => {
                            Err(resp) /* because error part of this method is the actix result response */
                        }
                    }

                },
                Err(e) => {
                    
                    let resp_err = &e.to_string();


                    /* custom error handler */
                    use helpers::error::{ErrorKind, StorageError::{Diesel, Redis}, PanelError};
                    let error_content = &e.to_string();
                    let error_content = error_content.as_bytes().to_vec();  
                    let error_instance = PanelError::new(*STORAGE_IO_ERROR_CODE, error_content, ErrorKind::Storage(Diesel(e)), "Id::save");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: resp_err,
                        status: 500,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::InternalServerError().json(resp))
                    );

                }
            }

    
    }
    
}
