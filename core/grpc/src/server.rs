


use log::error;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use tonic::Status;
use crate::{*, kyc::kyc_service_server::KycService};
use tonic::Request as TonicRequest;
use tonic::Response as TonicResponse;

const fn app_name<'a>() -> &'a str{
    APP_NAME
}
const APP_NAME: &str = "Conse";


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CheckKycRequest{
    pub caller_cid: String,
    pub tx_signature: String,
    pub hash_data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct UserWalletInfoResponse{
    pub username: String,
    pub avatar: Option<String>,
    pub mail: Option<String>, /* unique */
    pub screen_cid: Option<String>, /* keccak256 */
    pub stars: Option<i64>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct PanelHttpKycResponse{
    /* 
        since we don't know the type of data coming from panel server
        we're mapping it into json Value first then based on our need
        we can map the json value into our desired structure
    */
    pub data: serde_json::Value, /* can by any type of json data */
    pub message: String,
    pub status: u16,
    pub is_error: bool
}

/* this is our server */
#[derive(Clone, Debug, Default)]
pub struct KycServer{}


/* > ------------------------------------------------------------------
   | -> each rpc ds is like an actix actor which contains:
   |    * inner message handlers to communicate with different parts of the app's actors
   |    * tcp based stream handlers and listeners to stream over incoming connections 
   |      to map packet bytes like Capnp, Protobuf, serde_json, Multipart, BSON and 
   |      Payload into desired data struct
   |    * inner concurrent task handlers for sending/receiving message, handling
   |      async tasks from outside of the app using tokio::spawn,mpsc,mailbox,mutex,
   |      rwlock,select,time 
   | -> two actors in two apps communicate through streaming and pubsub channels using rcp http2 and redis
   | -> two actors in an app communicate through streaming and pubsub channels using mpsc and redis 
   |
*/
impl KycServer{

    pub async fn start() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

        let addr = format!("{}:{}", 
            std::env::var("HOST").expect("⚠️ no host variable set"), 
            std::env::var("KYC_GRPC_PORT").expect("⚠️ no panel port variable set").parse::<u16>().unwrap()
        ).parse::<std::net::SocketAddr>().unwrap();

        info!("➔ 🚀 {} panel gRPC server has launched from [{}] at {}", 
            app_name(), addr, chrono::Local::now().naive_local());

        let kyc_rpc_server = KycServer::default(); 
        /* 
            creating a new server service actor from the KycServer 
            structure which is our rpc server 
        */
        TonicServer::builder()
            .add_service(KycServiceServer::new(kyc_rpc_server))
            .serve(addr)
            .await
            .unwrap();
        
        Ok(())
        
    }

    fn restart() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

        Ok(())

    }

}

/* -----------------------------------------------------------------------------------------------
    every actor like actix actors must have some message structs to send pre defined message
    to other actors or send message from different parts of the app to the actor also it must
    contains a handler to handle the incoming messages or streams and respond the caller with
    appropriate message, node.proto is an actor which contains message structs and service 
    handlers KycService handler contains a method called verify which can be used to handle 
    incoming requests and send a response back to the caller, we're implementing the KycService 
    handlder trait for the KycServer struct in here which allows us to handle and accept the 
    requests and send tonic response directly back to the caller of the verify method, for every
    service handler in proto file we have to implement the trait in here for the server struct
    so client can call the methods directly.

    each service in protobuf is a trait in rust which allows us to implement it for any server 
    struct and overwrite its handler methods to accept rpc request in form of single structure
    or streaming and sequencing of data structures.
*/
#[tonic::async_trait]
impl KycService for KycServer{

    /* ---------------------------------------------------------------
        verify is a method of the KycService actor that can be called
        directly by the gRPC client, note that the TonicResponse is 
        not Send means that we can't share it across threads
    */
    async fn verify(&self, request: TonicRequest<KycRequest>) -> Result<TonicResponse<KycResponse>, Status> {

        info!("got an gRPC request at time {:?} | {:?}", 
            chrono::Local::now().naive_local(), request);
        
        let request_parts = request.into_parts();
        let kyc_rpc_request_body = request_parts.2;
        let metadata = request_parts.0;
        let get_headeres = metadata.get("authorization");
        
        let kyc_http_request_body = CheckKycRequest{ 
            caller_cid: kyc_rpc_request_body.cid, 
            tx_signature: kyc_rpc_request_body.tx_signature, 
            hash_data: kyc_rpc_request_body.hash_data 
        };

        match get_headeres{

            Some(metadata_value) => {

                let jwt = format!("Bearer {}", metadata_value.to_str().unwrap());
                
                /* ----- local endpoint ----- */
                // let endpoint = format!("http://{}:{}/health/am-i-kyced",
                //     std::env::var("HOST").unwrap(),
                //     std::env::var("PANEL_PORT").unwrap()
                // );

                let endpoint = std::env::var("KYC_GRPC_PANEL_KYC_CALLBACK").unwrap();
                match reqwest::Client::new()
                    .post(endpoint.as_str())
                    .header("Authorization", &jwt)
                    .json(&kyc_http_request_body)
                    .send()
                    .await
                    {
                        Ok(resp) => {
                                  
                            match resp
                                .json::<PanelHttpKycResponse>()
                                .await
                                {
                                    Ok(kyc_http_resp) => {
                                        
                                        if kyc_http_resp.is_error{

                                            /* terminating the caller with an error */
                                            return Err(Status::unavailable(&format!("panel http server got an error -> {}", kyc_http_resp.message)));
                                        }

                                        let data = serde_json::from_value::<UserWalletInfoResponse>(kyc_http_resp.data).unwrap();
                                        let kyc_resp = KycResponse{ 
                                            username: data.username, 
                                            avatar: data.avatar.unwrap_or(String::from("")), 
                                            mail: data.mail.unwrap_or(String::from("")), 
                                            screen_cid: data.screen_cid.unwrap_or(String::from("")), 
                                            stars: data.stars.unwrap_or(0), 
                                            created_at: data.created_at 
                                        };
            
                                        Ok(TonicResponse::new(kyc_resp))
                                    },
                                    Err(e) => {

                                        error!("error response from panel http server at time {:?} | {}", 
                                            chrono::Local::now().naive_local(), e.to_string());
                                        let kyc_resp = KycResponse::default();
                                        Err(Status::unavailable(&e.to_string()))
                                    }
                                }

                        },
                        Err(e) => {

                            error!("error response from panel http server at time {:?} | {}", 
                                chrono::Local::now().naive_local(), e.to_string());
                            let kyc_resp = KycResponse::default();
                            Err(Status::data_loss(&e.to_string()))

                        }
                    }
            },
            None => {

                error!("found no jwt in metadata {:?}", chrono::Local::now().naive_local());
                let kyc_resp = KycResponse::default();
                Err(Status::unauthenticated("invalid token"))

            }
        }

    }
}