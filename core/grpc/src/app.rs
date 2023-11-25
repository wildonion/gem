

use env_logger::Env;
/* 
    methods, structures from kyc modules which is loaded from the kyc 
    rust code by compiling proto file 
*/
use kyc::{
    KycRequest, 
    KycResponse, 
    kyc_service_client::KycServiceClient, 
    kyc_service_server::KycServiceServer
};
use tonic::transport::Server as TonicServer;
use crate::server::KycServer;

mod server;

/* ---------------------------------------------------------
    loading the compiled proto file into rust code in here 
    contains traits and data structures to use them in here 
    to create rpc server and client
*/
pub mod kyc{
    tonic::include_proto!("kyc");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    dotenv::dotenv().expect(".env file must be in here!");
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    KycServer::start().await

}