


use tonic::Status;
use crate::{*, kyc::kyc_service_server::KycService};
use tonic::Request as TonicRequest;
use tonic::Response as TonicResponse;


/* this is our server */
#[derive(Clone, Debug, Default)]
pub struct KycServer{}


impl KycServer{

    pub async fn start() -> std::io::Result<()>{

        let addr = format!("{}:{}", 
            std::env::var("HOST").expect("⚠️ no host variable set"), 
            std::env::var("KYC_GRPC_PORT").expect("⚠️ no panel port variable set").parse::<u16>().unwrap()
        ).parse::<std::net::SocketAddr>().unwrap();
    
        let kyc = KycServer::default(); 
        TonicServer::builder()
            /* 
                creating a new server service actor from the EchoServer 
                structure which is our rpc server 
            */
            .add_service(KycServiceServer::new(kyc))
            .serve(addr)
            .await
            .unwrap();

        Ok(())
        
    }
}

/* -----------------------------------------------------------------------------------------
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
*/
#[tonic::async_trait]
impl KycService for KycServer{

    /* --------------------------------------------------------------
        verify is a method of the KycService actor that can be called
        directly by the gRPC client
    */
    async fn verify(&self, request: TonicRequest<KycRequest>) -> Result<TonicResponse<KycResponse>, Status> {

        println!("Got a request {:?}", request);

        let resp = KycResponse{
            screen_cid: format!("{}", walletreq::evm::get_keccak256_from(request.into_inner().cid)),
        };

        Ok(TonicResponse::new(resp))

    }
}