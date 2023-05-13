

use crate::*;



#[derive(Serialize, Deserialize, Debug)]
pub struct Response<'m, T>{
    pub data: Option<T>,
    pub message: &'m str, //// &str are a slice of String thus they're behind a pointer and every pointer needs a valid lifetime which is 'm in here 
    pub status: u16,
}


/*
    we can define as many as response object 
    since once the scope or method or the match
    arm gets executed the lifetime of the response
    object will be dropped from the ram since rust 
    doesn't have gc :) 
*/
#[macro_export]
macro_rules! resp {
    (
        $data:expr,
        $msg:expr,
        $code:expr,
    ) => {

        {
            use actix_web::HttpResponse;
            use crate::misc::Response;
            
            let code = $code.as_u16();
            let mut res = HttpResponse::build($code);
            
            let response_data = Response{
                data: Some($data),
                message: $msg,
                status: code
            };
            
            let response = Ok(
                res
                    .json(
                        response_data
                    )
            );
            return response;
        }
    }
}