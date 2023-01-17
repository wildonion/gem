



use serde_json::Value;
use serde::{Serialize, Deserialize};
use mongodb::bson::{self, oid::ObjectId, doc}; //// self referes to the bson struct itself cause there is a struct called bson inside the bson.rs file
use argon2::{self, Config};
use std::{env, collections::HashMap};
use borsh::{BorshDeserialize, BorshSerialize};








// NOTE - each collection has some documents which can be deserailzed into a struct inside the rust
// NOTE - to send some data back to the user we must serialize that data struct into the json and from there to utf8 to pass through the socket
// NOTE - to send fetched data from mongodb which is a bson object back to the user we must first deserialize the bson into the struct then serialize to json to serialize to utf8 to send back to the user which is a bson object back to the user we must first deserialize the bson into the json to send back to the user
// NOTE - RegisterResponse struct doesn't have the pwd field cause we don't want the user see the password if there was any user already inside the collection







/*
  ------------------------------------------------------------------------------------------------
| this struct will be used to deserialize user role update info json from client into this struct
| ------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UserRoleUpdateRequest{
    pub user_id: String, //// this is the id of the user took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub role_id: String, //// this is the id of the role took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub event_id: String, //// this is the id of the role took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  ------------------------------------------------------------------------------------------------
| this struct will be used to deserialize user side update info json from client into this struct
| ------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UserSideUpdateRequest{
    pub user_id: String, //// this is the id of the user took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub side_id: String, //// this is the id of the role took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub event_id: String, //// this is the id of the role took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  --------------------------------------------------------------------------------------------------
| this struct will be used to deserialize user status update info json from client into this struct
| --------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UserStatusUpdateRequest{
    pub user_id: String, //// this is the id of the user took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub status: u8, //// one of the status constant value defined in constants.rs
    pub event_id: String, //// this is the id of the user took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize user info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GetUserRequest{ //// we don't need _id field in this struct cause it'll be generated when we want to insert role info into the mongodb 
    pub _id: String, //// this is the id of the event took from the mongodb events collection and will be stored as String later we'll serialize it into bson mongodb ObjectId
}


/*
  --------------------------------------------------------------------------------------------------------------
| this struct will be used to serialize user info after update any field into the json to send back to the user
| --------------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UserUpdateResponse{
  pub username: String,
  pub phone: String,
  pub access_level: u8, // NOTE - 0 means dev, 1 means admin, 2 means user
  pub status: u8,
  pub role_id: Option<ObjectId>,
  pub side_id: Option<ObjectId>,
  pub created_at: Option<i64>,
  pub updated_at: Option<i64>,
  pub last_login_time: Option<i64>,
  pub wallet_address: Option<String>,
  pub balance: Option<u128>,
}


/*
  -----------------------------------------------------------------------------------------------------------------------------
| this struct will be used to deserialize the SMS response return part coming from the career to serialize to into this struct
| -----------------------------------------------------------------------------------------------------------------------------
|                                             >>>>> NOT IMPORTANT SCHEMA <<<<<
|
*/
#[derive(Default, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct SMSResponseReturn{
    pub status: u16,
    pub message: String,
}


/*
  ------------------------------------------------------------------------------------------------------------------------------
| this struct will be used to deserialize the SMS response entries part coming from the career to serialize to into this struct
| ------------------------------------------------------------------------------------------------------------------------------
|                                             >>>>> NOT IMPORTANT SCHEMA <<<<<
|
*/
#[derive(Default, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, Clone)]
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


/*
  ----------------------------------------------------------------------------------------------------------------------------------
| this struct will be used to deserialize the SMS response coming from the career to serialize to check the status code from career
| ----------------------------------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Serialize, Deserialize, Debug, Clone)] //// can't implement the Default trait for extra field cause Default is not implemented for Value enum
pub struct OTPCareerResponse{
    #[serde(flatten)] // NOTE - #[serde(flatten)] proc macro attribute can be used for factoring common keys into a shared structure, or for capturing remaining fields into a map with arbitrary string keys
    pub extra: HashMap<String, Value>, //// the OTP career response after deserializing to this struct will be like so: {"return": {"status": ..., "message": ...}, "entries": [{"messageid": ..., ...}]}
}


/*
  ----------------------------------------------------------------------------------------------------------------------------------
| this struct will be used to deserialize the SMS response coming from the career to serialize to check the status code from career
| ----------------------------------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)] //// can't implement the Default trait for extra field cause Default is not implemented for Value enum
pub struct SMSResponse{
    pub r#return: SMSResponseReturn, //// use r# to escape reserved keywords to use them as identifiers 
    pub entries: Vec<SMSResponseEntries>,
}


/*
  ----------------------------------------------------------------------------------------
| this struct will be used to deserialize register info json from client into this struct
| ----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct RegisterRequest{ // NOTE - those Option values can be None tho
    pub username: String,
    pub phone: String,
    pub pwd: String, //// hashed password
    pub status: u8, //// this is the status of the player - this field is 0 initially and is the last status and info of the player during the game
    pub access_level: Option<u8>, // NOTE - 0 means dev, 1 means admin, 2 means user - we set this field to Option cause we don't want to pass the access_level inside the request body thus it should be None initially, we'll fill it inside the server
    pub role_id: Option<ObjectId>, //// this is the id from the roles collection - this field is None initially and is the last status and info of the player during the game
    pub side_id: Option<ObjectId>, //// this is the id from the sides collection - this field is None initially and is the last status and info of the player during the game
    pub created_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
    pub updated_at: Option<i64>, //// we set this field to Option cause we don't want to pass the updated time inside the request body thus it should be None initially, we'll fill it inside the server
    pub last_login_time: Option<i64>, //// we set this field to Option cause we don't want to pass the last login time inside the request body thus it should be None initially, we'll fill it inside the server
    pub wallet_address: Option<String>,
    pub balance: Option<u128>
}


/*
  -------------------------------------------------------------------------------------
| this struct will be used to deserialize login info json from client into this struct
| -------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct LoginRequest{
    pub username: String,
    pub pwd: String,
}


/*
  --------------------------------------------------------------------------------------------------------------------------
| this struct will be used to deserialize user info bson from the mongodb into this struct to serialize as json back to user
| --------------------------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct LoginResponse{ // NOTE - those Option values can be None tho
    pub _id: Option<ObjectId>, //// this is the user id inside the users collection
    pub access_token: String,
    pub username: String,
    pub phone: String,
    pub access_level: u8, // NOTE - 0 means dev, 1 means admin, 2 means user
    pub status: u8,
    pub role_id: Option<ObjectId>,
    pub side_id: Option<ObjectId>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub last_login_time: Option<i64>,
    pub wallet_address: Option<String>,
    pub balance: Option<u128>,
}


/*
  --------------------------------------------------------------------------------------------------------------------------
| this struct will be used to deserialize user info bson from the mongodb into this struct to serialize as json back to user
| --------------------------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct RegisterResponse{ // NOTE - those Option values can be None tho
    pub _id: Option<ObjectId>, //// this is the user id inside the users collection
    pub username: String,
    pub phone: String,
    pub access_level: u8, // NOTE - 0 means dev, 1 means admin, 2 means user
    pub status: u8,
    pub role_id: Option<ObjectId>,
    pub side_id: Option<ObjectId>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub last_login_time: Option<i64>,
    pub wallet_address: Option<String>,
    pub balance: Option<u128>,
}


/*
  -------------------------------------------------------------------------------------------
| this struct will be used to deserialize OTP request info json from client into this struct
| -------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SendOTPRequest{
    pub phone: String,
}


/*
  --------------------------------------------------------------------------------------
| this struct will be used to serialize OTP info into the json to send back to the user
| --------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SendOTPResponse{
  pub exp_time: i64,
  pub phone: String,
  pub created_at: Option<i64>,
  pub updated_at: Option<i64>,

}


/*
  ------------------------------------------------------------------------------------
| this struct will be used to serialize OTP info into the bson to insert into mongodb
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SaveOTPInfo{
  pub exp_time: i64,
  pub code: String,
  pub phone: String,
  pub created_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server
  pub updated_at: Option<i64>, //// we set this field to Option cause we don't want to pass the created time inside the request body thus it should be None initially, we'll fill it inside the server

}


/*
  -------------------------------------------------------------------------------------------------
| this struct will be used to deserialize check OTP request info json from client into this struct
| -------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CheckOTPRequest{
    pub code: String,
    pub phone: String,
    pub time: i64, //// this is the current time coming from the client to check with the exp_time field inside the otp_info collection
}


/*
  ----------------------------------------------------------------------------------------------------------------------------------------
| this struct will be used to deserialize user info and otp info bson from the mongodb into this struct to serialize as json back to user
| ----------------------------------------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CheckOTPResponse{
    pub user_id: Option<ObjectId>, //// this is the user id inside the users collection
    pub otp_info_id: Option<ObjectId>, //// this is the otp info id inside the otp_info collection
    pub code: String,
    pub phone: String,
    pub last_otp_login_update: Option<i64>, //// this is the updated_at field inside the otp_info collection
}


/*
  ----------------------------------------------------------------------------------------
| this struct will be used to deserialize otp info bson from the mongodb into this struct
| ----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct OTPInfo{
    pub _id: Option<ObjectId>, //// this is the otp info id inside the otp_info collection
    pub exp_time: i64,
    pub code: String,
    pub phone: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}


/*
  ----------------------------------------------------------------------------------------
| this struct will be used to deserialize user info bson from the mongodb into this struct
| ----------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo{ // NOTE - those Option values can be None tho
    pub _id: Option<ObjectId>, //// this is the user id inside the users collection
    pub username: String,
    pub pwd: String,
    pub phone: String,
    pub access_level: u8, // NOTE - 0 means dev, 1 means admin, 2 means user
    pub status: u8,
    pub role_id: Option<ObjectId>,
    pub side_id: Option<ObjectId>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub last_login_time: Option<i64>,
    pub wallet_address: Option<String>,
    pub balance: Option<u128>,
}


/*
  ------------------------------------------------------------------------------------------------
| this struct will be used to deserialize user edit profile info json from client into this struct
| ------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct EditProfile{ // NOTE - those Option values can be None tho
    pub username: String,
    pub phone: String,
}


/*
  ----------------------------------------------------------------------------------------------------
| this struct will be used to put all available users in it and serialize as json to send back to user
| ----------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AvailableUsers{
    pub users: Vec<UserInfo>,
}

/*
  ------------------------------------------------------------------------------------
| this struct will be used to deserialize token info json from client into this struct
| ------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CheckTokenRequest{
    pub access_token: String,
}


/*
  --------------------------------------------------------------------------------------------------------------------------
| this struct will be used to deserialize user info bson from the mongodb into this struct to serialize as json back to user
| --------------------------------------------------------------------------------------------------------------------------
|
|
*/
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CheckTokenResponse{ // NOTE - those Option values can be None tho
    pub _id: Option<ObjectId>, //// this is the user id inside the users collection
    pub username: String,
    pub phone: String,
    pub access_level: u8, // NOTE - 0 means dev, 1 means admin, 2 means user
    pub status: u8, //// last status in an event game that this user has participated in
    pub role_id: Option<ObjectId>, //// last role in an event game that this user has participated in
    pub side_id: Option<ObjectId>, //// last side in an event game that this user has participated in
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub last_login_time: Option<i64>,
    pub wallet_address: Option<String>,
    pub balance: Option<u128>
}

impl RegisterRequest{

    pub async fn hash_pwd(raw_password: String) -> Result<String, argon2::Error>{
        let salt = env::var("SECRET_KEY").expect("⚠️ no secret key variable set");
        let salt_bytes = salt.as_bytes();
        let password_bytes = raw_password.as_bytes();
        argon2::hash_encoded(password_bytes, salt_bytes, &Config::default())
    }

}


impl LoginRequest{

    pub async fn verify_pwd(hashed_password: String, raw_password: String) -> Result<bool, argon2::Error>{
        let password_bytes = raw_password.as_bytes();
        Ok(argon2::verify_encoded(&hashed_password, password_bytes).unwrap())
    }

}