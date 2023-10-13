


use crate::*;
use mongodb::bson::oid::ObjectId;
use redis_async::client::PubsubConnection;
use secp256k1::ecdsa::Signature;
use secp256k1::{Secp256k1, All};
use crate::*;
use crate::constants::{CHARSET, APP_NAME, THIRDPARTYAPI_ERROR_CODE, TWITTER_24HOURS_LIMITED};
use crate::events::publishers::role::PlayerRoleInfo;
use crate::models::users::{NewIdRequest, IpInfoResponse, User};
use crate::models::users_deposits::NewUserDepositRequest;
use crate::models::users_tasks::UserTask;
use actix::Addr;
use models::users_contracts::*;


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
pub struct NftPortUploadFileToIpfsData{
    pub response: String,
    pub ipfs_url: String,
    pub file_name: String,
    pub content_type: String,
    pub file_size: i64,
    pub file_size_mb: f64
}


pub async fn start_minting_card_process(
    sender_screen_cid: String,
    deposit_object: NewUserDepositRequest, 
    recipient_info: User,
    contract_address: String,
    contract_owner: String,
    polygon_recipient_address: String,
    redis_client: redis::Client
) -> (String, String, u8){

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();

    /* upload card to ipfs */
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();
    let (metadata_uri, res_metadata_uri_status) = upload_file_to_ipfs(&nftport_token, redis_client.clone()).await;
        
    if res_metadata_uri_status == 1{
        return (String::from(""), String::from(""), 1);
    }

    /* log caching using redis */
    let upload_logs_key = format!("Sender:{}|Log:NftPortUploadFileToIpfsData|Time:{}", sender_screen_cid.clone(), chrono::Local::now().to_string());
    let ـ : RedisResult<String> = redis_conn.set(upload_logs_key, serde_json::to_string_pretty(&metadata_uri).unwrap()).await;
    info!("✅ NftPortUploadFileToIpfsData: {:#?}", metadata_uri.clone());

    if metadata_uri.response == String::from("OK"){

        let metadata_uri = metadata_uri.ipfs_url;

        /* upload metadata to ipfs */
        let mut custom_fields = HashMap::new();
        custom_fields.insert("amount".to_string(), deposit_object.amount.to_string());
        custom_fields.insert("sender".to_string(), sender_screen_cid.clone());
        custom_fields.insert("recipient".to_string(), polygon_recipient_address.clone());
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

        /* ------------------- NFTPORT RESPONSE HANDLING PROCESS -------------------
            since text() and json() method take the ownership of the instance
            thus can't call text() method on ref_resp which is behind a shared ref 
            cause it'll be moved.
            
            let ref_resp = res.as_ref().unwrap();
            let text_resp = ref_resp.text().await.unwrap();

            to solve this issue first we get the stream of the response chunk
            then map it to the related struct, after that we can handle logging
            and redis caching process without losing ownership of things!
        */
        let get_upload_meta_response = &mut res.unwrap();
        let get_upload_meta_response_bytes = get_upload_meta_response.chunk().await.unwrap();
        let err_resp_vec = get_upload_meta_response_bytes.unwrap().to_vec();
        let get_upload_meta_response_json = serde_json::from_slice::<NftPortUploadMetadataResponse>(&err_resp_vec);
        /* 
            if we're here means that we couldn't map the bytes into the NftPortUploadMetadataResponse 
            and perhaps we have errors in response from the nftport service
        */
        if get_upload_meta_response_json.is_err(){
                
            /* log caching using redis */
            let cloned_err_resp_vec = err_resp_vec.clone();
            let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
            let upload_mata_logs_key_err = format!("ERROR=>NftPortUploadMetadataResponse|Time:{}", chrono::Local::now().to_string());
            let ـ : RedisResult<String> = redis_conn.set(upload_mata_logs_key_err, err_resp_str).await;

            /* custom error handler */
            use error::{ErrorKind, ThirdPartyApiError, PanelError};
            let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "start_minting_card_process");
            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

            return (String::from(""), String::from(""), 1);

        }

        /* log caching using redis */
        let upload_meta_response = get_upload_meta_response_json.unwrap();
        info!("✅ NftPortUploadMetadataResponse: {:#?}", upload_meta_response.clone());
        let upload_mata_logs_key = format!("Sender:{}|Log:NftPortUploadMetadataResponse|Time:{}", sender_screen_cid.clone(), chrono::Local::now().to_string());
        let _: RedisResult<String> = redis_conn.set(upload_mata_logs_key, serde_json::to_string_pretty(&upload_meta_response).unwrap()).await;


        if upload_meta_response.response == String::from("OK"){

            /* mint request */
            let mut mint_data = HashMap::new();
            mint_data.insert("chain", "polygon");
            mint_data.insert("contract_address", &contract_address);
            mint_data.insert("metadata_uri", &upload_meta_response.metadata_uri);
            mint_data.insert("mint_to_address", &contract_owner);
            let nftport_mint_endpoint = format!("https://api.nftport.xyz/v0/mints/customizable");
            let res = reqwest::Client::new()
                .post(nftport_mint_endpoint.as_str())
                .header("Authorization", nftport_token.as_str())
                .json(&mint_data)
                .send()
                .await;


            /* ------------------- NFTPORT RESPONSE HANDLING PROCESS -------------------
                since text() and json() method take the ownership of the instance
                thus can't call text() method on ref_resp which is behind a shared ref 
                cause it'll be moved.
                
                let ref_resp = res.as_ref().unwrap();
                let text_resp = ref_resp.text().await.unwrap();

                to solve this issue first we get the stream of the response chunk
                then map it to the related struct, after that we can handle logging
                and redis caching process without losing ownership of things!
            */
            let get_mint_response = &mut res.unwrap();
            let get_mint_response_bytes = get_mint_response.chunk().await.unwrap();
            let err_resp_vec = get_mint_response_bytes.unwrap().to_vec();
            let get_mint_response_json = serde_json::from_slice::<NftPortMintResponse>(&err_resp_vec);
            /* 
                if we're here means that we couldn't map the bytes into the NftPortMintResponse 
                and perhaps we have errors in response from the nftport service
            */
            if get_mint_response_json.is_err(){
                    
                /* log caching using redis */
                let cloned_err_resp_vec = err_resp_vec.clone();
                let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
                let mint_logs_key_err = format!("ERROR=>NftPortMintResponse|Time:{}", chrono::Local::now().to_string());
                let ـ : RedisResult<String> = redis_conn.set(mint_logs_key_err, err_resp_str).await;

                /* custom error handler */
                use error::{ErrorKind, ThirdPartyApiError, PanelError};
                let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "start_minting_card_process");
                let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                return (String::from(""), String::from(""), 1);

            }

            /* log caching using redis */
            let mint_response = get_mint_response_json.unwrap();
            info!("✅ NftPortMintResponse: {:#?}", mint_response.clone());
            let mint_logs_key = format!("Sender:{}|Log:NftPortMintResponse|Time:{}", sender_screen_cid.clone(), chrono::Local::now().to_string());
            let _: RedisResult<String> = redis_conn.set(mint_logs_key, serde_json::to_string_pretty(&mint_response).unwrap()).await;
            

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


                    /* ------------------- NFTPORT RESPONSE HANDLING PROCESS -------------------
                        since text() and json() method take the ownership of the instance
                        thus can't call text() method on ref_resp which is behind a shared ref 
                        cause it'll be moved.
                        
                        let ref_resp = res.as_ref().unwrap();
                        let text_resp = ref_resp.text().await.unwrap();

                        to solve this issue first we get the stream of the response chunk
                        then map it to the related struct, after that we can handle logging
                        and redis caching process without losing ownership of things!
                    */
                    let get_nft_response = &mut res.unwrap();
                    let get_nft_response_bytes = get_nft_response.chunk().await.unwrap();
                    let err_resp_vec = get_nft_response_bytes.unwrap().to_vec();
                    let get_nft_response_json = serde_json::from_slice::<NftPortGetNftResponse>(&err_resp_vec);
                    /* 
                        if we're here means that we couldn't map the bytes into the NftPortGetNftResponse 
                        and perhaps we have errors in response from the nftport service
                    */
                    if get_nft_response_json.is_err(){
                            
                        /* log caching using redis */
                        let cloned_err_resp_vec = err_resp_vec.clone();
                        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
                        let get_nft_logs_key_err = format!("ERROR=>NftPortGetNftResponse|Time:{}", chrono::Local::now().to_string());
                        let ـ : RedisResult<String> = redis_conn.set(get_nft_logs_key_err, err_resp_str).await;

                        /* custom error handler */
                        use error::{ErrorKind, ThirdPartyApiError, PanelError};
                        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "start_minting_card_process");
                        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                        return (String::from(""), String::from(""), 1);

                    }

                    /* log caching using redis */
                    let get_nft_response = get_nft_response_json.unwrap();
                    info!("✅ NftPortGetNftResponse: {:#?}", get_nft_response.clone());
                    let get_nft_logs_key = format!("Sender:{}|Log:NftPortGetNftResponse|Time:{}", sender_screen_cid.clone(), chrono::Local::now().to_string());
                    let _: RedisResult<String> = redis_conn.set(get_nft_logs_key, serde_json::to_string_pretty(&get_nft_response).unwrap()).await;


                    if get_nft_response.response == String::from("OK"){

                        let token_id = get_nft_response.token_id;
                        info!("✅ Nft Minted With Id: {}", token_id.clone());
                        info!("✅ Nft Is Inside Contract: {}", contract_address.clone());

                        token_id
                        

                    } else{
                        String::from("")
                    }
                
                };
                
                if mint_tx_hash.starts_with("0x"){
                    return (mint_tx_hash, token_id_string, 0);
                } else{
                    return (String::from(""), String::from(""), 1);
                }

            } else{

                /* mint wasn't ok */
                return (String::from(""), String::from(""), 1);
            }

        } else{

            /* nftport_upload_meta wasn't ok */
            return (String::from(""), String::from(""), 1);
        }

    } else{

        /* upload in ipfs wasn't ok */
        return (String::from(""), String::from(""), 1);

    }
    

}

pub async fn start_burning_card_process( 
    contract_address: String,
    token_id: String,
    redis_client: redis::Client
) -> (String, u8){
    
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();

    /* 
        nft owner must be the contract owner to burn it that's why we minted the nft
        to our app wallet to be able to burn it later
    */
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

    /* ------------------- NFTPORT RESPONSE HANDLING PROCESS -------------------
        since text() and json() method take the ownership of the instance
        thus can't call text() method on ref_resp which is behind a shared ref 
        cause it'll be moved.
        
        let ref_resp = res.as_ref().unwrap();
        let text_resp = ref_resp.text().await.unwrap();

        to solve this issue first we get the stream of the response chunk
        then map it to the related struct, after that we can handle logging
        and redis caching process without losing ownership of things!
    */
    let get_burn_response = &mut res.unwrap();
    let get_burn_response_bytes = get_burn_response.chunk().await.unwrap();
    let err_resp_vec = get_burn_response_bytes.unwrap().to_vec();
    let get_burn_response_json = serde_json::from_slice::<NftPortBurnResponse>(&err_resp_vec);
    /* 
        if we're here means that we couldn't map the bytes into the NftPortBurnResponse 
        and perhaps we have errors in response from the nftport service
    */
    if get_burn_response_json.is_err(){
            
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let burn_nft_logs_key_err = format!("ERROR=>NftPortBurnResponse|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(burn_nft_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "start_burning_card_process");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
        
        return (String::from(""), 1);

    }

    /* log caching using redis */
    let burn_response = get_burn_response_json.unwrap();
    info!("✅ NftPortBurnResponse: {:#?}", burn_response.clone());
    let burn_nft_logs_key = format!("TokenId:{}|Log:NftPortBurnResponse|Time:{}", token_id.clone(), chrono::Local::now().to_string());
    let _: RedisResult<String> = redis_conn.set(burn_nft_logs_key, serde_json::to_string_pretty(&burn_response).unwrap()).await;


    if burn_response.response == String::from("OK"){

        let burn_tx_hash = burn_response.transaction_hash;

        if burn_tx_hash.starts_with("0x"){
            return (burn_tx_hash, 0);
        } else{
            return (String::from(""), 1);
        }

    } else{
        
        return (String::from(""), 1);

    }


}

pub async fn upload_file_to_ipfs(nftport_token: &str, redis_client: redis::Client) -> (NftPortUploadFileToIpfsData, u8){

    let upload_ipfs_response = {

        let mut redis_conn = redis_client.get_async_connection().await.unwrap();
        let auth_header = format!("Authorization: {}", nftport_token);

        let get_upload_output = std::process::Command::new("curl")
            .arg("-X")
            .arg("POST")
            .arg("-H")
            .arg("Content-Type: multipart/form-data")
            .arg("-H")
            .arg(&auth_header)
            .arg("-F")
            .arg("file=@assets/card.png")
            .arg("https://api.nftport.xyz/v0/files")
            .output();

        /* if we're here means that we have io error from the std::process::Command */
        if get_upload_output.is_err(){
            
            /* custom error handler */
            use error::{ErrorKind, ThirdPartyApiError, PanelError};
            let process_cmd_error_content = get_upload_output.as_ref().unwrap_err().to_string();
            let process_cmd_error_content_vec = process_cmd_error_content.as_bytes().to_vec();
            let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, process_cmd_error_content_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(process_cmd_error_content.clone())), "upload_file_to_ipfs");
            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
            
            /* log caching using redis */
            let upload_logs_key_err = format!("ERROR=>ExecutecURLCommand|Time:{}", chrono::Local::now().to_string());
            let ـ : RedisResult<String> = redis_conn.set(upload_logs_key_err, process_cmd_error_content.clone()).await;
            
            error!("cURL process command output error: {}", process_cmd_error_content);
            return (NftPortUploadFileToIpfsData::default(), 1);
        }

        let res = &get_upload_output.as_ref().unwrap().stdout;
        let cloned_err_resp_vec = res.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();

        info!("decoded cURL output response: {}", err_resp_str);
        let get_upload_ipfs_response_json = serde_json::from_slice::<NftPortUploadFileToIpfsData>(&res);
        
        /* 
            if we're here means that we couldn't map the bytes into the NftPortUploadFileToIpfsData 
            and perhaps we have errors in response from the nftport service
        */
        if get_upload_ipfs_response_json.is_err(){

            /* log caching using redis */
            let upload_logs_key_err = format!("ERROR=>NftPortUploadFileToIpfsData|Time:{}", chrono::Local::now().to_string());
            let ـ : RedisResult<String> = redis_conn.set(upload_logs_key_err, err_resp_str).await;

            /* custom error handler */
            use error::{ErrorKind, ThirdPartyApiError, PanelError};
            let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, res.to_owned(), ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "upload_file_to_ipfs");
            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

            error!("serde decoding Nftport response error: {}", err_resp_str);
            return (NftPortUploadFileToIpfsData::default(), 1);

        }

        let upload_ipfs_response = get_upload_ipfs_response_json.unwrap();
        (upload_ipfs_response, 0)

    };

    upload_ipfs_response

}