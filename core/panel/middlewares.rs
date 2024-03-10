


/* ------------------------------ */
//      middleware plugins
/* ------------------------------ */
/* 
    a Service in middleware represents anything that takes a request and returns a response
    a middleware is a service that has a request, response error and result in form of future object
    it takes any request of any form and processes it then to change the flow of app before sending 
    any response then call the poll_ready() to to determine if the service is ready to be invoked 
    finally we can inspect or mutate the request and response objects as needed inside the call() 
    method, generally middleware functions can perform the following tasks: execute any code, make 
    changes to the request and the response objects, also the next() next function is a function in 
    when invoked, executes the middleware succeeding the current middleware.
*/
pub mod http;