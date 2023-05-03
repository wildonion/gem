


use hyper::Request;
use crate::misc;
use crate::constants::*;
use crate::resp; //// this has been imported from the misc inside the app.rs and we can simply import it in here using crate::resp
use futures::{executor::block_on, TryFutureExt, TryStreamExt}; //// futures is used for reading and writing streams asyncly from and into buffer using its traits and based on orphan rule TryStreamExt trait is required to use try_next() method on the future object which is solved by .await - try_next() is used on futures stream or chunks to get the next future IO stream and returns an Option in which the chunk might be either some value or none
use bytes::Buf; //// it'll be needed to call the reader() method on the whole_body buffer and is used for manipulating coming network bytes from the socket
use hyper::{header, StatusCode, Body, Response};






// -------------------------------- not found controller
// ➝ Return : Hyper Response Body or Hyper Error
// -------------------------------------------------------------------------

pub async fn register_notif(req: Request<Body>) -> ConseResult<hyper::Response<Body>, hyper::Error>{ //// the return type is hyper response


    // TODO - fetch redis data from its server 
    // ...

    resp!{
        misc::app::Nill, //// the data type
        misc::app::Nill(&[]), //// the data itself
        NOT_IMPLEMENTED, //// response message
        StatusCode::NOT_IMPLEMENTED, //// status code
        "application/json" //// the content type 
    }


}