


use mongodb::bson::oid::ObjectId;
use redis_async::client::PubsubConnection;
use secp256k1::ecdsa::Signature;
use secp256k1::{Secp256k1, All};
use crate::*;
use crate::constants::{CHARSET, APP_NAME};
use crate::events::publishers::role::PlayerRoleInfo;
use crate::models::users::{NewIdRequest, IpInfoResponse, User};
use crate::models::users_deposits::NewUserDepositRequest;
use actix::Addr;



#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct TcpServerData{
    pub data: String,
    pub from_cid: String,
    pub tx_signature: String,
    pub hash_data: String,
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

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct GetTokenValueResponse{
    pub irr: i64,
    pub usd: i64
}

pub async fn start_minting_card_process(
        deposit_object: NewUserDepositRequest, 
        recipient_info: User,
        contract_address: String,
        polygon_recipient_address: String
    ) -> (String, String){

    /* upload card to ipfs */
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();
    let metadata_uri = upload_file_to_ipfs("assets/card.png", &nftport_token).await;
    info!("✅ NftPortUploadFileToIpfsData: {:#?}", metadata_uri.clone());

    if metadata_uri.response == String::from("OK"){

        let metadata_uri = metadata_uri.ipfs_url;

        /* upload metadata to ipfs */
        let mut custom_fields = HashMap::new();
        custom_fields.insert("amount".to_string(), deposit_object.amount.to_string());
        let meta_name = format!("{} gift card with value of {} tokens", APP_NAME, deposit_object.amount);
        let meta_desc = format!("Transferring a {} gift card to {}", APP_NAME, recipient_info.username);
        let upload_data = NftPortUploadMetadataRequest{
            name: meta_name,
            description: meta_desc,
            file_url: metadata_uri,
            custom_fields,
        };

        let nftport_upload_meta_endpoint = format!("https://api.nftport.xyz/v0/metadata");
        let res = reqwest::Client::new()
            .post(nftport_upload_meta_endpoint.as_str())
            .header("Authorization", nftport_token.as_str())
            .json(&upload_data)
            .send()
            .await;

        let upload_meta_response = res.unwrap().json::<NftPortUploadMetadataResponse>().await.unwrap();
        info!("✅ NftPortUploadMetadataRequest: {:#?}", upload_meta_response.clone());

        if upload_meta_response.response == String::from("OK"){

            /* mint request */
            let mint_to_contract_owner = "0xB3E106F72E8CB2f759Be095318F70AD59E96bfC2";
            let mut mint_data = HashMap::new();
            mint_data.insert("chain", "polygon");
            mint_data.insert("contract_address", &contract_address);
            mint_data.insert("metadata_uri", &upload_meta_response.metadata_uri);
            mint_data.insert("mint_to_address", &mint_to_contract_owner);
            let nftport_mint_endpoint = format!("https://api.nftport.xyz/v0/mints/customizable");
            let res = reqwest::Client::new()
                .post(nftport_mint_endpoint.as_str())
                .header("Authorization", nftport_token.as_str())
                .json(&mint_data)
                .send()
                .await;
    
            let mint_response = res.unwrap().json::<NftPortMintResponse>().await.unwrap();
            info!("✅ NftPortMintResponse: {:#?}", mint_response.clone());
            
            if mint_response.response == String::from("OK"){

                let mint_tx_hash = mint_response.transaction_hash;

                /* sleep till the transaction gets confirmed on blockchain */
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                let token_id_string = {
        
                    /* get minted nft info */
                    let nftport_get_nft_endpoint = format!("https://api.nftport.xyz/v0/mints/{}?chain=polygon", mint_tx_hash);
                    let res = reqwest::Client::new()
                        .get(nftport_get_nft_endpoint.as_str())
                        .header("Authorization", nftport_token.as_str())
                        .send()
                        .await;
        
                    let get_nft_response = res.unwrap().json::<NftPortGetNftResponse>().await.unwrap();
                    info!("✅ NftPortGetNftResponse: {:#?}", get_nft_response.clone());
    
                    if get_nft_response.response == String::from("OK"){

                        let token_id = get_nft_response.token_id;
                        let nftport_transfer_nft = format!("https://api.nftport.xyz/v0/mints/transfers");
                        let mut transfer_data = HashMap::new();
                            transfer_data.insert("chain", "polygon");
                            transfer_data.insert("contract_address", &contract_address);
                            transfer_data.insert("token_id", &token_id);
                            transfer_data.insert("transfer_to_address", &polygon_recipient_address);
                        
                        let res = reqwest::Client::new()
                            .post(nftport_transfer_nft.as_str())
                            .header("Authorization", nftport_token.as_str())
                            .json(&transfer_data)
                            .send()
                            .await;

                        let get_nft_transfer_response = NftPortTransferNftResponse::default();
                        info!("✅ NftPortTransferNftResponse: {:#?}", res.unwrap().text().await.unwrap());
                        
                        if get_nft_transfer_response.response == String::from("OK"){
                            token_id
                        } else{
                            String::from("")
                        }

                    } else{
                        String::from("")
                    }
                
                };
                
                if mint_tx_hash.starts_with("0x"){
                    return (mint_tx_hash, token_id_string);
                } else{
                    return (String::from(""), String::from(""));
                }

            } else{

                /* mint wasn't ok */
                return (String::from(""), String::from(""));
            }

        } else{

            /* nftport_upload_meta wasn't ok */
            return (String::from(""), String::from(""));
        }

    } else{

        /* upload in ipfs wasn't ok */
        return (String::from(""), String::from(""));

    }
        

}

pub async fn start_burning_card_process( 
        contract_address: String,
        token_id: String,
    ) -> String{

    /* burn request */
    let transfer_to_contract_owner = "0xB3E106F72E8CB2f759Be095318F70AD59E96bfC2";
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();
    
    /* 
        in order to burn an nft the owner must be the contract itself 
        thus first we'll transfer the minted nft to the contract owner
    */
    let nftport_transfer_nft = format!("https://api.nftport.xyz/v0/mints/transfers");
    let mut transfer_data = HashMap::new();
        transfer_data.insert("chain", "polygon");
        transfer_data.insert("contract_address", &contract_address);
        transfer_data.insert("token_id", &token_id);
        transfer_data.insert("transfer_to_address", &transfer_to_contract_owner);
    
    let res = reqwest::Client::new()
        .post(nftport_transfer_nft.as_str())
        .header("Authorization", nftport_token.as_str())
        .json(&transfer_data)
        .send()
        .await;

    let get_nft_transfer_response = res.unwrap().json::<NftPortTransferNftResponse>().await.unwrap();
    info!("✅ NftPortTransferNftResponse: {:#?}", get_nft_transfer_response.clone());
    
    if get_nft_transfer_response.response == String::from("OK"){
        
        /* nft owner must be the contract owner to burn it */
        let mut burn_data = HashMap::new();
        burn_data.insert("chain", "polygon");
        burn_data.insert("contract_address", &contract_address);
        burn_data.insert("token_id", &token_id);
        let nftport_burn_endpoint = format!("https://api.nftport.xyz/v0/mints/customizable");
        let res = reqwest::Client::new()
            .delete(nftport_burn_endpoint.as_str())
            .header("Authorization", nftport_token.as_str())
            .json(&burn_data)
            .send()
            .await;

        let burn_response = res.unwrap().json::<NftPortBurnResponse>().await.unwrap();
        info!("✅ NftPortBurnResponse: {:#?}", burn_response.clone());
        
        if burn_response.response == String::from("OK"){

            let burn_tx_hash = burn_response.transaction_hash;

            if burn_tx_hash.starts_with("0x"){
                return burn_tx_hash;
            } else{
                return String::from("");
            }

        } else{
            
            return String::from("");

        }

    } else{
        String::from("")
    }


}

pub async fn upload_file_to_ipfs(path: &str, nftport_token: &str) -> NftPortUploadFileToIpfsData{

    let upload_ipf_response = {

        let nftport_host = std::env::var("NFTPORT_HOST").unwrap();
        let nftport_port = std::env::var("NFTPORT_PORT").unwrap();
        let upload_ipfs_endpoint = format!("http://{}:{}/upload/{}", nftport_host, nftport_port, nftport_token);
        let res = reqwest::Client::new()
            .post(upload_ipfs_endpoint.as_str())
            .send()
            .await;

        
        let upload_ipf_response = res.unwrap().json::<NftPortUploadFileToIpfsResponse>().await.unwrap();

        upload_ipf_response.res

    };

    upload_ipf_response

}

pub fn gen_random_chars(size: u32) -> String{
    let mut rng = rand::thread_rng();
    (0..size).map(|_|{
        /* converint the generated random ascii to char */
        char::from_u32(rng.gen_range(33..126)).unwrap() // generating a char from the random output of type u32 using from_u32() method
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

pub async fn get_ip_data(user_ip: String) -> IpInfoResponse{

    /* region detection process based on ip parsnig */
    let mut ipinfo_data = IpInfoResponse::default();
    let (ipinfo_data_sender, mut ipinfo_data_receiver) = 
        tokio::sync::mpsc::channel::<IpInfoResponse>(1024);
    
    tokio::spawn(async move{

        let ipinfo_token = std::env::var("IPINFO_TOKEN").unwrap();
        let get_ip_api = format!("https://ipinfo.io/{}", user_ip);
        let token = format!("Bearer {}", ipinfo_token);
        let get_ip_response = reqwest::Client::new()
            .get(get_ip_api.as_str())
            .header("Authorization", &token)
            .send()
            .await;

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

pub async fn calculate_token_value(tokens: i64) -> (i64, i64){

    let currencty_layer_secret_key = std::env::var("CURRENCY_LAYER_TOKEN").unwrap();
    let endpoint = format!("http://apilayer.net/api/live?access_key={}&currencies=EUR,GBP,IRR&source=USD&format=1", currencty_layer_secret_key);
    let get_currencies = reqwest::Client::new()
        .get(endpoint.as_str())
        .send()
        .await;

    let currencies = get_currencies.unwrap().json::<CurrencyLayerResponse>().await.unwrap();

    let value_of_a_token_usd = (1.0 as f64 + currencies.quotes.USDEUR + currencies.quotes.USDGBP) / 3.0 as f64;
    
    let final_value = tokens as f64 * value_of_a_token_usd;
    let scaled_final_value = (final_value * 1000000.0).round(); // scale to keep 4 decimal places (e.g., 1.2345 becomes 12345)
    let final_value_i64: i64 = scaled_final_value as i64;

    let irr_price = scaled_final_value * currencies.quotes.USDIRR;
    let scaled_final_irr_price = (irr_price * 1000000.0).round(); 
    let final_irr_price_i64: i64 = scaled_final_irr_price as i64;


    (final_value_i64, final_irr_price_i64)


}

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
*/
pub fn string_to_static_str(s: String) -> &'static str { 
    /* 
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
*/
pub fn vector_to_static_slice(s: Vec<u32>) -> &'static [u32] { 
    /* 
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


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Keys{
    pub twitter_bearer_token: String,
    pub twitter_access_token: String,
    pub twitter_access_token_secret: String,
    pub twitter_consumer_key: String,
    pub twitter_consumer_secret: String,
    pub twitter_api_key: String,
    pub twitter_api_secret: String
}


#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct TwitterAccounts{
    pub keys: Vec<Keys>
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortMintResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
    pub metadata_uri: String,
    pub mint_to_address: String
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUploadMetadataRequest{
    pub name: String,
    pub description: String,
    pub file_url: String,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortGetNftResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub token_id: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortTransferNftResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortBurnResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUploadMetadataResponse{
    pub response: String,
    pub metadata_uri: String,
    pub name: String,
    pub description: String,
    pub file_url: String,
    pub external_url: Option<String>,
    pub animation_url: Option<String>,
    pub custom_fields: Option<HashMap<String, String>>,
    pub attributes: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUploadFileToIpfsResponse{
    pub res: NftPortUploadFileToIpfsData
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUploadFileToIpfsData{
    pub response: String,
    pub ipfs_url: String,
    pub file_name: String,
    pub content_type: String,
    pub file_size: i64,
    pub file_size_mb: f64
}

/*  ----------------------
   | shared state storage 
   |----------------------
   | redis
   | redis async
   | redis actor
   | mongodb
   | postgres
   |
*/
pub mod s3{

    pub use super::*;

    #[derive(Clone)] // can't bound Copy trait cause engine and url are String which are heap data structure 
    pub struct Db{
        pub mode: Mode,
        pub engine: Option<String>,
        pub url: Option<String>,
        pub instance: Option<Client>,
        pub pool: Option<Pool<ConnectionManager<PgConnection>>>,
        pub redis: Option<RedisClient>,
        pub redis_async_pubsub_conn: Option<Arc<PubsubConnection>>,
        pub redis_actix_actor: Option<Addr<RedisActor>>,
    }
    
    impl Default for Db{
        fn default() -> Db {
            Db{
                mode: self::Mode::Off,
                engine: None,
                url: None,
                instance: None,
                pool: None, // pg pool
                redis: None,
                redis_async_pubsub_conn: None,
                redis_actix_actor: None,
            }
        }
    }
    
    impl Db{
        
        pub async fn new() -> Result<Db, Box<dyn std::error::Error>>{
            Ok(
                Db{ // building an instance with generic type C which is the type of the db client instance
                    mode: Mode::On, // 1 means is on 
                    engine: None, 
                    url: None,
                    instance: None,
                    pool: None, // pg pool
                    redis: None,
                    redis_async_pubsub_conn: None,
                    redis_actix_actor: None,
                }
            )
        }
        /* 
            don't call a method which has self (not &self) as it's first 
            param since by call it on the instance the instance will be 
            dropped from the ram move borrowed form of the type in most 
            cases unless its pointer is a shared pointer in which we 
            must deref it using * or clone
         
            Client object uses std::sync::Arc internally, so it can safely be 
            shared across threads or async tasks like tokio::spawn(async move{}) 
            green threads also it is highly recommended to create a single 
            Client and persist it for the lifetime of your application.
        */
        pub async fn GetMongoDbInstance(&self) -> Client{ // it'll return an instance of the mongodb client - we set the first argument to &self in order to have the instance of the object later on after calling this method and prevent ownership moving
            Client::with_uri_str(self.url.as_ref().unwrap()).await.unwrap() // building mongodb client instance
        }
    
        pub async fn GetPostgresPool(&self) -> Pool<ConnectionManager<PgConnection>>{
            let uri = self.url.as_ref().unwrap().as_str();
            let manager = ConnectionManager::<PgConnection>::new(uri);
            let pool = Pool::builder().test_on_check_out(true).build(manager).unwrap();
            pool
        }
    
    }
    
    #[derive(Clone)]
    pub struct Storage{
        pub id: Uuid,
        pub db: Option<Db>, // we could have no db at all
    }
    
    impl Storage{
    
        /* 
            since unwrap() takes the ownership of the instance, because 
            it doesn't have &self in its first param, it has self, thus
            we must call as_ref() on the instance before using it to return 
            a reference to the instance to take the ownership of the referenced
            instance by using the unwrap()
        */
        
        pub async fn get_mongodb(&self) -> Option<&Client>{
            match self.db.as_ref().unwrap().mode{
                Mode::On => self.db.as_ref().unwrap().instance.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Client> or Option<&T>
                Mode::Off => None, // no storage is available cause it's off
            }
        }
    
        pub async fn get_pgdb(&self) -> Option<&Pool<ConnectionManager<PgConnection>>>{ // Pool is an structure which takes a generic M which is bounded to ManageConnection trait
            match self.db.as_ref().unwrap().mode{
                Mode::On => self.db.as_ref().unwrap().pool.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<&Pool<ConnectionManager<PgConnection>>> or Option<&T>
                Mode::Off => None, // no storage is available cause it's off
            }
        }
    
        pub async fn get_redis(&self) -> Option<&RedisClient>{ /* an in memory data storage */
            match self.db.as_ref().unwrap().mode{
                Mode::On => self.db.as_ref().unwrap().redis.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
                Mode::Off => None, // no storage is available cause it's off
            }
        }
    
        pub fn get_redis_sync(&self) -> Option<&RedisClient>{ /* an in memory data storage */
            match self.db.as_ref().unwrap().mode{
                Mode::On => self.db.as_ref().unwrap().redis.as_ref(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
                Mode::Off => None, // no storage is available cause it's off
            }
        }
    
        pub async fn get_async_redis_pubsub_conn(&self) -> Option<Arc<PubsubConnection>>{ /* an in memory data storage */
            match self.db.as_ref().unwrap().mode{
                Mode::On => self.db.as_ref().unwrap().redis_async_pubsub_conn.clone(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
                Mode::Off => None, // no storage is available cause it's off
            }
        }
    
        pub fn get_async_redis_pubsub_conn_sync(&self) -> Option<Arc<PubsubConnection>>{ /* an in memory data storage */
            match self.db.as_ref().unwrap().mode{
                Mode::On => self.db.as_ref().unwrap().redis_async_pubsub_conn.clone(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
                Mode::Off => None, // no storage is available cause it's off
            }
        }
    
        pub async fn get_redis_actix_actor(&self) -> Option<Addr<RedisActor>>{ /* an in memory data storage */
            match self.db.as_ref().unwrap().mode{
                Mode::On => self.db.as_ref().unwrap().redis_actix_actor.clone(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
                Mode::Off => None, // no storage is available cause it's off
            }
        }
    
        pub fn get_redis_actix_actor_sync(&self) -> Option<Addr<RedisActor>>{ /* an in memory data storage */
            match self.db.as_ref().unwrap().mode{
                Mode::On => self.db.as_ref().unwrap().redis_actix_actor.clone(), // return the db if it wasn't detached from the server - instance.as_ref() will return the Option<RedisClient> or Option<&T>
                Mode::Off => None, // no storage is available cause it's off
            }
        }
    
    }
    
    #[derive(Copy, Clone, Debug)]
    pub enum Mode{ // enum uses 8 bytes (usize which is 64 bits on 64 bits arch) tag which is a pointer pointing to the current variant - the total size of this enum is 8 bytes tag + the largest variant size = 8 + 0 = 8 bytes; cause in our case On and Off variant both have 0 size
        On, // zero byte size
        Off, // zero byte size
    }
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
}


/*
    we can define as many as response object 
    since once the scope or method or the match
    arm gets executed the lifetime of the response
    object will be dropped from the ram since rust 
    doesn't have gc :) 
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
            use crate::misc::Response;
            
            let code = $code.as_u16();
            let mut res = HttpResponse::build($code);
            
            let response_data = Response::<$data_type>{
                data: Some($data),
                message: $msg,
                status: code
            };
            
            let resp = if let Some(cookie) = $cookie{
                res
                    .cookie(cookie.clone())
                    .append_header(("cookie", cookie.value()))
                    .json(
                        response_data
                    )
            } else{
                res
                    .json(
                        response_data
                    )
            }; 

            return Ok(resp);
        }
    }
}


/* 

    ---------------- MACRO PATTERNS -----------------

    rust types can be fallen into one the following categories

    item      ➔ an Item | an item, like a function, struct, module, etc.
    block     ➔ a BlockExpression | a block (i.e. a block of statements and/or an expression, surrounded by braces)
    stmt      ➔ a Statement without the trailing semicolon (except for item statements that require semicolons)
    pat_param ➔ a PatternNoTopAlt
    pat       ➔ at least any PatternNoTopAlt, and possibly more depending on edition
    expr      ➔ an Expression
    ty        ➔ a Type
    ident     ➔ an IDENTIFIER_OR_KEYWORD or RAW_IDENTIFIER
    path      ➔ a TypePath style path | a path (e.g. foo, ::std::mem::replace, transmute::<_, int>, …)
    tt        ➔ a TokenTree (a single token or tokens in matching delimiters (), [], or {})
    meta      ➔ an Attr, the contents of an attribute | a meta item; the things that go inside #[...] and #![...] attributes
    lifetime  ➔ a LIFETIME_TOKEN
    vis       ➔ a possibly empty Visibility qualifier
    literal   ➔ matches -?LiteralExpression

*/

#[macro_export]
macro_rules! server {
    (
        
        /* ... setup args go here ... */

    ) => {
        
        {

            use std::env;
            use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse, get, ResponseError};
            use actix_web::middleware::Logger;
            use dotenv::dotenv;
            use crate::constants::*;
            use crate::events::subscribers::notifs::role::RoleNotifServer;
            use crate::events::subscribers::notifs::mmr::MmrNotifServer;
            use crate::events::subscribers::notifs::ecq::EcqNotifServer;

            
            env::set_var("RUST_LOG", "trace");
            // env::set_var("RUST_LOG", "actix_web=debug");
            dotenv().expect("⚠️ .env file not found");
            env_logger::init_from_env(Env::default().default_filter_or("info"));
            let host = std::env::var("HOST").expect("⚠️ no host variable set");
            let port = std::env::var("PANEL_PORT").expect("⚠️ no panel port variable set").parse::<u16>().unwrap();
            let db_host = env::var("DB_HOST").expect("⚠️ no db host variable set");
            let db_port = env::var("DB_PORT").expect("⚠️ no db port variable set");
            let db_username = env::var("DB_USERNAME").expect("⚠️ no db username variable set");
            let db_password = env::var("DB_PASSWORD").expect("⚠️ no db password variable set");
            let db_engine = env::var("DB_ENGINE").expect("⚠️ no db engine variable set");
            let db_name = env::var("DB_NAME").expect("⚠️ no db name variable set");

            /* 
                app_sotrage contains the mongodb, postgres and actix_redis, redis 
                and redis_async which can be used to authorize then publish topics,
                response caching and subscribing asyncly to topics respectively
            */
            let app_storage = storage!{ // this publicly has exported inside the misc so we can access it here 
                db_name,
                db_engine,
                db_host,
                db_port,
                db_username,
                db_password
            }.await;

            

            /*  
                                        SETTING UP SHARED STATE DATA
                
                make sure we're starting the RoleNotifServer, MmrNotifServer and EcqNotifServer actor in here 
                and pass the actor isntance to the routers' threadpool otherwise the actor will be started each 
                time by calling the related websocket route
            */
            let role_ntif_server_instance = RoleNotifServer::new(app_storage.clone()).start();
            let shared_ws_role_notif_server = Data::new(role_ntif_server_instance.clone());
            
            let mmr_ntif_server_instance = MmrNotifServer::new(app_storage.clone()).start();
            let shared_ws_mmr_notif_server = Data::new(mmr_ntif_server_instance.clone());

            let ecq_ntif_server_instance = EcqNotifServer::new(app_storage.clone()).start();
            let shared_ws_ecq_notif_server = Data::new(ecq_ntif_server_instance.clone());

            let shared_storage = Data::new(app_storage.clone());

            /*
                the HttpServer::new function takes a factory function that produces an instance of the App, 
                not the App instance itself. This is because each worker thread needs to have 
                its own App instance.

                handle streaming async tasks like socket connections in a none blocking manner asyncly and 
                concurrently using tokio::spawn(async move{}) and shared state data between tokio::spawn() 
                green threadpool using jobq channels and clusters using redis and routers' threads using arc, 
                mutex and rwlock also data must be Send + Sync + 'static also handle incoming async events 
                into the server using tokio::select!{} eventloop. 

                we're sharing the db_instance and redis connection state between routers' threads to get the 
                data inside each api also for this the db and redis connection data must be shareable and safe 
                to send between threads which must be bounded to Send + Sync traits 

                since every api or router is an async task that must be handled inside the hyper threads thus 
                the data that we want to use inside of them and share it between other routers must be 
                Arc<Mutex<Data>> + Send + Sync + 'static 

                mongodb and redis connection instances must be only Arc (shareable) to share them between threads 
                since we don't want to mutate them in actix routers' threads. 
            */
            info!("➔ 🚀 {} panel server has launched from [{}:{}] at {}", APP_NAME, host, port, chrono::Local::now().naive_local());
            let s = match HttpServer::new(move ||{
                App::new()
                    /* 
                        SHARED STATE DATA
                    */
                    .app_data(Data::clone(&shared_storage.clone()))
                    .app_data(Data::clone(&shared_ws_role_notif_server.clone()))
                    .app_data(Data::clone(&shared_ws_mmr_notif_server.clone()))
                    .app_data(Data::clone(&shared_ws_ecq_notif_server.clone()))
                    .wrap(Cors::permissive())
                    .wrap(Logger::default())
                    .wrap(Logger::new("%a %{User-Agent}i %t %P %r %s %b %T %D"))
                    /*
                        INIT WS SERVICE
                    */
                    .service(
                        actix_web::web::scope("/subscribe")
                            .configure(services::init_ws_notif)
                    )
                    /*
                        INIT DEV SERIVE APIs 
                    */
                    .service(
                        actix_web::web::scope("/dev")
                            .configure(services::init_dev)   
                    )
                    /*
                        INIT ADMIN SERIVE APIs
                    */
                    .service(
                        actix_web::web::scope("/admin")
                            .configure(services::init_admin)
                    )
                    /*
                        INIT USER SERIVE APIs 
                    */
                    .service(
                        actix_web::web::scope("/user")
                            .configure(services::init_user)
                    )
                    /*
                        INIT HEALTH SERIVE
                    */
                    .service(
                        actix_web::web::scope("/health")
                            .configure(services::init_health)
                    )
                    /*
                        INIT BOT SERIVE
                    */
                    .service(
                        actix_web::web::scope("/public")
                            .configure(services::init_public)
                    )
                    /*
                        INIT SWAGGER UI SERIVES
                    */
                    .service(SwaggerUi::new("/swagger/{_:.*}").urls(vec![
                        (
                            Url::new("admin", "/api-docs/admin.json"),
                            apis::admin::AdminApiDoc::openapi(),
                        ),
                        (
                            Url::new("dev", "/api-docs/dev.json"),
                            apis::dev::DevApiDoc::openapi(),
                        ),
                        (
                            Url::new("user", "/api-docs/user.json"),
                            apis::user::UserApiDoc::openapi(),
                        ),
                        (
                            Url::new("health", "/api-docs/health.json"),
                            apis::health::HealthApiDoc::openapi(),
                        ),
                        (
                            Url::new("public", "/api-docs/public.json"),
                            apis::public::PublicApiDoc::openapi(),
                        )
                    ]))
                }) // each thread of the HttpServer instance needs its own app factory 
                .bind((host.as_str(), port)){
                    Ok(server) => {
                        server
                            /* 
                                running server in a threadpool with 10 spawned threads to handle 
                                incoming connections asyncly and concurrently 
                            */
                            .workers(10) 
                            .run()
                            .await
                    },
                    Err(e) => {
        
                        /* custom error handler */
                        use error::{ErrorKind, ServerError::{ActixWeb, Ws}, PanelError};
                         
                        let error_content = &e.to_string();
                        let error_content = error_content.as_bytes().to_vec();
        
                        let error_instance = PanelError::new(*SERVER_IO_ERROR_CODE, error_content, ErrorKind::Server(ActixWeb(e)), "HttpServer::new().bind");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
        
                        panic!("panicked at running actix web server at {}", chrono::Local::now());
                        
        
                    }
                };


            /* 
                this can't be reachable unless we hit the ctrl + c since the 
                http server will be built inside multiple threads in which all 
                server instances will be ran constanly in the background, and 
                must be the last thing that can be reachable before sending Ok(())
                from the main function, it's like the app will be halted in this
                section of the code cause anything after those threads rquires 
                that all the threads to be stopped and joined in order to execute 
                the logic after running the http server.
            */
            // info!("➔ 🎛️ starting conse panel on address: [{}:{}]", host, port);
            
            s /* returning the server */

        }
    };
}

#[macro_export]
macro_rules! storage {

    ($name:expr, $engine:expr, $host:expr, $port:expr, $username:expr, $password:expr) => {
                
        async { // this is the key! this curly braces is required to use if let statement, use libs and define let inside macro
            
            use crate::misc::*;
            use crate::misc::s3::*;

            /* -=-=-=-=-=-=-=-=-=-=-= REDIS SETUP -=-=-=-=-=-=-=-=-=-=-= */

            let redis_password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());
            let redis_username = env::var("REDIS_USERNAME").unwrap_or("".to_string());
            let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
            let redis_port = std::env::var("REDIS_PORT").unwrap_or("6379".to_string()).parse::<u64>().unwrap();
            let redis_actor_conn_url = format!("{redis_host}:{redis_port}");

            let redis_conn_url = if !redis_password.is_empty(){
                format!("redis://:{}@{}:{}", redis_password, redis_host, redis_port)
            } else if !redis_password.is_empty() && !redis_username.is_empty(){
                format!("redis://{}:{}@{}:{}", redis_username, redis_password, redis_host, redis_port)
            } else{
                format!("redis://{}:{}", redis_host, redis_port)
            };

            let none_async_redis_client = redis::Client::open(redis_conn_url.as_str()).unwrap();
            let redis_actor = RedisActor::start(redis_actor_conn_url.as_str());
            let mut redis_conn_builder = ConnectionBuilder::new(redis_host, redis_port as u16).unwrap();
            redis_conn_builder.password(redis_password);
            let async_redis_pubsub_conn = Arc::new(redis_conn_builder.pubsub_connect().await.unwrap());
            
            /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */

            
            let empty_app_storage = Some( // putting the Arc-ed db inside the Option
                Arc::new( // cloning app_storage to move it between threads
                    Storage{ // defining db context 
                        id: Uuid::new_v4(),
                        db: Some(
                            Db{
                                mode: Mode::Off,
                                instance: None,
                                engine: None,
                                url: None,
                                pool: None, // pg pool
                                redis: None,
                                redis_async_pubsub_conn: None,
                                redis_actix_actor: None
                            }
                        ),
                    }
                )
            );
            let app_storage = if $engine.as_str() == "mongodb"{
                info!("➔ 🛢️ switching to mongodb on address: [{}:{}]", $host, $port);
                let environment = env::var("ENVIRONMENT").expect("⚠️ no environment variable set");
                let db_addr = if environment == "dev"{
                    format!("{}://{}:{}", $engine, $host, $port)
                } else if environment == "prod"{
                    format!("{}://{}:{}@{}:{}", $engine, $username, $password, $host, $port)
                } else{
                    "".to_string()
                };
                match Db::new().await{
                    Ok(mut init_db) => { // init_db instance must be mutable since we want to mutate its fields
                        init_db.engine = Some($engine);
                        init_db.url = Some(db_addr);
                        let mongodb_instance = init_db.GetMongoDbInstance().await; // the first argument of this method must be &self in order to have the init_db instance after calling this method, cause self as the first argument will move the instance after calling the related method and we don't have access to any field like init_db.url any more due to moved value error - we must always use & (like &self and &mut self) to borrotw the ownership instead of moving
                        Some( // putting the Arc-ed db inside the Option
                            Arc::new( // cloning app_storage to move it between threads
                                Storage{ // defining db context 
                                    id: Uuid::new_v4(),
                                    db: Some(
                                        Db{
                                            mode: init_db.mode,
                                            instance: Some(mongodb_instance),
                                            engine: init_db.engine,
                                            url: init_db.url,
                                            pool: None, // pg pool
                                            redis: Some(none_async_redis_client.clone()),
                                            redis_async_pubsub_conn: Some(async_redis_pubsub_conn.clone()),
                                            redis_actix_actor: Some(redis_actor.clone())
                                        }
                                    ),
                                }
                            )
                        )
                    },
                    Err(e) => {
                        error!("😕 init db error - {}", e);
                        empty_app_storage // whatever the error is we have to return and empty app storage instance 
                    }
                }
            } else if $engine.as_str() == "postgres"{
                info!("➔ 🛢️ switching to postgres on address: [{}:{}]", $host, $port);
                let environment = env::var("ENVIRONMENT").expect("⚠️ no environment variable set");                
                let db_addr = if environment == "dev"{
                    format!("{}://{}:{}", $engine, $host, $port)
                } else if environment == "prod"{
                    format!("{}://{}:{}@{}:{}/{}", $engine, $username, $password, $host, $port, $name)
                } else{
                    "".to_string()
                };
                match Db::new().await{
                    Ok(mut init_db) => { // init_db instance must be mutable since we want to mutate its fields
                        init_db.engine = Some($engine);
                        init_db.url = Some(db_addr);
                        let pg_pool = init_db.GetPostgresPool().await; // the first argument of this method must be &self in order to have the init_db instance after calling this method, cause self as the first argument will move the instance after calling the related method and we don't have access to any field like init_db.url any more due to moved value error - we must always use & (like &self and &mut self) to borrotw the ownership instead of moving
                        Some( // putting the Arc-ed db inside the Option
                            Arc::new( // cloning app_storage to move it between threads
                                Storage{ // defining db context 
                                    id: Uuid::new_v4(),
                                    db: Some(
                                        Db{
                                            mode: init_db.mode,
                                            instance: None,
                                            engine: init_db.engine,
                                            url: init_db.url,
                                            pool: Some(pg_pool),
                                            redis: Some(none_async_redis_client.clone()),
                                            redis_async_pubsub_conn: Some(async_redis_pubsub_conn.clone()),
                                            redis_actix_actor: Some(redis_actor.clone())
                                        }
                                    ),
                                }
                            )
                        )
                    },
                    Err(e) => {
                        error!("😕 init db error - {}", e);
                        empty_app_storage // whatever the error is we have to return and empty app storage instance 
                    }
                }
            } else{
                empty_app_storage
            };

            app_storage // returning the created app_storage

        }
    };

}

#[macro_export]
macro_rules! mafia_passport {
    (
      $token:expr /* this is the generated token from the conse mafia hyper server */
    ) 
    => {

        { // this is required if we want to import modules and use the let statements
            
            use std::env;

            let host = env::var("HOST").expect("⚠️ no host variable set");
            let port = env::var("MAFIA_PORT").expect("⚠️ no port variable set");
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

#[macro_export]
macro_rules! verify {
    (
      $endpoint:expr,
      $body:expr,
      $task_id:expr,
      $doer_id:expr,
      $connection:expr,
      $redis_client:expr,
      $task_type:expr,
      $tusername:expr,
      $tweet_link:expr
    ) 
    => {

        { // this is required if we want to import modules and use the let statements

            use crate::models::bot::Twitter;
            use crate::misc::Response;

            info!("🤖 sending request to the twitter bot hosted on [{:#?}]", $endpoint);
            let response_value: serde_json::Value = reqwest::Client::new()
                .post($endpoint)
                .json(&$body)
                .send()
                .await.unwrap()
                .json()
                .await.unwrap();

            /* I believe that the bot code has some shity response structure :) since I didn't designed it*/

            let data_field = response_value.get("data");
            if data_field.is_some(){
                let status = data_field.unwrap().get("status");
                if status.is_some(){

                    let bool_status = status.unwrap().to_string();
                    if bool_status == "false"{

                        /* twitter error */

                        match diesel::delete(users_tasks
                            .filter(users_tasks::task_id.eq($task_id)))
                            .filter(users_tasks::user_id.eq($doer_id))
                            .execute($connection)
                            {
                                Ok(num_deleted) => {
                                    
                                    if num_deleted > 0{
            
                                        let resp = Response::<&[u8]>{
                                            data: Some(&[]),
                                            message: TASK_NOT_VERIFIED,
                                            status: 406
                                        };
                                        return Ok(
                                            HttpResponse::NotAcceptable().json(resp)
                                        );                                
            
                                    } else{
                                        
                                        let resp = Response::<&[u8]>{
                                            data: Some(&[]),
                                            message: USER_TASK_HAS_ALREADY_BEEN_DELETED,
                                            status: 417
                                        };
                                        return Ok(
                                            HttpResponse::ExpectationFailed().json(resp)
                                        ); 
            
                                    }
                                
                                },
                                Err(e) => {
            
                                    let resp = Response::<&[u8]>{
                                        data: Some(&[]),
                                        message: &e.to_string(),
                                        status: 500
                                    };
                                    return Ok(
                                        HttpResponse::InternalServerError().json(resp)
                                    );
            
                                }
                            }

                    } else{

                        /* task is verified by twitter */

                        match UserTask::find($doer_id, $task_id, $connection).await{
                            false => {

                                /* try to insert into users_tasks since it's done */
                                let res = Twitter::do_task($doer_id, $task_id, $tusername, $task_type, $tweet_link, $connection).await;
                                return res;
                            },
                            _ => {
        
                                /* user task has already been inserted  */
                                let resp = Response::<&[u8]>{
                                    data: Some(&[]),
                                    message: USER_TASK_HAS_ALREADY_BEEN_INSERTED,
                                    status: 302
                                };
                                return Ok(
                                    HttpResponse::Found().json(resp)
                                );
        
                            }
                        }

                    }
                } else{

                    /* twitter rate limit issue */

                    let resp = Response::<&[u8]>{
                        data: Some(&[]),
                        message: TWITTER_RATE_LIMIT,
                        status: 406
                    };
                    return Ok(
                        HttpResponse::NotAcceptable().json(resp)
                    );  
                
                }
            } else{

                /* twitter rate limit issue */

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: TWITTER_RATE_LIMIT,
                    status: 406
                };
                return Ok(
                    HttpResponse::NotAcceptable().json(resp)
                );  
            }
        }
    }
}

/* ------------------ 
  |    DSL MACROS
  |------------------
  |

*/

#[macro_export]
macro_rules! o_O {
    (
        $(
            $x:expr; [ $( $y:expr ), * ]
        ); * /* multiple of this pattern */
    ) => {
        &[ $($( $x + $y ), *), * ]
    }
}
//////
/// let a: &[i32] = o_O![10; [1, 2, 3]; 20; [4, 5, 6]];
//////

#[macro_export]
macro_rules! list {
    ($id1:ident | $id2:ident <- [$start:expr; $end:expr], $cond:expr) => { //// the match pattern can be any syntax :) - only ident can be followed by some symbols and words like <-, |, @ and etc
        { //.... code block to return vec since if we want to use let statements we must be inside {} block
            let mut vec = Vec::new();
            for num in $start..$end + 1{
                if $cond(num){
                    vec.push(num);
                }
            }
            vec
        } //....
    };
}
//////
/// let even = |x: i32| x%2 == 0;
/// let odd = |x: i32| x%2 != 0;
/// let evens = list![x | x <- [1; 10], even];
//////

#[macro_export]
macro_rules! dict {
    ($($key:expr => $val:expr)*) => { //// if this pattern matches the input the following code will be executed - * means we can pass more than one key => value statement
        { //.... code block to return vec since if we want to use let statements we must be inside {} block
            use std::collections::HashMap;
            let mut map = HashMap::new();
            $(
                map.insert($key, $value);
            )* //// * means we're inserting multiple key => value statement inside the map 
            map
        } //....
    };
}
//////
/// let d = dict!{"wildonion" => 1, "another_wildonion" => 2, "array": vec![1,3,4235,], "age": 24};
//////

#[macro_export]
macro_rules! exam {
    ($l:expr; and $r:expr) => { //// logical and match 
        $crate::macros::even(); //// calling even() function which is inside the macros module
        println!("{}", $l && $r);
    };

    ($l:expr; or $r:expr) => { //// logical or match 
        println!("{}", $l || $r);
    };
}
//////
/// exam!(1 == 2; and 3 == 2+1)
/// exam!(1 == 2; or 3 == 2+1)
//////


#[macro_export]
macro_rules! cmd {
    ($iden:ident, $ty: tt) => {
        pub struct $iden(pub $ty);
        impl Default for $iden{
            fn default() -> Self{
                todo!()
            }
        }  
    };

    ($func_name:ident) => {
        fn $func_name(){
            println!("you've just called {:?}()", stringify!($func_name));
        }
    }
}
//////
/// cmd!{bindgen, id} //// bindgen is the name of the struct and id is the name of the field
//////


#[macro_export]
macro_rules! query { // NOTE - this is a macro with multiple syntax support and if any pattern matches with the caller pattern, then the code block of that pattern will be emitted
    
    ( $value_0:expr, $value_1:expr, $value_2:expr ) => { //// passing multiple object syntax
        // ...
    };

    ( $($name:expr => $value:expr)* ) => { //// passing multiple key => value syntax 
        // ...

    };

}


#[macro_export]
macro_rules! log {
    ($arg:tt) => { //// passing single String message 
        $crate::env::log($arg.as_bytes()) //// log function only accepts utf8 bytes
    };
    ($($arg:tt)*) => { //// passing multiple String messages 
        $crate::env::log(format!($($arg)*).as_bytes()) //// log function only accepts utf8 bytes
    };
}


#[macro_export]
macro_rules! impl_ecq_engine_constructor {
    ($( $new:ident: [ $( $pos:expr ),* ] anchored at $anchor:expr; )*) => { //// the match pattern can be any syntax :) - only ident can be followed by some symbols and words like <-, |, @ and etc 
        $(
            pub fn $new() -> Self{
                Self{
                    positions: [$( $pos ),*].into_iter().collect(),
                    anchor: $anchor,
                }
            }
        )* //// * means defining function for every new Pos
    };
}


#[macro_export]
macro_rules! iterator{
    ($ty:ty, $ident:ident; $($state_ident:ident: $state_ty:ty),*; $next:expr) => (
        struct $ident {
            $($state_ident: $state_ty), *
        }

        impl Iterator for $ident {
            type Item = $ty;

            fn next(&mut self) -> Option<$ty> {
                $next(self)
            }
        }
    );
}
//////
// iterator!(i32, TestIterator; index: i32; |me: &mut TestIterator| {
//     let value = Some(me.index);
//     me.index += 1;
//     value
// });
//////


macro_rules! pat {
    ($i:ident) => (Some($i))
}

// if let pat!(x) = Some(1) {
//     assert_eq!(x, 1);
// }

macro_rules! Tuple {
    { $A:ty, $B:ty } => { ($A, $B) };
}

type N2 = Tuple!(i32, i32);

macro_rules! const_maker {
    ($t:ty, $v:tt) => { const CONST: $t = $v; };
}
trait T {
    const_maker!{i32, 7}
}

macro_rules! example {
    () => { println!("Macro call in a macro!"); };
}


// #[derive(Debug, Clone)]
// pub struct Shape{
//     typ: &'static str,
//     positions: HashSet<Pos>,
//     anchor: Pos,
// }


// #[derive(Debug, Clone, Copy)]
// pub struct Pos(pub i32, pub i32);



// impl Shape {
//     impl_ecq_engine_constructor! {
//       new_i "🟦": [Pos(0, 0), Pos(1, 0), Pos(2, 0), Pos(3, 0)] @ Pos(1, 0);
//       new_o "🟨": [Pos(0, 0), Pos(1, 0), Pos(0, 1), Pos(1, 1)] @ Pos(0, 0);
//       new_t "🟫": [Pos(0, 0), Pos(1, 0), Pos(2, 0), Pos(1, 1)] @ Pos(1, 0);
//       new_j "🟪": [Pos(0, 0), Pos(0, 1), Pos(0, 2), Pos(-1, 2)] @ Pos(0, 1);
//       new_l "🟧": [Pos(0, 0), Pos(0, 1), Pos(0, 2), Pos(1, 2)] @ Pos(0, 1);
//       new_s "🟩": [Pos(0, 0), Pos(1, 0), Pos(0, 1), Pos(-1, 1)] @ Pos(0, 0);
//       new_z "🟥": [Pos(0, 0), Pos(-1, 0), Pos(0, 1), Pos(1, 1)] @ Pos(0, 0);
//     }
// }
    
    
 #[macro_export]
macro_rules! contract {

    /*

        contract!{

            NftContract, //// name of the contract
            "wildonion.near", //// the contract owner
            /////////////////////
            //// contract fields
            /////////////////////
            [
                contract_owner: AccountId, 
                deposit_by_owner: HashMap<AccountId, near_sdk::json_types::U128>, 
                contract_balance: near_sdk::json_types::U128
            ]; //// fields
            /////////////////////
            //// contract methods
            /////////////////////
            [ 
                "init" => [ //// array of init methods
                    pub fn init_contract(){
            
                    }
                ],
                "private" => [ //// array of private methods
                    pub fn get_all_deposits(){

                    }
                ],
                "payable" => [ //// array of payable methods
                    pub fn deposit(){
            
                    }
                ],
                "external" => [ //// array of external methods
                    fn get_address_bytes(){

                    }
                ]
            ]

        }

    */

    // event!{
    //     name: "list_owner",
    //     log: [NewOwner, AddDeposit],

    //     // event methods

    //     fn add_owner(){

    //     } 

    //     fn add_deposit(){
            
    //     }
    // }

    // emit!{
    //     event_name
    // }

    (
     $name:ident, $signer:expr, //// ident can be used to pass struct
     [$($fields:ident: $type:ty),*]; 
     [$($method_type:expr => [$($method:item),*]),* ]
    ) 
     
     => {
            #[near_bindgen]
            #[derive(serde::Deserialize, serde::Serialize)]
            pub struct $name{
                $($fields: $type),*
            }

            impl $name{
                        
                // https://stackoverflow.com/questions/64790850/how-do-i-write-a-macro-that-returns-the-implemented-method-of-a-struct-based-on
                // TODO - implement methods here 
                // ...
            }
    }
}