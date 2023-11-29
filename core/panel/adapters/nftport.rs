


use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::*;
use crate::misc::Response;
use crate::models::users_collections::{NewUserCollectionRequest, UpdateUserCollectionRequest};
use crate::models::users_nfts::{NewUserNftRequest, UpdateUserNftRequest};
use crate::schema::users::identifier;
use crate::schema::users_nfts::nft_description;
use actix_web::web::Query;
use mongodb::bson::oid::ObjectId;
use redis_async::client::PubsubConnection;
use serde_json::json;
 
use crate::*;
use crate::constants::{CHARSET, APP_NAME, THIRDPARTYAPI_ERROR_CODE, TWITTER_24HOURS_LIMITED, NFT_UPLOAD_PATH, NFT_UPLOAD_ISSUE, EMPTY_NFT_IMG, UNSUPPORTED_FILE_TYPE};
use crate::events::publishers::role::PlayerRoleInfo;
use crate::models::users::{NewIdRequest, IpInfoResponse, User};
use crate::models::users_deposits::NewUserDepositRequest;
use crate::models::users_tasks::UserTask;
use actix::Addr;


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
pub struct NftPortTransferResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUpdateNftResponse{
    pub response: String,
    pub chain: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortCreateCollectionContractResponse{
    pub response: String,
    pub chain: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
    pub owner_address: String,
    pub r#type: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct OnchainNfts{
    pub onchain_nfts: Option<serde_json::Value>
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NftPortUpdateCollectionContractResponse{
    pub response: String,
    pub chain: String,
    pub transaction_hash: String,
    pub transaction_external_url: String,
    pub freeze_metadata: bool
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
pub struct NftPortUploadMetadataToIpfsResponse{
    pub response: String,
    pub metadata_uri: String,
    pub name: String,
    pub description: String,
    pub file_url: String,
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


/* transfer nft by minting to recipient that has token value behind of it */
pub async fn start_minting_card_process(
    sender_screen_cid: String,
    deposit_object: NewUserDepositRequest, 
    contract_address: String,
    contract_owner: String,
    polygon_recipient_address: String,
    nft_img_url: String,
    nft_name: String,
    nft_desc: String,
    redis_client: redis::Client
) -> (String, String, u8){

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    
    /* upload card to ipfs */
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();

    /* ---------------------------- we're using the ifps nft_img_url ----------------------------
    let (metadata_uri, res_metadata_uri_status) = upload_file_to_ipfs(&nftport_token, redis_client.clone()).await;
        
    if res_metadata_uri_status == 1{
        return (String::from(""), String::from(""), 1);
    }

    /* log caching using redis */
    let upload_logs_key = format!("Sender:{}|Log:NftPortUploadFileToIpfsData|Time:{}", sender_screen_cid.clone(), chrono::Local::now().to_string());
    let ـ : RedisResult<String> = redis_conn.set(upload_logs_key, serde_json::to_string_pretty(&metadata_uri).unwrap()).await;
    info!("✅ NftPortUploadFileToIpfsData: {:#?}", metadata_uri.clone());
    if metadata_uri.response == String::from("OK"){
    --------------------------------------------------------------------------------------- */

    if !nft_img_url.is_empty(){

        // let metadata_uri = metadata_uri.ipfs_url;
        let metadata_uri = nft_img_url; // front has already uploaded the img of nft in ipfs

        /* upload metadata to ipfs */
        let mut custom_fields = HashMap::new();
        custom_fields.insert("amount".to_string(), deposit_object.amount.to_string());
        custom_fields.insert("sender".to_string(), sender_screen_cid.clone());
        custom_fields.insert("recipient".to_string(), polygon_recipient_address.clone());
        let upload_data = NftPortUploadMetadataRequest{
            name: nft_name,
            description: nft_desc,
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
            let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::start_minting_card_process");
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
                let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::start_minting_card_process");
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
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

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
                        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::start_minting_card_process");
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

pub async fn start_transferring_card_process( 
    contract_address: String,
    token_id: String,
    polygon_recipient_address: String,
    redis_client: redis::Client
) -> (String, u8){
    
    /* 
        Note: transferring is possible only if the token is owned by the contract owner and 
        the token has not been transferred/sold yet and that's why we're not burning nft in
        here because nft owner must be the contract owner to burn it and we want to make the
        user the owner of the nft once he claimed the tokens which allows user to sell his nft 
        on polygon markeplaces later 
    */

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();

    let mut transfer_data = HashMap::new();
    transfer_data.insert("chain", "polygon");
    transfer_data.insert("contract_address", &contract_address);
    transfer_data.insert("token_id", &token_id);
    transfer_data.insert("transfer_to_address", &polygon_recipient_address);
    let nftport_transfer_endpoint = format!("https://api.nftport.xyz/v0/mints/transfers");
    let res = reqwest::Client::new()
        .post(nftport_transfer_endpoint.as_str())
        .header("Authorization", nftport_token.as_str())
        .json(&transfer_data)
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
    let get_transfer_response = &mut res.unwrap();
    let get_transfer_response_bytes = get_transfer_response.chunk().await.unwrap();
    let err_resp_vec = get_transfer_response_bytes.unwrap().to_vec();
    let get_transfer_response_json = serde_json::from_slice::<NftPortTransferResponse>(&err_resp_vec);
    /* 
        if we're here means that we couldn't map the bytes into the NftPortTransferResponse 
        and perhaps we have errors in response from the nftport service
    */
    if get_transfer_response_json.is_err(){
            
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let transfer_nft_logs_key_err = format!("ERROR=>NftPortTransferResponse|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(transfer_nft_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::start_transferring_card_process");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
        
        return (String::from(""), 1);

    }

    /* log caching using redis */
    let transfer_response = get_transfer_response_json.unwrap();
    info!("✅ NftPortTransferResponse: {:#?}", transfer_response.clone());
    let transfer_nft_logs_key = format!("TokenId:{}|Log:NftPortTransferResponse|Time:{}", token_id.clone(), chrono::Local::now().to_string());
    let _: RedisResult<String> = redis_conn.set(transfer_nft_logs_key, serde_json::to_string_pretty(&transfer_response).unwrap()).await;


    if transfer_response.response == String::from("OK"){

        let transfer_tx_hash = transfer_response.transaction_hash;

        if transfer_tx_hash.starts_with("0x"){
            return (transfer_tx_hash, 0);
        } else{
            return (String::from(""), 1);
        }

    } else{
        
        return (String::from(""), 1);

    }


}

pub async fn upload_file_to_ipfs(nftport_token: &str, redis_client: redis::Client, img_path: &str) -> (NftPortUploadFileToIpfsData, u8){

    let upload_ipfs_response = {

        let mut redis_conn = redis_client.get_async_connection().await.unwrap();
        let auth_header = format!("Authorization: {}", nftport_token);
        let pic_path = format!("file=@{}", img_path);
        
        let get_upload_output = std::process::Command::new("curl")
            .arg("-X")
            .arg("POST")
            .arg("-H")
            .arg("Content-Type: multipart/form-data")
            .arg("-H")
            .arg(&auth_header)
            .arg("-F")
            .arg(pic_path)
            .arg("https://api.nftport.xyz/v0/files")
            .output();

        /* if we're here means that we have io error from the std::process::Command */
        if get_upload_output.is_err(){
            
            /* custom error handler */
            use error::{ErrorKind, ThirdPartyApiError, PanelError};
            let process_cmd_error_content = get_upload_output.as_ref().unwrap_err().to_string();
            let process_cmd_error_content_vec = process_cmd_error_content.as_bytes().to_vec();
            let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, process_cmd_error_content_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(process_cmd_error_content.clone())), "nftport::upload_file_to_ipfs");
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
            let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, res.to_owned(), ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::upload_file_to_ipfs");
            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

            error!("serde decoding Nftport response error: {}", err_resp_str);
            return (NftPortUploadFileToIpfsData::default(), 1);

        }

        let upload_ipfs_response = get_upload_ipfs_response_json.unwrap();
        (upload_ipfs_response, 0)

    };

    upload_ipfs_response

}

pub async fn create_collection(
    redis_client: redis::Client,
    new_collection_request: NewUserCollectionRequest,
) -> (String, String, u8){

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();
    let NewUserCollectionRequest{ 
        col_name, 
        symbol, 
        owner_cid, 
        metadata_updatable, 
        base_uri, 
        royalties_share, 
        royalties_address_screen_cid, 
        .. /* don't care about the rest of the fields */ 
    } = new_collection_request;

    let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid);
    let mut collection_data = HashMap::new();
    collection_data.insert("chain", "polygon");
    collection_data.insert("name", &col_name);
    collection_data.insert("symbol", &symbol);
    collection_data.insert("owner_address", owner_screen_cid);
    
    let mu = format!("{}", metadata_updatable.unwrap());
    collection_data.clone().insert("metadata_updatable", mu.as_str());
    
    let rs = format!("{}", royalties_share);
    collection_data.insert("royalties_share", rs.as_str());
    
    collection_data.insert("base_uri", &base_uri);
    collection_data.insert("royalties_address", &royalties_address_screen_cid);
    let nftport_create_collection_endpoint = format!("https://api.nftport.xyz/v0/contracts");
    let res = reqwest::Client::new()
        .post(nftport_create_collection_endpoint.as_str())
        .header("Authorization", nftport_token.as_str())
        .json(&collection_data)
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
    let get_collection_creation_response = &mut res.unwrap();
    let get_collection_creation_response_bytes = get_collection_creation_response.chunk().await.unwrap();
    let err_resp_vec = get_collection_creation_response_bytes.unwrap().to_vec();
    let get_collection_creation_response_json = serde_json::from_slice::<NftPortCreateCollectionContractResponse>(&err_resp_vec);
    /* 
        if we're here means that we couldn't map the bytes into the NftPortCreateCollectionContractResponse 
        and perhaps we have errors in response from the nftport service
    */
    if get_collection_creation_response_json.is_err(){
            
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let collection_creation_logs_key_err = format!("ERROR=>NftPortCreateCollectionContractResponse|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(collection_creation_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::create_collection");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
        
        return (String::from(""), String::from(""), 1);

    }

    /* log caching using redis */
    let collection_creation = get_collection_creation_response_json.unwrap();
    info!("✅ NftPortCreateCollectionContractResponse: {:#?}", collection_creation.clone());
    let collection_creation_logs_key = format!("OwnerAddress:{}|Log:NftPortCreateCollectionContractResponse|Time:{}", collection_creation.owner_address.clone(), chrono::Local::now().to_string());
    let _: RedisResult<String> = redis_conn.set(collection_creation_logs_key, serde_json::to_string_pretty(&collection_creation).unwrap()).await;


    if collection_creation.response == String::from("OK"){

        /* sleep till the transaction gets confirmed on blockchain */
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        /* getting the deployed contract address */
        let get_tx_hash_info = format!("https://api.nftport.xyz/v0/contracts/{}?chain=polygon", collection_creation.transaction_hash);
        let res: serde_json::Value = reqwest::Client::new()
            .get(get_tx_hash_info.as_str())
            .header("Authorization", nftport_token.as_str())
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
            

        let res = serde_json::to_value(res).unwrap();
        let collection_contract_address = if res.get("contract_address").is_some(){
            res.get("contract_address").unwrap().as_str().unwrap().to_string()
        } else{
            String::from("")
        };

        if collection_contract_address.starts_with("0x"){
            return (collection_contract_address, collection_creation.transaction_hash, 0);
        } else{
            /* can't get contract onchain info */
            return (String::from(""), collection_creation.transaction_hash, 1);
        }

    } else{
        
        return (String::from(""), String::from(""), 1);

    }



}

pub async fn update_collection(
    redis_client: redis::Client,
    update_collection_request: UpdateUserCollectionRequest,
    contract_address: String,
) -> (String, u8){

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();
    let UpdateUserCollectionRequest{ 
        owner_cid, 
        base_uri, 
        royalties_share, 
        royalties_address_screen_cid, 
        freeze_metadata,
        .. /* don't care about the rest of the fields */
    } = update_collection_request;

    let owner_screen_cid = &walletreq::evm::get_keccak256_from(owner_cid);
    let mut collection_data = HashMap::new();
    collection_data.insert("chain", "polygon");
    collection_data.insert("contract_address", &contract_address);

    let fzm = &format!("{}", freeze_metadata);
    collection_data.insert("freeze_metadata", fzm);
    
    /* 
        if the contract is not already frozen and the metadata_updatable 
        is not false we can update the base_uri 
    */
    if !base_uri.is_empty(){
        collection_data.insert("base_uri", &base_uri);
    }

    let rs = format!("{}", royalties_share);
    collection_data.insert("royalties_share", rs.as_str());
    collection_data.insert("royalties_address", &royalties_address_screen_cid);
    let nftport_update_collection_endpoint = format!("https://api.nftport.xyz/v0/contracts");
    let res = reqwest::Client::new()
        .put(nftport_update_collection_endpoint.as_str())
        .header("Authorization", nftport_token.as_str())
        .json(&collection_data)
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
    let get_collection_update_response = &mut res.unwrap();
    let get_collection_update_response_bytes = get_collection_update_response.chunk().await.unwrap();
    let err_resp_vec = get_collection_update_response_bytes.unwrap().to_vec();
    let get_collection_update_response_json = serde_json::from_slice::<NftPortUpdateCollectionContractResponse>(&err_resp_vec);
    /* 
        if we're here means that we couldn't map the bytes into the NftPortUpdateCollectionContractResponse 
        and perhaps we have errors in response from the nftport service
    */
    if get_collection_update_response_json.is_err(){
            
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let collection_update_logs_key_err = format!("ERROR=>NftPortUpdateCollectionContractResponse|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(collection_update_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::update_collection");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
        
        return (String::from(""), 1);

    }

    /* log caching using redis */
    let collection_update = get_collection_update_response_json.unwrap();
    info!("✅ NftPortUpdateCollectionContractResponse: {:#?}", collection_update.clone());
    let collection_update_logs_key = format!("OwnerAddress:{}|Log:NftPortUpdateCollectionContractResponse|Time:{}", owner_screen_cid, chrono::Local::now().to_string());
    let _: RedisResult<String> = redis_conn.set(collection_update_logs_key, serde_json::to_string_pretty(&collection_update).unwrap()).await;


    if collection_update.response == String::from("OK"){

        if collection_update.transaction_hash.starts_with("0x"){
            return (collection_update.transaction_hash, 0);
        } else{
            /* can't get contract onchain info */
            return (String::from(""), 1);
        }

    } else{
        
        return (String::from(""), 1);

    }

}

pub trait NftExt{
    type AssetInfo; /* type of asset, either NewUserNftRequest or UpdateUserNftRequest */
    fn get_nft_name(&self) -> String;
    fn get_nft_description(&self) -> String;
    fn get_nft_contract_address(&self) -> String;
    fn get_nft_current_owner_address(&self) -> String;
    fn get_nft_extra(&self) -> Option<serde_json::Value>;
    fn get_nft_attribute(&self) -> Option<serde_json::Value>;
    fn get_recipient_screen_cid(&self) -> String;
    fn get_self(self) -> Self::AssetInfo;
}

pub async fn upload_nft_to_ipfs<N>(
    redis_client: redis::Client,
    nft_img_path_on_server: String,
    asset_info: N
) -> String where N: NftExt + Clone + Send + Sync + 'static{


    let nft_name_ = asset_info.get_nft_name();
    let nft_description_ = asset_info.get_nft_description();
    let nft_attributes = asset_info.get_nft_attribute().unwrap();

    let asset_info = asset_info.get_self();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();

    let (upload_ipfs_response, status) = self::upload_file_to_ipfs(&nftport_token, redis_client, &nft_img_path_on_server).await;
    
    if status == 1{
        return String::from("");
    }

    let file_url = upload_ipfs_response.ipfs_url;

    /* 
        sending all data as json value to the nftport server since attributes 
        must be a valid json list not string 
    */
    let mut upload_data = HashMap::new();
    upload_data.insert("name", serde_json::to_value(nft_name_.clone()).unwrap());
    upload_data.insert("description", serde_json::to_value(nft_description_).unwrap());
    upload_data.insert("file_url", serde_json::to_value(file_url).unwrap());
    upload_data.insert("attributes", nft_attributes);
    let nftport_upload2ipfs_endpoint = format!("https://api.nftport.xyz/v0/metadata");
    let res = reqwest::Client::new()
        .post(nftport_upload2ipfs_endpoint.as_str())
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
    let get_upload_response = &mut res.unwrap();
    let get_upload_response_bytes = get_upload_response.chunk().await.unwrap();
    let err_resp_vec = get_upload_response_bytes.unwrap().to_vec();
    let get_upload_response_json = serde_json::from_slice::<NftPortUploadMetadataToIpfsResponse>(&err_resp_vec);
    /* 
        if we're here means that we couldn't map the bytes into the NftPortUploadMetadataToIpfsResponse 
        and perhaps we have errors in response from the nftport service
    */
    if get_upload_response_json.is_err(){
            
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let upload_logs_key_err = format!("ERROR=>NftPortUploadMetadataToIpfsResponse|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(upload_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::upload_nft_to_ipfs");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

        return String::from("");

    }

    /* log caching using redis */
    let upload_response = get_upload_response_json.unwrap();
    info!("✅ NftPortUploadMetadataToIpfsResponse: {:#?}", upload_response.clone());
    let upload_logs_key = format!("NftName:{}|Log:NftPortUploadMetadataToIpfsResponse|Time:{}", nft_name_, chrono::Local::now().to_string());
    let _: RedisResult<String> = redis_conn.set(upload_logs_key, serde_json::to_string_pretty(&upload_response).unwrap()).await;
    

    if upload_response.response == String::from("OK"){
    
        if upload_response.metadata_uri.starts_with("ipfs"){
            return upload_response.metadata_uri;
        } else{
            return String::from("");
        }
    
    } else{
        return String::from("");
    }

}

pub async fn mint_nft(
    redis_client: redis::Client,
    asset_info: UpdateUserNftRequest
) -> (String, String, u8){

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    
    /* upload card to ipfs */
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();

    if !asset_info.metadata_uri.is_empty(){

        let metadata_uri = asset_info.metadata_uri;

        /* mint request */
        let mut mint_data = HashMap::new();
        mint_data.insert("chain", "polygon");
        mint_data.insert("contract_address", &asset_info.contract_address);
        mint_data.insert("metadata_uri", &metadata_uri);
        mint_data.insert("mint_to_address", &asset_info.current_owner_screen_cid);
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
            let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::mint_nft");
            let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

            return (String::from(""), String::from(""), 1);

        }

        /* log caching using redis */
        let mint_response = get_mint_response_json.unwrap();
        info!("✅ NftPortMintResponse: {:#?}", mint_response.clone());
        let mint_logs_key = format!("Minter:{}|Log:NftPortMintResponse|Time:{}", asset_info.current_owner_screen_cid.clone(), chrono::Local::now().to_string());
        let _: RedisResult<String> = redis_conn.set(mint_logs_key, serde_json::to_string_pretty(&mint_response).unwrap()).await;
        

        if mint_response.response == String::from("OK"){

            let mint_tx_hash = mint_response.transaction_hash;

            /* sleep till the transaction gets confirmed on blockchain */
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

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
                    let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::mint_nft");
                    let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */

                    return (String::from(""), String::from(""), 1);

                }

                /* log caching using redis */
                let get_nft_response = get_nft_response_json.unwrap();
                info!("✅ NftPortGetNftResponse: {:#?}", get_nft_response.clone());
                let get_nft_logs_key = format!("Minter:{}|Log:NftPortGetNftResponse|Time:{}", asset_info.current_owner_screen_cid.clone(), chrono::Local::now().to_string());
                let _: RedisResult<String> = redis_conn.set(get_nft_logs_key, serde_json::to_string_pretty(&get_nft_response).unwrap()).await;


                if get_nft_response.response == String::from("OK"){

                    let token_id = get_nft_response.token_id;
                    info!("✅ Nft Minted With Id: {}", token_id.clone());
                    info!("✅ Nft Is Inside Contract: {}", asset_info.contract_address.clone());

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

        /* upload in ipfs wasn't ok */
        return (String::from(""), String::from(""), 1);

    }

}

pub async fn transfer_nft(
    redis_client: redis::Client,
    asset_info: UpdateUserNftRequest,
) -> (String, u8){

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();

    let transfer_to = asset_info.get_recipient_screen_cid();

    if transfer_to.is_empty(){
        return (String::from(""), 1);
    }

    let contract_address = asset_info.clone().contract_address;
    let token_id = asset_info.clone().onchain_id.unwrap();

    let mut transfer_data = HashMap::new();
    transfer_data.insert("chain", "polygon");
    transfer_data.insert("contract_address", &contract_address);
    transfer_data.insert("token_id", &token_id);
    transfer_data.insert("transfer_to_address", &transfer_to);
    let nftport_transfer_endpoint = format!("https://api.nftport.xyz/v0/mints/transfers");
    let res = reqwest::Client::new()
        .post(nftport_transfer_endpoint.as_str())
        .header("Authorization", nftport_token.as_str())
        .json(&transfer_data)
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
    let get_transfer_response = &mut res.unwrap();
    let get_transfer_response_bytes = get_transfer_response.chunk().await.unwrap();
    let err_resp_vec = get_transfer_response_bytes.unwrap().to_vec();
    let get_transfer_response_json = serde_json::from_slice::<NftPortTransferResponse>(&err_resp_vec);
    /* 
        if we're here means that we couldn't map the bytes into the NftPortTransferResponse 
        and perhaps we have errors in response from the nftport service
    */
    if get_transfer_response_json.is_err(){
            
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let transfer_nft_logs_key_err = format!("ERROR=>NftPortTransferResponse|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(transfer_nft_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::transfer_nft");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
        
        return (String::from(""), 1);

    }

    /* log caching using redis */
    let transfer_response = get_transfer_response_json.unwrap();
    info!("✅ NftPortTransferResponse: {:#?}", transfer_response.clone());
    let transfer_nft_logs_key = format!("TokenId:{}|Log:NftPortTransferResponse|Time:{}", token_id, chrono::Local::now().to_string());
    let _: RedisResult<String> = redis_conn.set(transfer_nft_logs_key, serde_json::to_string_pretty(&transfer_response).unwrap()).await;


    if transfer_response.response == String::from("OK"){

        let transfer_tx_hash = transfer_response.transaction_hash;

        if transfer_tx_hash.starts_with("0x"){
            return (transfer_tx_hash, 0);
        } else{
            return (String::from(""), 1);
        }

    } else{
        
        return (String::from(""), 1);

    }

}

pub async fn update_nft(
    redis_client: redis::Client,
    asset_info: UpdateUserNftRequest
) -> (String, u8){

    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();

    let contract_address = asset_info.clone().contract_address;
    let token_id = asset_info.clone().onchain_id.unwrap();
    let metadata_uri = asset_info.clone().metadata_uri;
    let freeze_metadata = format!("{}", asset_info.clone().freeze_metadata.unwrap());

    let mut update_data = HashMap::new();
    update_data.insert("chain", "polygon");
    update_data.insert("contract_address", &contract_address);
    update_data.insert("token_id", &token_id);
    update_data.insert("metadata_uri", &metadata_uri);
    update_data.insert("freeze_metadata", &freeze_metadata);
    let nftport_update_endpoint = format!("https://api.nftport.xyz/v0/mints/customizable");
    let res = reqwest::Client::new()
        .put(nftport_update_endpoint.as_str())
        .header("Authorization", nftport_token.as_str())
        .json(&update_data)
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
    let get_update_response = &mut res.unwrap();
    let get_update_response_bytes = get_update_response.chunk().await.unwrap();
    let err_resp_vec = get_update_response_bytes.unwrap().to_vec();
    let get_update_response_json = serde_json::from_slice::<NftPortUpdateNftResponse>(&err_resp_vec);
    /* 
        if we're here means that we couldn't map the bytes into the NftPortUpdateNftResponse 
        and perhaps we have errors in response from the nftport service
    */
    if get_update_response_json.is_err(){
            
        /* log caching using redis */
        let cloned_err_resp_vec = err_resp_vec.clone();
        let err_resp_str = std::str::from_utf8(cloned_err_resp_vec.as_slice()).unwrap();
        let update_nft_logs_key_err = format!("ERROR=>NftPortUpdateNftResponse|Time:{}", chrono::Local::now().to_string());
        let ـ : RedisResult<String> = redis_conn.set(update_nft_logs_key_err, err_resp_str).await;

        /* custom error handler */
        use error::{ErrorKind, ThirdPartyApiError, PanelError};
        let error_instance = PanelError::new(*THIRDPARTYAPI_ERROR_CODE, err_resp_vec, ErrorKind::ThirdPartyApi(ThirdPartyApiError::ReqwestTextResponse(err_resp_str.to_string())), "nftport::update_nft");
        let error_buffer = error_instance.write().await; /* write to file also returns the full filled buffer from the error  */
        
        return (String::from(""), 1);

    }

    /* log caching using redis */
    let update_response = get_update_response_json.unwrap();
    info!("✅ NftPortUpdateNftResponse: {:#?}", update_response.clone());
    let update_nft_logs_key = format!("TokenId:{}|Log:NftPortUpdateNftResponse|Time:{}", token_id, chrono::Local::now().to_string());
    let _: RedisResult<String> = redis_conn.set(update_nft_logs_key, serde_json::to_string_pretty(&update_response).unwrap()).await;


    if update_response.response == String::from("OK"){

        let update_tx_hash = update_response.transaction_hash;

        if update_tx_hash.starts_with("0x"){
            return (update_tx_hash, 0);
        } else{
            return (String::from(""), 1);
        }

    } else{
        
        return (String::from(""), 1);

    }


}

/*  -----------------------------------------------------------------------------------------
    we've defined the asset_info as a generic type and bound it to NftExt trait to get those
    data fields that we need to store nft image onchain by calling trait methods on either 
    NewUserNftRequest (insert new nft) or UpdateUserNftRequest (update nft) instances cause
    both of these two structures have the fields that we want it but don't know which one is
    going to be passed to the method!
*/
type Files = HashMap<String, Vec<u8>>;
pub async fn get_nft_onchain_metadata_uri<N>(
    file: (String, Vec<u8>), /* the filename and its bytes */
    redis_client: redis::Client, 
    asset_info: N) -> Result<String, PanelHttpResponse>
    where N: NftExt + Clone + Send + Sync + 'static, 
    Files: Clone + Send + Sync + 'static,
        <N as NftExt>::AssetInfo: Send + Sync + 'static{ /* also the AssetInfo, the dynamic type, in trait must be bounded to Send Sync 'static */

    /* uploading nft image on server */
    let filename = file.0;
    /*************************************************************************/
    /*** we can manipulate image bytes in here, doing some image processing 
     *** like shifting and swapping bytes :) ***/
    /*************************************************************************/
    let img_bytes = file.1; 

    let ext_position_png = filename.find("png");
    let ext_position_jpg = filename.find("jpg");
    let ext_position_jpeg = filename.find("jpeg");
    let ext_position_pdf = filename.find("pdf");
    let ext_position_mp4 = filename.find("mp4");
    let ext_position_mp3 = filename.find("mp3");
    let ext_position_gif = filename.find("gif");

    let (ext_position, file_kind) = if ext_position_png.is_some(){
        (ext_position_png.unwrap(), "img")
    } else if ext_position_jpg.is_some(){
        (ext_position_jpg.unwrap(), "img")
    } else if ext_position_jpeg.is_some(){
        (ext_position_jpeg.unwrap(), "img")
    } else if ext_position_pdf.is_some(){
        (ext_position_pdf.unwrap(), "pdf")
    } else if ext_position_mp4.is_some(){
        (ext_position_mp4.unwrap(), "mp4")
    } else if ext_position_mp3.is_some(){
        (ext_position_mp3.unwrap(), "mp3")
    } else if ext_position_gif.is_some(){
        (ext_position_gif.unwrap(), "gif")
    } else{

        let resp = Response::<&[u8]>{
            data: Some(&[]),
            message: UNSUPPORTED_FILE_TYPE,
            status: 406,
            is_error: true,
        };
        return Err(
            Ok(HttpResponse::NotAcceptable().json(resp))
        );
    };
    
    let mut nft_img_path = String::from("");
    tokio::fs::create_dir_all(NFT_UPLOAD_PATH).await.unwrap();
    let identifier_ = &format!("nft:{}-incontract:{}-by:{}", asset_info.get_nft_name(), asset_info.get_nft_contract_address(), asset_info.get_nft_current_owner_address());
    let img_filename = format!("{}:{}-{}:{}.{}", "nft", identifier_, file_kind, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(), &filename[ext_position..]);
    let filepath = format!("{}/{}", NFT_UPLOAD_PATH, sanitize_filename::sanitize(&img_filename));
    nft_img_path = filepath.clone();
    
    /* 
        web::block() executes a blocking function on a actix threadpool
        using spawn_blocking method of actix runtime so in here we're 
        creating a file inside a actix runtime threadpool to fill it with 
        the incoming bytes inside the field object by streaming over field
        object to extract the bytes
    */
    let mut f = web::block(|| std::fs::File::create(filepath).unwrap()).await.unwrap();

    /* writing fulfilled buffer bytes into the created file with the created filepath */
    f = web::block(move || f.write_all(&img_bytes).map(|_| f))
        .await
        .unwrap()
        .unwrap();

    if nft_img_path.is_empty(){
            
        let resp = Response::<'_, &[u8]>{
            data: Some(&[]),
            message: EMPTY_NFT_IMG,
            status: 406,
            is_error: true
        };
        return Err(
            Ok(HttpResponse::NotAcceptable().json(resp))
        );
    }

    /* 
        start uploading nft in the background inside tokio green threadpool, the metadata 
        uri will be shared between outside of the threadpool and tokio spawn using 
        mpsc jobq channel
    */
    let mut nft_metadata_uri = String::from("");
    let (metadata_uri_sender, mut metadata_uri_receiver)
        = tokio::sync::mpsc::channel::<String>(1024);
    let asset_data = asset_info.clone();
    tokio::spawn(async move{

        let final_metadata_uri = self::upload_nft_to_ipfs(
            redis_client.clone(), 
            nft_img_path,
            asset_data
        ).await;

        /* 
            sending the final_metadata_uri into the mpsc channel so we can receive it 
            in other scopes outside of tokio::spawn green threadpool
        */
        if let Err(why) = metadata_uri_sender.clone().send(final_metadata_uri).await{
            error!("can't send `final_metadata_uri` to the mpsc channel because: {}", why.to_string());
        }

    });

    /* receiving asyncly from the channel in outside of the tokio spawn */
    while let Some(uri) = metadata_uri_receiver.recv().await{
        nft_metadata_uri = uri;
    }

    if nft_metadata_uri.is_empty(){

        let resp = Response::<'_, &[u8]>{
            data: Some(&[]),
            message: NFT_UPLOAD_ISSUE,
            status: 417,
            is_error: true
        };
        return Err(
            Ok(HttpResponse::ExpectationFailed().json(resp))
        );

    }

    Ok(nft_metadata_uri)

}

pub async fn get_nfts_owned_by(caller_screen_cid: &str, from: i64, to: i64) -> OnchainNfts{

    /* -----------------------------------------------------------------
        > in those case that we don't want to create a separate struct 
        and allocate an instance of it to map a utf8 bytes data coming
        from a server or client into its feilds we can use serde_json::to_value()
        which maps an instance of a structure into a serde json value 
        or serde_json::json!({}) to create a json value from those fields 
        that we want to return them, but if we want to mutate data in rust we 
        have to convert the json value or received bytes into the structure, 
    */
    let nftport_token = std::env::var("NFTYPORT_TOKEN").unwrap();
    let nftport_get_nfts = format!("https://api.nftport.xyz/v0/accounts/{}?chain=polygon&page_size={}&continuation={}&include=metadata", caller_screen_cid, to, from);
    let res_value: serde_json::Value = reqwest::Client::new()
        .get(nftport_get_nfts.as_str())
        .header("Authorization", nftport_token.as_str())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    /* return the response directly to the client without mapping it into the struct */
    OnchainNfts{
        onchain_nfts: Some(res_value)
    }


}