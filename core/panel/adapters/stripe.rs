


use crate::{*, constants::{APP_NAME, THIRDPARTYAPI_ERROR_CODE}};


/* stripe api adapter to decode incoming u8 bytes from stripe server */

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StripeWebhookPayload{

}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StripeCreateProductResponse{
    pub id: Option<String>,
    pub object: Option<String>,
    pub active: Option<bool>,
    pub created: Option<i64>,
    pub default_price: Option<i64>,
    pub description: Option<String>,
    pub features: Option<Vec<String>>,
    pub images: Option<Vec<String>>,
    pub livemode: Option<bool>,
    pub metadata: Option<HashMap<String, String>>,
    pub name: Option<String>,
    pub package_dimensions: Option<HashMap<String, f64>>,
    pub shippable: Option<bool>,
    pub statement_descriptor: Option<String>,
    pub tax_code: Option<String>,
    pub unit_label: Option<String>,
    pub updated: Option<i64>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StripeCreateCheckoutSessionData{
    pub session_id: String,
    pub session_url: String,
    pub success_url: String,
    pub cancel_url: String,
    pub status: String,
    pub payment_intent: String,
    pub payment_status: String,
    pub expires_at: i64
}

pub async fn create_product(
    redis_client: redis::Client,
    usd_token_price: i64,
    tokens: i64,
    buyer_cid: &str,
) -> StripeCreateProductResponse{
    
    let stripe_test_secret_key = env::var("STRIPE_SECRET_KEY").unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let stripe_token_image = env::var("STRIPE_TOKEN_IMAGE_URL").unwrap();

    /* create product */
    let mut product_data = HashMap::new();
    let product_desc = format!("Buying {} {} Tokens", tokens, APP_NAME);
    let product_name = format!("{} Token", APP_NAME);
    product_data.insert("name".to_string(), product_name);
    product_data.insert("description".to_string(), product_desc);
    product_data.insert("images[0]".to_string(), stripe_token_image);

    let stripe_create_prod_endpoint = format!("https://api.stripe.com/v1/products");
    let res = reqwest::Client::new()
        .post(stripe_create_prod_endpoint.as_str())
        .basic_auth(&stripe_test_secret_key, Some(""))
        .form(&product_data)
        .send()
        .await;

    /* ------------------- STRIPE RESPONSE HANDLING PROCESS -------------------
        since text() and json() method take the ownership of the instance
        thus can't call text() method on ref_resp which is behind a shared ref 
        cause it'll be moved.
        
        let ref_resp = res.as_ref().unwrap();
        let text_resp = ref_resp.text().await.unwrap();

        to solve this issue first we get the stream of the response chunk
        then map it to the related struct, after that we can handle logging
        and redis caching process without losing ownership of things!
    */
    let get_create_prod_response = &mut res.unwrap();
    let get_create_prod_response_bytes = get_create_prod_response.chunk().await.unwrap();
    let err_resp_vec = get_create_prod_response_bytes.unwrap().to_vec();
    let get_create_prod_response_json = serde_json::from_slice::<StripeCreateProductResponse>(&err_resp_vec);
    
    /* 
        if we're here means that we couldn't map the bytes into the StripeCreateProductResponse 
        and perhaps we have errors in response from the stripe service like missing a field
    */
    if get_create_prod_response_json.is_err(){
            
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let create_prod_logs_key_err = format!("ERROR=>StripeCreateProductResponse|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(create_prod_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "stripe_create_prod");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

        return StripeCreateProductResponse::default(); /* return a default product object */

    }

    /* log caching using redis */
    let create_prod_response = get_create_prod_response_json.unwrap(); /* successfully created a product object */
    info!("🪪 USDTokenPrice:{}|{} Tokens:{}|Buyer CID:{}|Log:StripeCreateProductResponse|Time:{}", usd_token_price.clone(), APP_NAME, tokens, buyer_cid, chrono::Local::now().to_string());
    info!("✅ StripeCreateProductResponse: {:#?}", create_prod_response.clone());
    let create_prod_logs_key = format!("USDTokenPrice:{}|{} Tokens:{}|Buyer CID:{}|Log:StripeCreateProductResponse|Time:{}", usd_token_price.clone(), APP_NAME, tokens, buyer_cid, chrono::Local::now().to_string());
    let _: RedisResult<String> = redis_conn.set(create_prod_logs_key, serde_json::to_string_pretty(&create_prod_response).unwrap()).await;

    create_prod_response

}

pub async fn create_price(
    redis_client: redis::Client,
    usd_token_price: i64,
    product_id: &str
) -> String{

    /* create price */
    let stripe_test_secret_key = env::var("STRIPE_SECRET_KEY").unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

    let mut price_data = HashMap::new();
    price_data.insert("unit_amount".to_string(), usd_token_price.to_string());
    price_data.insert("currency".to_string(), "usd".to_string());
    price_data.insert("product".to_string(), product_id.to_string());

    let stripe_create_price_endpoint = format!("https://api.stripe.com/v1/prices");
    let res = reqwest::Client::new()
        .post(stripe_create_price_endpoint.as_str())
        .basic_auth(&stripe_test_secret_key, Some(""))
        .form(&price_data)
        .send()
        .await
        .unwrap();
        

    /* accessing json data dynamically without mapping the response bytes into a struct */
    let res_value = res.json::<serde_json::Value>().await.unwrap();

    info!("✅ StripeCreatePriceResponse: {:#?}", res_value);

    if let Some(price_id) = res_value["id"].as_str(){

        /* log caching using redis */
        info!("🪪 USDTokenPrice:{}|Product Id:{}|Price Id:{}|Log:StripeCreatePriceResponse|Time:{}", usd_token_price.clone(), product_id, price_id, chrono::Local::now().to_string());
        info!("✅ StripeCreatePrice Id: {:#?}", price_id);
        let create_price_logs_key = format!("USDTokenPrice:{}|Product Id:{}|Price Id:{}|Log:StripeCreatePriceResponse|Time:{}", usd_token_price.clone(), product_id, price_id, chrono::Local::now().to_string());
        let _: RedisResult<String> = redis_conn.set(create_price_logs_key, price_id).await;

        price_id.to_string()
        
    } else {
            
        /* 
            can't call the .as_bytes() on this because the res_value.to_string().clone() will be 
            dropped at the end of this statement and we have to create a type with longer lifetime 
            then call .as_bytes() 
        */
        let cloned_err_resp_vec = res_value.to_string().clone(); 
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_bytes()).unwrap();
        let create_price_logs_key_err = format!("ERROR=>StripeCreatePriceResponse|Time:{}", chrono::Local::now().to_string());
        /* log caching using redis */
        let ـ : RedisResult<String> = redis_conn.set(create_price_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, cloned_err_resp_vec.as_bytes().to_vec(), ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "stripe_create_price");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

        return String::from("");

    }

    

}

pub async fn create_session(
    redis_client: redis::Client,
    price_id: &str,
    tokens: i64, // this is the quantity
    user_region: &str,
    user_mail: &str
) -> StripeCreateCheckoutSessionData{

    /* create session */
    let stripe_test_secret_key = env::var("STRIPE_SECRET_KEY").unwrap();
    let stripe_success_url = env::var("STRIPE_PAYMENT_SUCCESS_URL").unwrap();
    let stripe_cancel_url = env::var("STRIPE_PAYMENT_CANCEL_URL").unwrap();
    let stripe_automatic_tax = env::var("STRIPE_AUTOMATIC_TAX").unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    
    /* setup customer mail to enable multi currency feature based on user region */
    // more info: https://stripe.com/docs/payments/checkout/present-local-currencies?platform=automatic-currency-conversion#supported-currencies-and-integrations
    let mut splitted_mail = user_mail.split("@");
    let local_part = splitted_mail.next().unwrap();
    let mail_domain = splitted_mail.next().unwrap();
    let customer_mail = format!("{}+location_{}@{}", local_part, user_region, mail_domain);

    let mut session_data = HashMap::new();
    session_data.insert("mode".to_string(), "payment".to_string());
    session_data.insert("success_url".to_string(), stripe_success_url);
    session_data.insert("cancel_url".to_string(), stripe_cancel_url);
    /* 
        we'd need to flatten line_items and automatic_tax in the form request 
        as per the stripe server's expectations 
    */
    session_data.insert("line_items[0][price]".to_string(), price_id.to_string());
    session_data.insert("line_items[0][quantity]".to_string(), tokens.to_string());
    session_data.insert("automatic_tax[enabled]".to_string(), stripe_automatic_tax);
    session_data.insert("customer_email".to_string(), customer_mail);

    let stripe_create_price_endpoint = format!("https://api.stripe.com/v1/checkout/sessions");
    let res = reqwest::Client::new()
        .post(stripe_create_price_endpoint.as_str())
        .basic_auth(&stripe_test_secret_key, Some(""))
        .form(&session_data)
        .send()
        .await
        .unwrap();

    /* accessing json data dynamically without mapping the response bytes into a struct */
    let res_value = res.json::<serde_json::Value>().await.unwrap();

    info!("✅ StripeCreateCheckoutSessionResponse: {:#?}", res_value);

    /* let's trust the stripte server: if we have session_id then we have url and expires_at */
    if let Some(session_id) = res_value["id"].as_str(){

        let session_url = res_value["url"].as_str().unwrap();
        let succ_url = res_value["success_url"].as_str().unwrap();
        let canc_url = res_value["cancel_url"].as_str().unwrap();
        let expires_at = res_value["expires_at"].as_i64().unwrap();
        let status = res_value["status"].as_str().unwrap();
        let payment_intent = res_value["payment_intent"].as_str().unwrap_or("");
        let payment_status = res_value["payment_status"].as_str().unwrap();

        /* log caching using redis */
        info!("🪪 {} Tokens:{}|Session Id:{}|Session Url:{}|Price Id:{}|Log:StripeCreateCheckoutSession|Time:{}", APP_NAME, tokens, session_id, session_url, price_id, chrono::Local::now().to_string());
        let checkout_session_data = StripeCreateCheckoutSessionData{
            session_id: session_id.to_string(),
            session_url: session_url.to_string(),
            success_url: succ_url.to_string(),
            cancel_url: canc_url.to_string(),
            status: status.to_string(),
            payment_intent: payment_intent.to_string(),
            payment_status: payment_status.to_string(),
            expires_at
        };
        info!("✅ StripeCreateCheckoutSessionData: {:#?}", checkout_session_data);
        let create_session_logs_key = format!("{} Tokens:{}|Session Id:{}|Session Url:{}|Price Id:{}|Log:StripeCreateCheckoutSession|Time:{}", APP_NAME, tokens, session_id, session_url, price_id, chrono::Local::now().to_string());
        let _: RedisResult<String> = redis_conn.set(create_session_logs_key, serde_json::to_string_pretty(&checkout_session_data).unwrap()).await;

        checkout_session_data
        
    } else{

        /* 
            can't call the .as_bytes() on this because the res_value.to_string().clone() will be 
            dropped at the end of this statement and we have to create a type with longer lifetime 
            then call .as_bytes() 
        */
        let cloned_err_resp_vec = res_value.to_string().clone(); 
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_bytes()).unwrap();
        let create_session_logs_key_err = format!("ERROR=>StripeCreatePriceResponse|Time:{}", chrono::Local::now().to_string());
        /* log caching using redis */
        let ـ : RedisResult<String> = redis_conn.set(create_session_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, cloned_err_resp_vec.as_bytes().to_vec(), ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "stripe_create_session");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

        return StripeCreateCheckoutSessionData::default();

    }
    
}