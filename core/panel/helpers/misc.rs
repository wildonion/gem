


use std::io::Write;
use std::time::{UNIX_EPOCH, SystemTime};
use futures_util::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use redis_async::client::PubsubConnection;
use serenity::client::Context;
use crate::*;
use crate::constants::{CHARSET, APP_NAME, THIRDPARTYAPI_ERROR_CODE, TWITTER_24HOURS_LIMITED, NOT_VERIFIED_PHONE, USER_SCREEN_CID_NOT_FOUND, INVALID_SIGNATURE, NOT_VERIFIED_MAIL, INSUFFICIENT_FUNDS, UNSUPPORTED_FILE_TYPE, TOO_LARGE_FILE_SIZE};
use crate::events::publishers::role::PlayerRoleInfo;
use crate::models::users::{NewIdRequest, IpInfoResponse, User};
use crate::models::users_deposits::NewUserDepositRequest;
use crate::models::users_nfts::CreateNftMetadataUriRequest;
use crate::models::users_tasks::UserTask;
use actix::Addr;




/* ------------------------------------------------------------------------- */
/* --------------------------- HELPER STRUCTURES --------------------------- */
/* ------------------------------------------------------------------------- */
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TotalXRlInfo{
    pub app_rate_limit_info: Vec<XAppRlInfo>,
    pub x_15mins_interval_reqs: HashMap<u64, u64>
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct XRlInfo{
    pub bot: Option<String>,
    pub x_rate_limit_remaining: Option<String>,
    pub x_rate_limit_limit: Option<String>,
    pub x_rate_limit_reset: Option<String>,
    pub x_app_limit_24hour_limit: Option<String>,
    pub x_app_limit_24hour_reset: Option<String>,
    pub x_app_limit_24hour_remaining: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct XAppRlInfo{
    pub bot: Option<String>,
    pub x_app_limit_24hour_reset: Option<String>,
    pub x_app_limit_24hour_remaining: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct XUserRateLimitInfo{
    pub username: String,
    pub route: String,
    pub rl_info: XRlInfo,
    pub request_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct XBotRateLimitInfo{
    pub bot: String,
    pub rl_info: XRlInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keys{
    pub twitter_bearer_token: String,
    pub twitter_access_token: String,
    pub twitter_access_token_secret: String,
    pub twitter_consumer_key: String,
    pub twitter_consumer_secret: String,
    pub twitter_api_key: String,
    pub twitter_api_secret: String
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TwitterAccounts{
    pub keys: Vec<Keys>
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddGroupInfoToEvent{
    pub _id: String, // ObjectId is the bson type of _id inside the mongodb
    pub name: String,
    pub owner: String, // this is the id of the user took from the mongodb and will be stored as String later we'll serialize it into bson mongodb ObjectId
    pub image_path: Option<String>,
    pub god_id: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Voter{
    pub user_id: String,
    pub username: String,
    pub nft_owner_wallet_address: String,
    pub is_upvote: bool,
    pub score: u32, // NOTE - this is the number of event NFTs that this owner owns
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Phase{
    pub day: Vec<InsertPlayerInfoRequest>, // vector of all user infos at the end of the day that their status has changed
    pub mid_day: Vec<InsertPlayerInfoRequest>, // vector of all user infos at the end of the mid day that their status has changed
    pub night: Vec<InsertPlayerInfoRequest>, // vector of all user infos at the end of the night that their status has changed
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct InsertPlayerInfoRequest{
  pub user_id: String, // ObjectId is the bson type of _id inside the mongodb
  pub username: String,
  pub status: u8,
  pub role_name: String,
  pub role_id: String,
  pub side_id: String,
  pub chain_history: Vec<ChainInfo>,
  pub role_ability_history: Vec<RoleAbilityInfo>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ChainInfo{
  pub to_id: String,
  pub chained_at: i64,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct RoleAbilityInfo{
  pub role_id: String,
  pub current_ability: Option<u8>,
  pub updated_at: Option<i64>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct EventInfo{
    pub _id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub deck_id: String,
    pub entry_price: String,
    pub group_info: Option<AddGroupInfoToEvent>,
    pub image_path: Option<String>,
    pub creator_wallet_address: Option<String>,
    pub upvotes: Option<u16>,
    pub downvotes: Option<u16>,
    pub voters: Option<Vec<Voter>>,
    pub phases: Option<Vec<Phase>>,
    pub max_players: Option<u8>,
    pub players: Option<Vec<PlayerRoleInfo>>,
    pub is_expired: Option<bool>,
    pub is_locked: Option<bool>,
    pub started_at: Option<i64>,
    pub expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct PlayerEventInfo{
    pub _id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub deck_id: String,
    pub entry_price: String,
    pub group_info: Option<AddGroupInfoToEvent>,
    pub image_path: Option<String>,
    pub creator_wallet_address: Option<String>,
    pub upvotes: Option<u16>,
    pub downvotes: Option<u16>,
    pub voters: Option<Vec<Voter>>,
    pub phases: Option<Vec<Phase>>,
    pub max_players: Option<u8>,
    pub is_expired: Option<bool>,
    pub is_locked: Option<bool>,
    pub started_at: Option<i64>,
    pub expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CurrencyLayerResponse{
    pub success: bool,
    pub terms: String,
    pub privacy: String,
    pub timestamp: i64,
    pub source: String,
    pub quotes: Quote
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Quote{
    /* following need to be uppercase cause the response fields are uppercase */
    pub USDEUR: f64,
    pub USDGBP: f64,
    pub USDIRR: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetTokenValueResponse{
    pub irr: i64,
    pub usd: i64
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GithubCommitWebhookEventRequest{

}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Limit{
    pub from: Option<i64>,
    pub to: Option<i64>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct R1Keys{
    pub r1pubkey: String,
    pub r1signature: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Search{
    pub q: String,
    pub from: Option<i64>, // can be not passed 
    pub to: Option<i64> // can be not passed 
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnlimitedSearch{
    pub q: String 
}

/*
    can't bound the T to ?Sized since 
    T is inside the Option which the size
    of the Option depends on the T at 
    compile time hence the T must be 
    Sized, also we're using a lifetime 
    to use the str slices in message

    in the case of passing &[] data we must 
    specify the type of T and pass the type to 
    the Response signature like Response::<&[Type]>{} 
    since the size of &[] can't be known 
    at compile time hence we must specify the exact
    type of T inside &[]    

*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Response<'m, T>{
    pub data: Option<T>,
    pub message: &'m str, // &str are a slice of String thus they're behind a pointer and every pointer needs a valid lifetime which is 'm in here 
    pub status: u16,
    pub is_error: bool
}

/* ----------------------------------------------------------------------- */
/* --------------------------- HELPER METHODS ---------------------------- */
/* ----------------------------------------------------------------------- */
pub async fn is_bot_24hours_limited(
    connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
    rl_data: Vec<XAppRlInfo>
) -> Result<(), PanelHttpResponse>{

    /* ---------------------------------------------------------------------- */
    /* checking rate limit data to see if we should reject the request or not */
    /* ---------------------------------------------------------------------- */
    let get_done_tasks = UserTask::all(connection).await;
    let Ok(done_tasks) = get_done_tasks else{
        let error_resp = get_done_tasks.unwrap_err();
        return Err(error_resp);
    };

    
    /* bots has always the latest app rate limit infos fetched from twitter bot server */
    let bots = rl_data;
    info!("🤖 bots info {:#?}", bots);
    

    let first_bot = &bots[0];
    let second_bot = &bots[1];

    
    /*  
        since we have two bots in there once the first one gets rate limited the second one 
        takes place and in here we should check the last one params.

        note that with this logic we have to know the exact number of bots configured in
        twitter bot server to handle this process cause all bots except the last one are 
        rate limited last bot is not rate limited yet.

        we don't need to check the rate limit for the first bot since the twitter bot server
        will switch to the second bot once the first one gets rate limted
    */
    if first_bot.bot.is_some() && second_bot.bot.is_some(){
        let last_bot = bots.clone().into_iter().last().unwrap();
        /*  
            since "".to_string() will be dropped at the end of unwrap_or() statement
            thus it's a temp variable and taking a reference or having &"".to_string() 
            to that is not allowed due to dangling pointer issue, and we should define 
            a separate type contains the String::from("") and take a pointer to that type
            in order to have a longer lifetime. 
        */
        let empty_init = String::from("");
        let last_bot_x_app_limit_24hour_remaining = last_bot.x_app_limit_24hour_remaining.as_ref().unwrap_or(&empty_init);
        let last_bot_x_app_limit_24hour_reset = last_bot.x_app_limit_24hour_reset.as_ref().unwrap_or(&empty_init);
        /*  
            some routes may have null x_app_limit_24hour_remaining so we don't care 
            about them since they have user rate limit count
        */
        if !last_bot_x_app_limit_24hour_remaining.is_empty() && !last_bot_x_app_limit_24hour_reset.is_empty(){
            
            let reset_at = last_bot.x_app_limit_24hour_reset.as_ref().unwrap();
            info!("🤖 bot{} -> x_app_limit_24hour_remaining: {} will be reset at: {}", bots.len(), last_bot_x_app_limit_24hour_remaining, last_bot_x_app_limit_24hour_reset);
            
            if last_bot_x_app_limit_24hour_remaining == &"2".to_string() && 
                /* 
                    also the current time must be smaller than than the reset time of 
                    the current limitation window 
                */
                reset_at.parse::<i64>().unwrap() > chrono::Local::now().timestamp(){

                    let timestamp_milli = last_bot_x_app_limit_24hour_remaining.parse::<i64>().unwrap() * 1000 as i64;
                    let datetime = chrono::NaiveDateTime::from_timestamp_millis(timestamp_milli).unwrap().to_string();
                    let reset_at = format!("{}, Bot{} Reset At {}", TWITTER_24HOURS_LIMITED, bots.len(), datetime);
                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: &reset_at,
                        status: 406,
                        is_error: true,
                    };
                    return Err(
                        Ok(HttpResponse::NotAcceptable().json(resp))
                    );
        
            } else{
                
                Ok(())
                
            }
        } else{
            Ok(())
        }
    } else{
        Ok(())
    }

}

pub async fn fetch_x_app_rl_data(redis_client: redis::Client) -> TotalXRlInfo{

    let bot_endpoint = env::var("XBOT_ENDPOINT").expect("⚠️ no twitter bot endpoint key variable set");
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

    /* ----------------------------------------------------------------------- */
    /* ------------------ get x_15mins_interval_request data ------------------ */
    /* ----------------------------------------------------------------------- */
    let redis_result_x_15mins_interval_request: RedisResult<String> = redis_conn.get("x_15mins_interval_request").await;
    let redis_x_15mins_interval = match redis_result_x_15mins_interval_request{
        Ok(data) => {
            let rl_data = serde_json::from_str::<HashMap<u64, u64>>(data.as_str()).unwrap();
            rl_data
        },
        Err(e) => {
            let empty_x_15mins_interval = HashMap::<u64, u64>::new();
            let rl_data = serde_json::to_string(&empty_x_15mins_interval).unwrap();
            let _: () = redis_conn.set("x_15mins_interval", rl_data).await.unwrap();
            HashMap::new()
        }
    }; 
    /* ----------------------------------------------------------------------- */
    /* ----------------------------------------------------------------------- */


    /* ------------------------------------------------------------------ */
    /* ------------------ get redis_x_app_rl_info data ------------------ */
    /* ------------------------------------------------------------------ */

    /* check that we have reached the rate limit or not */
    let get_redis_x_app_rl_info: RedisResult<String> = redis_conn.get("redis_x_app_rl_info").await;
    let mut use_redis_data = true;
    let redis_data = match get_redis_x_app_rl_info{
        Ok(redis_x_app_rl_info) => serde_json::from_str::<Vec<XAppRlInfo>>(&redis_x_app_rl_info).unwrap(),
        Err(e) => {
            /* 
                if there isn't a key with redis_x_app_rl_info name or if this is 
                the first try to fetch the data from redis we simply return a default
                vector with the XAppRlInfo data but we don't want to use this data
                for rate limit checking we want the actual data coming from the bot 
                server so we used a flag in here.
            */
            use_redis_data = false;
            vec![XAppRlInfo::default()]
        }
    };

    let bot_endpoint = env::var("XBOT_ENDPOINT").expect("⚠️ no twitter bot endpoint key variable set");
    let get_rl_info_route = format!("http://{}/get-app-ratelimit-info", bot_endpoint);
    let res = reqwest::Client::new()
        .get(get_rl_info_route)
        .send()
        .await;


    /* ------------------- TWITTER BOT RESPONSE HANDLING PROCESS -------------------
        since text() and json() method take the ownership of the instance
        thus can't call text() method on ref_resp which is behind a shared ref 
        cause it'll be moved.
        
        let ref_resp = res.as_ref().unwrap();
        let text_resp = ref_resp.text().await.unwrap();

        to solve this issue first we get the stream of the response chunk
        then map it to the related struct, after that we can handle logging
        and redis caching process without losing ownership of things!
    */
    let get_xrl_response = &mut res.unwrap();
    let get_xrl_response_bytes = get_xrl_response.chunk().await.unwrap();
    let err_resp_vec = get_xrl_response_bytes.unwrap().to_vec();
    let get_xrl_response_json = serde_json::from_slice::<Vec<XAppRlInfo>>(&err_resp_vec);
    /* 
        if we're here means that we couldn't map the bytes into the Vec<XAppRlInfo> 
        and perhaps we have errors in response from the twitter bot service
    */
    if get_xrl_response_json.is_err(){
            
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let get_nft_logs_key_err = format!("ERROR=>XAppRlInfo|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(get_nft_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use helpers::error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "fetch_x_app_rl_data");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

        return TotalXRlInfo::default();

    }

    let rl_info = get_xrl_response_json.unwrap();
    info!("🚧 rl info coming from bot server {:?}", rl_info);

    /* redis caching */
    let final_data = if !use_redis_data || (redis_data != rl_info){ /* check that the redis and bot rl data vectors are equal or not, don't check their length cause they have always same length */
        let _: () = redis_conn.set("redis_x_app_rl_info", serde_json::to_string(&rl_info).unwrap()).await.unwrap();
        rl_info
    } else{
        redis_data
    };


    /* ------------------------------------------------------------- */
    /* ------------------------------------------------------------- */

    TotalXRlInfo{ // response data
        app_rate_limit_info: final_data,
        x_15mins_interval_reqs: redis_x_15mins_interval, 
    }
    
}

pub async fn get_ip_data(user_ip: String) -> IpInfoResponse{

    /* region detection process based on ip parsnig */
    let mut ipinfo_data = IpInfoResponse::default();
    let (ipinfo_data_sender, mut ipinfo_data_receiver) = 
        tokio::sync::mpsc::channel::<IpInfoResponse>(1024);
    
    /* 
        getting the ip info in the background using tokio::spawn() and receive the 
        result using mpsc jobq channel
    */
    tokio::spawn(async move{

        let ipinfo_token = std::env::var("IPINFO_TOKEN").unwrap();
        let get_ip_api = format!("https://ipinfo.io/{}", user_ip);
        let token = format!("Bearer {}", ipinfo_token);
        let get_ip_response = reqwest::Client::new()
            .get(get_ip_api.as_str())
            .header("Authorization", &token)
            .send()
            .await;

        /* 
            getting the text of the response takes the ownership thus we can't have the text
            and json of the response at the same time 
        */
        // let response = get_ip_response.unwrap();
        // let response = response.text().await.unwrap();

        let ip_response_data = get_ip_response
            .unwrap()
            .json::<IpInfoResponse>()
            .await
            .unwrap();

        ipinfo_data_sender.send(ip_response_data).await;

    });

    while let Some(channel_ipinfo_data) = ipinfo_data_receiver.recv().await{
        /*
            
            since u_country is of type String thus an slice of it must be valid 
            as long as the actual type is valid or u_country is valid and once the 
            u_country gets dropped any slice of it or reference to it can't live long
            enough and will be dropped too otherwise leads us to dangling pointer, 
            thus we should return the u_country itself from this loop and convert it
            to &str later.

            let u_country = ipinfo_data.country.to_lowercase();

            // u_country_str is defined outside of the loop!
            u_country_str = u_country.as_str();
        
        */

        ipinfo_data = channel_ipinfo_data;

    }

    ipinfo_data

}

pub fn is_password_valid(s: &str) -> bool {
    // all the followings must be true eventually
    let mut has_whitespace = false;
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special_char = false;

    // the bitwise OR assignment ( |= ) operator performs bitwise OR 
    // on the two operands and assigns the result to the left operand
    // false | true = true, false | false = false
    for c in s.chars() {
        has_whitespace |= c.is_whitespace();
        has_lower |= c.is_lowercase();
        has_upper |= c.is_uppercase();
        has_digit |= c.is_digit(10);
        has_special_char |= c.is_ascii_punctuation();
    }

    // if one of the criteria becomes false the return value will be false
    // since there are && between them 
    !has_whitespace && has_special_char && has_upper && has_lower && has_digit && s.len() >= 8
}

pub fn gen_random_chars(size: u32) -> String{
    let mut rng = rand::thread_rng();
    (0..size).map(|_|{
        /* converting the generated random ascii to char */
        char::from_u32(rng.gen_range(33..126)).unwrap() // generating a char from the random output of type u32 using from_u32() method
    }).collect()
}

pub fn gen_random_chars_0_255(size: u32) -> String{
    let mut rng = rand::thread_rng();
    (0..size).map(|_|{
        /* converting the generated random ascii to char */
        char::from_u32(rng.gen_range(0..255)).unwrap() // generating a char from the random output of type u32 using from_u32() method
    }).collect()
}

pub fn gen_random_number(from: u32, to: u32) -> u32{
    let mut rng = rand::thread_rng(); // we can't share this between threads and across .awaits
    rng.gen_range(from..to)
} 

pub fn gen_random_idx(idx: usize) -> usize{
    if idx < CHARSET.len(){
        idx
    } else{
        gen_random_idx(random::<u8>() as usize)
    }
}

pub fn vector_to_static_slice(s: Vec<u8>) -> &'static [u8] { 
    /*  
        we cannot obtain &'static str from a Vec because Vecs may not live 
        for the entire life of our program, and that's what &'static lifetime means. 
        we can only get a slice parameterized by Vec own lifetime from it, we can 
        obtain a static str but it involves leaking the memory of the Vec. this is 
        not something we should do lightly, by leaking the memory of the Vec, this 
        guarantees that the memory will never be freed (thus the leak), therefore, any 
        references to the inner object can be interpreted as having the 'static lifetime.
        
        also here it's ok to return the reference from function since our reference lifetime 
        is static and is valid for the entire life of the app
        
        leaking the memory of the heap data Vec which allows us to have an 
        unfreed allocation that can be used to define static str using it since
        static means we have static lifetime during the whole lifetime of the app
        and reaching this using Vec is not possible because heap data types 
        will be dropped from the heap once their lifetime destroyed in a scope
        like by moving them into another scope hence they can't be live longer 
        than static lifetime

        Note: this will leak memory! the memory for the Vec will not be freed 
        for the remainder of the program. Use this sparingly
    */
    Box::leak(s.into_boxed_slice()) 
}

pub fn string_to_static_str(s: String) -> &'static str { 
    /* 
        we cannot obtain &'static str from a String because Strings may not live 
        for the entire life of our program, and that's what &'static lifetime means. 
        we can only get a slice parameterized by String own lifetime from it, we can 
        obtain a static str but it involves leaking the memory of the String. this is 
        not something we should do lightly, by leaking the memory of the String, this 
        guarantees that the memory will never be freed (thus the leak), therefore, any 
        references to the inner object can be interpreted as having the 'static lifetime.
        
        also here it's ok to return the reference from function since our reference lifetime 
        is static and is valid for the entire life of the app

        leaking the memory of the heap data String which allows us to have an 
        unfreed allocation that can be used to define static str using it since
        static means we have static lifetime during the whole lifetime of the app
        and reaching this using String is not possible because heap data types 
        will be dropped from the heap once their lifetime destroyed in a scope
        like by moving them into another scope hence they can't be live longer 
        than static lifetime

        Note: this will leak memory! the memory for the String will not be freed 
        for the remainder of the program. Use this sparingly
    */
    Box::leak(s.into_boxed_str()) 

}

// -----====-----====-----====-----====-----====-----====-----====
// resp object macro, the most important section in the code 
// the following facitilate sending data back to the client by 
// building a respone object every time the server wants to
// send data back to the client, the macro however gets called
// from where the server is creating data to send it, to inject
// headers and cookies the logics must goes here.
// -----====-----====-----====-----====-----====-----====-----====
/*
    we can define as many as response object since once the scope
    or method or the match arm gets executed the lifetime of the 
    response object will be dropped from the ram due to the fact 
    that rust doesn't have gc :) 
*/
#[macro_export]
macro_rules! resp {
    (   
        $data_type:ty,
        $data:expr,
        $msg:expr,
        $code:expr,
        $cookie:expr,
    ) => {

        {
            use actix_web::HttpResponse;
            use crate::helpers::misc::Response;
            use actix_web::http::header::Expires;
            use std::time::{SystemTime, Duration};
            
            let code = $code.as_u16();
            let mut res = HttpResponse::build($code);

            let response_data = Response::<$data_type>{
                data: Some($data),
                message: $msg,
                status: code,
                is_error: if code == 200 || code == 201 || code == 302{
                    false
                } else{
                    true
                }
            };
            
            // response expiration in client, the Expire gives the date/time after 
            // which the response is considered stale.
            let expiration = SystemTime::now() + Duration::from_secs(60 * 60 * 24); 
            let resp = if let Some(cookie) = $cookie{
                res
                    .cookie(cookie.clone())
                    .append_header(("cookie", cookie.value()))
                    .insert_header(Expires(expiration.into()))
                    .json(
                        response_data
                    )
            } else{
                res
                    .insert_header(Expires(expiration.into()))
                    .json(
                        response_data
                    )
            }; 

            return Ok(resp);
        }
    }
}

#[macro_export]
macro_rules! rendezvous_passport {
    (
      $token:expr /* this is the generated token from the conse rendezvous hyper server */
    ) 
    => {

        { // this is required if we want to import modules and use the let statements
            
            use std::env;

            let host = env::var("HOST").expect("⚠️ no host variable set");
            let port = env::var("RENDEZVOUS_PORT").expect("⚠️ no port variable set");
            let check_token_api = format!("http://{}:{}/auth/check-token", host, port);
            
            let get_response_value = reqwest::Client::new()
                .post(check_token_api.as_str())
                .header("Authorization", $token)
                .send()
                .await;

            match get_response_value{

                Ok(response_value) => {

                    let mut response_value = response_value.json::<serde_json::Value>().await.unwrap();

                    let msg = response_value["message"].take();
                    if msg == serde_json::json!("Access Granted"){
                        true
                    } else{
                        false
                    }
                },
                Err(_) => false

            }

            
            
        }
    }
}

#[macro_export]
macro_rules! is_rate_limited {
    (
        $redis_conn:expr,
        $identifier_key:expr,
        $identifier_type:ty,
        $redis_key:expr
    ) => {
        
        {
            /* rate limiter based on username */
            let chill_zone_duration = 30_000u64; //// 30 seconds chillzone
            let now = chrono::Local::now().timestamp_millis() as u64;
            let mut is_rate_limited = false;
            
            let redis_result_id_rate_limiter: RedisResult<String> = $redis_conn.get($redis_key).await;
            let mut redis_id_rate_limiter = match redis_result_id_rate_limiter{
                Ok(data) => {
                    let rl_data = serde_json::from_str::<HashMap<$identifier_type, u64>>(data.as_str()).unwrap();
                    rl_data
                },
                Err(e) => {
                    let empty_id_rate_limiter = HashMap::<$identifier_type, u64>::new();
                    let rl_data = serde_json::to_string(&empty_id_rate_limiter).unwrap();
                    let _: () = $redis_conn.set($redis_key, rl_data).await.unwrap();
                    HashMap::new()
                }
            };

            if let Some(last_used) = redis_id_rate_limiter.get(&($identifier_key)){
                if now - *last_used < chill_zone_duration{
                    is_rate_limited = true;
                }
            }
            
            if is_rate_limited{
            
                true

            } else{

                /* updating the last rquest time */
                redis_id_rate_limiter.insert($identifier_key, now); //// updating the redis rate limiter map
                let rl_data = serde_json::to_string(&redis_id_rate_limiter).unwrap();
                let _: () = $redis_conn.set($redis_key, rl_data).await.unwrap(); //// writing to redis ram

                false
            }

        }
    }
}