





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

pub fn init_user(config: &mut web::ServiceConfig){
    
    config.service(apis::health::index);
    
    // other routs
    // ...


}

pub fn init_health(config: &mut web::ServiceConfig){
    
    // other routs
    // ...


}

pub fn init_mmq(config: &mut web::ServiceConfig){
    
    // other routs
    // ...


}