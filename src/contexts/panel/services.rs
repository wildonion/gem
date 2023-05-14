





use crate::*;



pub fn init(config: &mut web::ServiceConfig){

    config.service(apis::dev::index);
    
    // other routs
    // ...


}