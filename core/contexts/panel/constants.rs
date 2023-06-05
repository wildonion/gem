





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
pub static IAM_HEALTHY: &str = "Ok";
pub static INVALID_PASSPORT_DATA: &str = "Invalid Passport Data";
pub static PASSPORT_DATA_NOT_FOUND: &str = "Passport Data Not Found";
pub static INVALID_TOKEN: &str = "Invalid Token";
pub static NOT_AUTH_HEADER: &str = "No Authorization Header Is Provided";
pub static ACCESS_GRANTED: &str = "Access Granted";
pub static ACCESS_DENIED: &str = "Access Denied";
pub static DO_LOGIN: &str = "Please Login To Generate New JWT";
pub static USER_NOT_FOUND: &str = "User Not Found";
pub static TASK_NOT_FOUND: &str = "Task Not Found";
pub static TASK_VERIFIED: &str = "Task Verified Successfully";
pub static TASK_NOT_VERIFIED: &str = "Task Couldn't Be Verified Successfully (Maybe User Has Been Deleted/Twitter Rate Limit Issue), Deleted Relevant User Task";
pub static USER_TASK_HAS_ALREADY_BEEN_DELETED_BEFORE: &str = "The User Task Has Been Deleted Before";
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