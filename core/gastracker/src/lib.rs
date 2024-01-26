

use actix_web::HttpResponse;
use log::error;
use log::info;
use redis::RedisResult;
use redis::AsyncCommands;
use serde::{Serialize, Deserialize};

pub type PanelHttpResponse = Result<actix_web::HttpResponse, actix_web::Error>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<'m, T>{
    pub data: Option<T>,
    pub message: &'m str, // &str are a slice of String thus they're behind a pointer and every pointer needs a valid lifetime which is 'm in here 
    pub status: u16,
    pub is_error: bool
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

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CurrencyLayerResponseGbp{
    pub success: bool,
    pub terms: String,
    pub privacy: String,
    pub timestamp: i64,
    pub source: String,
    pub quotes: QuoteGbp
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct QuoteGbp{
    /* following need to be uppercase cause the response fields are uppercase */
    pub GBPEUR: f64,
    pub GBPUSD: f64,
    pub GBPIRR: f64,
}

/* >_
    use serde_json::Value codec in case that we don't know the type of data 
    coming from or sending to server or client
*/

pub async fn gwei_to_usd() -> Result<f64, PanelHttpResponse>{
        
    let endpoint = format!("https://api.coinlore.net/api/ticker/?id=33536");
    let res: serde_json::Value = match reqwest::Client::new()
        .get(endpoint.as_str())
        .send()
        .await
        {
            Ok(resp) => {
                resp
                    .json()
                    .await
                    .unwrap()
            },
            Err(e) => {

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: &e.to_string(),
                    status: 406,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                )
            }
        };
        

    let mut gwei_to_usd = 0.0;
    if res.is_array(){
        let matic_info = &res.as_array().unwrap()[0];
        if matic_info["price_usd"].is_string(){
            let matic_price = matic_info["price_usd"].as_str().unwrap();
            // 1 MATIC = 1,000,000,000 gwei
            // 0.731197 in USD = 1 MATIC
            // 0.731197 in USD = 1,000,000,000 gwei
            // 1 gwei = ? USD -> 0.731197 in USD / 1_000_000_000.0f64
            gwei_to_usd = matic_price.parse::<f64>().unwrap() / 1_000_000_000.0f64;
        }
    }
    
    Ok(gwei_to_usd)

}


pub async fn calculate_gas_in_token(redis_client: redis::Client) -> Result<i64, PanelHttpResponse>{

    let blocknative_key = std::env::var("BLOCKNATIVE_TOKEN").unwrap();
    let endpoint = format!("https://api.blocknative.com/gasprices/blockprices?chainid=137");
    let res: serde_json::Value = match reqwest::Client::new()
        .get(endpoint.as_str())
        .header("Authorization", &blocknative_key)
        .send()
        .await
        {
            Ok(resp) => {
                resp
                    .json()
                    .await
                    .unwrap()
            },
            Err(e) => {

                let resp = Response::<&[u8]>{
                    data: Some(&[]),
                    message: &e.to_string(),
                    status: 406,
                    is_error: true
                };
                return Err(
                    Ok(HttpResponse::NotAcceptable().json(resp))
                )
            }
        };

    let default_block_prices: Vec<serde_json::Value> = vec![]; 
    let mut gas_price = 0.0;


    if res["blockPrices"].is_array(){
        let blockprices = res["blockPrices"].as_array().unwrap_or(&default_block_prices);
        for bp in blockprices{
            if bp["baseFeePerGas"].is_f64(){
                gas_price = bp["baseFeePerGas"].as_f64().unwrap_or(0.0);
            }
        }
    }

    let get_gwei_to_usd = gwei_to_usd().await;
    let Ok(gwei_to_usd) = get_gwei_to_usd else{
        let resp_err = get_gwei_to_usd.unwrap_err();
        return Err(resp_err);
    };

    let gas_price_in_usd = gas_price * gwei_to_usd;
    let current_token_value = calculate_token_value(1, redis_client.clone()).await.0;
    let amount_of_token_to_be_burned = gas_price_in_usd / (current_token_value as f64 / 10000000.0);
    let amount_of_token_to_be_burned_i64 = (amount_of_token_to_be_burned * 10000000.0).round() as i64;
    
    info!(" ---> 💸 amount_of_token_to_be_burned {}", amount_of_token_to_be_burned_i64);
    info!(" ---> 💰 current_token_value {}", current_token_value);
    info!(" ---> 💵 gas_price_in_usd {}", gas_price_in_usd);

    Ok(

        // the default onchain fee is 2 which costs approximately $2
        if amount_of_token_to_be_burned_i64 == 0{
            2
        } else{
            amount_of_token_to_be_burned_i64
        }
    )

}


pub async fn calculate_token_value(tokens: i64, redis_client: redis::Client) -> (i64, i64){

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let currencty_layer_secret_key = std::env::var("CURRENCY_LAYER_TOKEN").unwrap();
    let endpoint = format!("http://apilayer.net/api/live?access_key={}&currencies=EUR,GBP,IRR&source=USD&format=1", currencty_layer_secret_key);
    let res = reqwest::Client::new()
        .get(endpoint.as_str())
        .send()
        .await;

    let get_currencies_response = &mut res.unwrap();
    let get_currencies_response_bytes = get_currencies_response.chunk().await.unwrap();
    let err_resp_vec = get_currencies_response_bytes.unwrap().to_vec();
    let get_currencies_response_json = serde_json::from_slice::<CurrencyLayerResponse>(&err_resp_vec);
    
    /* 
        if we're here means that we couldn't map the bytes into the CurrencyLayerResponse 
        and perhaps we have errors in response from the currency layer
    */
    if get_currencies_response_json.is_err(){
        
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let get_currencies_logs_key_err = format!("ERROR=>CurrencyLayerResponse|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(get_currencies_logs_key_err, err_resp_str).await;

        error!("serde decoding currecny layer response error: {}", err_resp_str);

        return (0, 0);

    }
    
    let currencies = get_currencies_response_json.unwrap();

    let value_of_a_token_usd = (1.0 as f64 + currencies.quotes.USDEUR + currencies.quotes.USDGBP) / 3.0 as f64;
    
    let final_value = tokens as f64 * value_of_a_token_usd;
    let scaled_final_value = (final_value * 10000000.0).round(); // scale to keep 7 decimal places
    let final_value_i64: i64 = scaled_final_value as i64;

    let irr_price = scaled_final_value * currencies.quotes.USDIRR;
    let scaled_final_irr_price = (irr_price * 10000000.0).round(); 
    let final_irr_price_i64: i64 = scaled_final_irr_price as i64;


    (final_value_i64, final_irr_price_i64)


}

// ------------------------------------------
// calculate token value to usd based on:
// 1 token = 1 pence or 0.01 pound
// 100 tokens = 1 GBP or 100 pences
// 100 tokens = 1.27 USD
// 250 tokens = 250 * 1.27 / 100
// ------------------------------------------
pub async fn calculate_token_value_gbp_based(tokens: i64, redis_client: redis::Client) -> f64{

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let currencty_layer_secret_key = std::env::var("CURRENCY_LAYER_TOKEN").unwrap();
    let endpoint = format!("http://apilayer.net/api/live?access_key={}&currencies=EUR,USD,IRR&source=GBP&format=1", currencty_layer_secret_key);
    let res = reqwest::Client::new()
        .get(endpoint.as_str())
        .send()
        .await;

    let get_currencies_response = &mut res.unwrap();
    let get_currencies_response_bytes = get_currencies_response.chunk().await.unwrap();
    let err_resp_vec = get_currencies_response_bytes.unwrap().to_vec();
    let get_currencies_response_json = serde_json::from_slice::<CurrencyLayerResponseGbp>(&err_resp_vec);
    
    /* 
        if we're here means that we couldn't map the bytes into the CurrencyLayerResponseGbp 
        and perhaps we have errors in response from the currency layer
    */
    if get_currencies_response_json.is_err(){
        
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let get_currencies_logs_key_err = format!("ERROR=>CurrencyLayerResponseGbp|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(get_currencies_logs_key_err, err_resp_str).await;

        error!("serde decoding currecny layer response error: {}", err_resp_str);

        return 0.0;

    }
    
    let currencies = get_currencies_response_json.unwrap();
    let gbp_to_usd = currencies.quotes.GBPUSD;

    // 1 GBP = 1.27 USD
    // 1 GBP = 100 token
    // 100 token = 1.27 USD
    // 250 = ? usd
    let rounded_value = (gbp_to_usd * 100.0).round() as i64; // scale to keep 2 decimal places
    let final_usd_value = (rounded_value as f64 / 100.0) as f64; // this is the dollar price of 1 GBP
    let tokens_value = (tokens as f64 * final_usd_value) / 100.0;
    tokens_value


}