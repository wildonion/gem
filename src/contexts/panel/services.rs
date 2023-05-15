





use crate::*;



pub fn init_dev(config: &mut web::ServiceConfig){

    config.service(apis::dev::index);
    
    // other routs
    // ...


}


pub fn init_admin(config: &mut web::ServiceConfig){
    
    // other routs
    // ...


}