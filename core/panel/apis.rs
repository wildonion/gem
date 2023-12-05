



/*  > ---------------------------------------------------------------------------------------------
    | every api return type is Result<actix_web::HttpRequest, actix_web::Error>
    | that means if everyting goes well an api should return Ok(actix_web::HttpRequest)
    | in form utf8 bytes through the actix tcp socket to the caller or the client otherwise
    | the error part will be triggered in the server.
    |
    |   dev    ---> all apis with dev access
    |   admin  ---> all apis with admin access
    |   user   ---> all apis with user access 
    |   health ---> all apis related to server health
    |   public ---> all public apis
    |   role   ---> all websocket reveal role subscription apis with user access
    |   clp    ---> all websocket chatroom launchpad apis with user access
    |
    |
*/
pub mod dev;
pub mod admin;
pub mod user;
pub mod health;
pub mod public;
pub mod role;
pub mod clp;