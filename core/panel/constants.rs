


use crate::*;

pub const APP_NAME: &str = "Conse";
pub const DEV_ACCESS: u8 = 0;
pub const ADMIN_ACCESS: u8 = 1; // God access
pub static BUSY_BOT: &str = "Bot Is Busy";
pub static FETCHED: &str = "Fetched Successfully";
pub static LOGGEDIN: &str = "Loggedin Successfully";
pub static REGISTERED: &str = "Registered Successfully";
pub static LOGOUT: &str = "Loggedout Successfully";
pub static CREATED: &str = "Created Successfully";
pub static DELETED: &str = "Deleted Successfully";
pub static UPDATED: &str = "Updated Successfully";
pub static FOUND_TASK: &str = "Task Has Already Been Registered";
pub static STORAGE_ISSUE: &str = "Storage Is Not Available";
pub static PUSH_NOTIF_ACTIVATED: &str = "Push Notif Has Been Activated";
pub static IAM_HEALTHY: &str = "Ok";
pub static INVALID_PASSPORT_DATA: &str = "Invalid Passport Data";
pub static PASSPORT_DATA_NOT_FOUND: &str = "Passport Data Not Found";
pub static INVALID_TOKEN: &str = "Invalid Token";
pub static NOT_AUTH_HEADER: &str = "No Authorization Header Is Provided";
pub static ACCESS_GRANTED: &str = "Access Granted";
pub static ACCESS_DENIED: &str = "Access Denied";
pub static DO_LOGIN: &str = "Invalid Token Time, Please Login To Generate New JWT";
pub static USER_NOT_FOUND: &str = "User Not Found";
pub static USERNAME_CANT_BE_EMPTY: &str = "Username Can't Be Empty";
pub static WALLET_CANT_BE_EMPTY: &str = "Wallet Can't Be Empty";
pub static TASK_NOT_FOUND: &str = "Task Not Found";
pub static TASK_CREATED: &str = "Task Created For The Passed In User";
pub static TASK_VERIFIED: &str = "Task Verified Successfully";
pub static TASK_NOT_VERIFIED: &str = "Deleted Relevant User Task Since The Task Couldn't Be Verified Successfully By Twitter";
pub static TASK_DELETED_WITH_NO_DOER: &str = "Task Deleted Successfully With No Doer";
pub static TASK_DELETED_WITH_DOER: &str = "Task Deleted Successfully With At Least One Doer";
pub static USER_TASK_HAS_ALREADY_BEEN_DELETED: &str = "The User Task Has Already Been Deleted Due To Verification Issue";
pub static USER_TASK_HAS_ALREADY_BEEN_INSERTED: &str = "The User Task Has Already Been Inserted";
pub static NOT_A_TWITTER_TASK: &str = "Not A Twitter Tasks";
pub static INVALID_TWITTER_TASK_NAME: &str = "Invalid Twitter Task Type";
pub static WRONG_PASSWORD: &str = "Wrong Password";
pub static NOT_FOUND_COOKIE_EXP: &str = "No Expiration Time Found In Cookie";
pub static EXPIRED_COOKIE: &str = "Cookie Has Been Expired";
pub static CANT_GENERATE_COOKIE: &str = "Can't Generate Cookie";
pub static NOT_FOUND_COOKIE_VALUE_OR_JWT: &str = "No Value Found In Cookie Or JWT In Header";
pub static NOT_FOUND_TOKEN: &str = "JWT Not Found In Cookie";
pub static INVALID_COOKIE_FORMAT: &str = "Invalid Cookie Format";
pub static INVALID_COOKIE_TIME_HASH: &str = "Invalid Cookie Time Hash";
pub static NOT_FOUND_COOKIE_TIME_HASH: &str = "No Time Hash Found In Cookie";
pub static EMPTY_USERS_TASKS: &str = "No User Tasks Are Available";
pub static COMPLETE_VERIFICATION_PROCESS: &str = "Users Tasks Verification Completed Successfully";
pub const CHARSET: &[u8] = b"0123456789";

pub static TWITTER_RATE_LIMIT: &str = "Reached Twitter Rate Limit";
pub static TWITTER_USER_IS_NOT_VALID: &str = "Twitter Username Is Not Valid";
pub static TWITTER_CODE_IS_NOT_VALID: &str = "Twitter Code Task Is Not Done";
pub static TWITTER_VERIFIED_CODE: &str = "Twitter Code Task Is Done";
pub static TWITTER_VERIFIED_USERNAME: &str = "Twitter Username Is Verified";
pub static TWITTER_USER_DATA_NOT_FOUND: &str = "Twitter User Data Followers Not Found";
pub static TWITTER_USER_FOLLOWERS_NOT_FOUND: &str = "Twitter User Followers Not Found";
pub static TWITTER_USER_TWEETS_NOT_FOUND: &str = "Twitter User Tweets Not Found";
pub static TWITTER_VERIFIED_TWEET: &str = "Twitter Tweet Content Task Is Done";
pub static TWITTER_NOT_VERIFIED_TWEET_CONTENT: &str = "Twitter Tweet Content Task Is Not Done";
pub static TWITTER_VERIFIED_LIKE: &str = "Twitter Like Task Is Done";
pub static TWITTER_NOT_VERIFIED_LIKE: &str = "Twitter Like Task Is Not Done";
pub static TWITTER_TWEET_NOT_FOUND: &str = "Twitter Tweet Not Found For The Given Id";
pub static TWITTER_VERIFIED_RETWEET: &str = "Twitter Retweet Task Is Done";
pub static TWITTER_NOT_VERIFIED_RETWEET: &str = "Twitter Retweet Task Is Not Done";
pub static TWITTER_VERIFIED_HASHTAG: &str = "Twitter Hashtag Task Is Done";
pub static TWITTER_NOT_VERIFIED_HASHTAG: &str = "Twitter Hashtag Task Is Not Done";
pub static TWITTER_CANT_LOOP_OVER_ACCOUNTS: &str = "Can't Loop Over Twitter Accounts";
pub static TWITTER_KEYS_ADDED: &str = "Twitter Keys Added Successfully";
pub static TWITTER_VERIFICATION_RATE_LIMIT: &str = "Entering Chillzone";

/* u16 bits is 2 bytes long which is 4 chars in hex */
pub static SERVER_IO_ERROR_CODE: &u16 = &0xFFFE; // 2
pub static STORAGE_IO_ERROR_CODE: &u16 = &0xFFFF; // 1

pub const WS_HEARTBEAT_INTERVAL: StdDuration = StdDuration::from_secs(5);
pub const WS_SUBSCRIPTION_INTERVAL: StdDuration = StdDuration::from_secs(1);
pub const WS_CLIENT_TIMEOUT: StdDuration = StdDuration::from_secs(3600);
pub static WS_INVALID_PATH: &str = "Invalid Path Params";
pub static WS_UPDATE_NOTIF_ROOM_ISSUE: &str = "Can't Update Notif Room";
pub static WS_SUBSCRIPTION_ISSUE: &str = "Can't Send Subscribe Message To Redis Actor";
pub static WS_SUBSCRIPTION_TIMEOUT: &str = "Subscription Timeout";
pub static WS_SUBSCRIPTION_INTERVAL_ISSUE: &str = "Can't Start Subscription Interval";
pub static WS_INVALID_SUBSCRIPTION_TYPE: &str = "Invalid Subscription Type";
pub const WS_REDIS_SUBSCIPTION_INTERVAL: StdDuration = StdDuration::from_secs(5);
pub const WS_REDIS_SUBSCIPTION_INTERVAL_NUMBER: u64 = 5;