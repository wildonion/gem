

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
    to create rpc server and client, once we build the server
    for production all the generated rust codes from proto 
    will be compile too, thus there is no need to put any code
    in the root path.

    the include! macro is primarily used for two purposes, 
    it is used to include documentation that is written in a 
    separate file and it is used to include build artifacts 
    usually as a result from the build.rs script, so there must 
    be expressions in a file or crate that want to be loaded 
    with include!() macro
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