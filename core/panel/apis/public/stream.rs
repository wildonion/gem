

pub use super::*;




// more logics in: https://github.com/wildonion/zoomate/blob/main/src/helpers/tcpserver.rs
#[post("/test-stream")]
pub(self) async fn test_stream(
    // payload and multipart are both in form of bytes that 
    // can be collected using while let some streaming
    req: HttpRequest,
    mut stream: Payload,
    // json_body: web::Json<LoginInfoRequest>,
    // some_path: web::Path<(String, i32)>,
    // multipart_body: Multipart,
) -> Result<actix_web::HttpResponse, helpers::error0::PanelError>{

    // streaming over the incoming binary data from client
    // later on we can map the buffer into its related strucutre
    let mut buffer = vec![];
    while let Some(chunk) = stream.next().await{
        let bytes = chunk.unwrap();
        buffer.extend_from_slice(bytes.chunk());
    }

    // we can use ? operator since the From<std::io::Error> trait has implemented for the PanelError
    // runtime ERROR: cause file doesn't exist
    let f = std::fs::File::open("openme.txt")?; 

    // extracting multipart formdata
    // let extracted_multipart = multipartreq::extract(
    //     std::sync::Arc::new(
    //         tokio::sync::Mutex::new(multipart_body)
    //     )
    // ).await.unwrap();
    // let json_value_formdata = extracted_multipart.0;
    // let files = extracted_multipart.1;

    // getting the json body
    // let json_body = json_body.to_owned();


    tokio::spawn(async move{ 

        // start a tcp streamer in the background 
        helpers::server::start_streaming().await;
         
    });

    resp!{
        usize, // the data type
        buffer.len(), // response data
        &format!("Stream Length Fetched"), // response message
        StatusCode::OK, // status code
        None::<Cookie<'_>>, // cookie
    }

}

pub mod exports{
    pub use super::test_stream;
}