


use crate::*;

pub const APP_NAME: &str = "Conse";
pub type PanelHttpResponse = Result<actix_web::HttpResponse, actix_web::Error>;


pub static DEPOSITED_SUCCESSFULLY: &str = "Deposited Successfully";
pub static DEPOSITED_NOT_FOUND: &str = "Deposited Object Not Found";
pub static ALREADY_WITHDRAWN: &str = "Already Withdrawn";
pub static WITHDRAWN_SUCCESSFULLY: &str = "Withdrawn Successfully";
pub static CANT_MINT_CARD: &str = "Can't Mint The Card, Contact Administrator";
pub static CANT_BURN_CARD: &str = "Card Is Already Burnt";
pub static CANT_DEPOSIT: &str = "Can't Deposit At The Moment, Try Again Later";
pub static CANT_WITHDRAW: &str = "Can't Withdraw At The Moment, Try Again Later";
pub static ID_BUILT: &str = "New Conse Id Built Successfully, Remember To Save Your `singer` Field Value";
pub static RATE_LIMITED: &str = "Rate Limited, Chill 30 Seconds";
pub static CID_RECORD_UPDATED: &str = "Conse Id Record Updated Successfully";
pub static SIGNATURE_ENCODE_ISSUE: &str = "Can't Encode Signature From String";
pub static NO_DEPOSIT_FOR_THIS_RECIPIENT: &str = "No Deposit Found For The Passed In Recipient";
pub static INVALID_CID: &str = "Can't Encode Conse Id From String";
pub static CID_HAS_NO_DEPOSIT_YET: &str = "This CID Has No Any Deposit Yet";
pub static RECIPIENT_HAS_NO_DEPOSIT_YET: &str = "This Recipient Has No Any Deposit Yet";
pub static DEPOSIT_NOT_FOUND: &str = "No Deposit Found With Thid Id";
pub static EMPTY_WITHDRAWAL_ADDRESS: &str = "Can't Withdraw At The Moment, Make Sure You Have A Valid Withdrawal Address";
pub static NOT_VERIFIED_MAIL: &str = "Mail Is Not Verified";
pub static VERIFICATION_CODE_SENT: &str = "Mail Verification Code Has Benn Sent Successfully";
pub static EXPIRED_MAIL_CODE: &str = "Mail Code Has Been Expired";
pub static INVALID_MAIL: &str = "Mail Is Invalid For This User";
pub static INVALID_MAIL_CODE: &str = "Invalid Mail Code";
pub static MAIL_VERIFIED: &str = "User Mail Has Been Verified Successfully";
pub static NO_MAIL_FOR_THIS_USER: &str = "There Is No Verification Process For This Mail";
pub static ALREADY_VERIFIED_MAIL: &str = "Mail Is Already Verified";

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
pub static PUSH_NOTIF_SENT: &str = "Push Notif Has Been Sent With New Roles";
pub static IAM_HEALTHY: &str = "Ok";
pub static INVALID_PASSPORT_DATA: &str = "Invalid Passport Data";
pub static PASSPORT_DATA_NOT_FOUND: &str = "Passport Data Not Found";
pub static INVALID_TOKEN: &str = "Invalid Token Or Mafia Server Is Down";
pub static EVENT_IMG_UPDATED: &str = "Event Image Updated Successfully";
pub static UNSUPPORTED_IMAGE_TYPE: &str = "Image Type Is Not Supported, Only [.png, .jpg or .jpeg]";
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


pub const CHARSET: &[u8] = b"0123456789"; /* converting chars into an slice of their ascii codes which is utf8 bytes */
pub const EVENT_UPLOAD_PATH: &str = "assets/images/events";
pub const LOGS_FOLDER_ERROR_KIND: &str = "logs/error-kind";

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
pub static SERVER_IO_ERROR_CODE: &u16 = &0xFFFE; // is 2 in decimal
pub static STORAGE_IO_ERROR_CODE: &u16 = &0xFFFF; // is 1 in decimal

pub const WS_HEARTBEAT_INTERVAL: StdDuration = StdDuration::from_secs(5);
pub const WS_SUBSCRIPTION_INTERVAL: StdDuration = StdDuration::from_secs(1);
pub const WS_CLIENT_TIMEOUT: StdDuration = StdDuration::from_secs(3600);
pub static WS_INVALID_PATH: &str = "Invalid Path Params";
pub static WS_UPDATE_NOTIF_ROOM_ISSUE: &str = "Can't Update Notif Room";