

use std::error::Error;

pub use super::*;



pub async fn open_file() -> Result<(), helpers::error0::FileEror>{

    // in order to use the ? operator the From<std::io::Error> trait must be 
    // implemented for the FileEror so Rust can create the error by calling 
    // the from() method on the FileEror to create the error type based on 
    // the error variant which in our case is std::io::Error
    let f = std::fs::File::open("openme.txt")?;
    Ok(())

}


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
) -> Result<actix_web::HttpResponse, helpers::error0::PanelErrorResponse>{

    // streaming over the incoming binary data from client
    // later on we can map the buffer into its related strucutre
    let mut buffer = vec![];
    while let Some(chunk) = stream.next().await{
        let bytes = chunk.unwrap();
        buffer.extend_from_slice(bytes.chunk());
    }
    
    // building the error of read/write file manually so we could return 
    // PanelErrorResponse in respond to the client

    // note that in the following method we've used the FileEror as the error part
    // of the result type which unwrap the error by using ? to log the exact caused 
    // of error to the console but note that can't use ? in here cause ? unwrap the
    // the error into PanelErrorResponse not the its KindaError enum variant, we use
    // match in here to catch the error
    match open_file().await{
        Ok(_) => {},
        Err(e) => { // as we can see the error type is a FileError which is one the variant of the ErrorKind enum
            // e.to_string() is the display message of the error, note without 
            // matching over the result and use unwrap() only the app gets crashed 
            // at runtime and logs the fulfilled buffer inside the Debug trait the 
            // fmt() method like so:
            // [FILE] - failed to read from or write to file
            // Caused by: 
            // No such file or directory (os error 2)
            // cause this api method requires an error type of PanelErrorResponse
            let source_error = e.source().unwrap().to_string(); // get the exact source of the error caused by the file opening io process
            error!("{:?}", source_error);
            let err = helpers::error0::PanelErrorResponse::from((
                source_error.as_bytes().to_vec(), 
                0, 
                helpers::error0::ErrorKind::File(e),
                String::from("")
            ));
            return Ok(err.error_response());
        }
    };


    // we can use ? operator since the From<std::io::Error> trait has implemented for the PanelError
    // runtime ERROR: cause file doesn't exist
    let f = std::fs::File::open("openme.txt")?; // ? returns http error response 

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