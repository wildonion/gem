





use crate::*;



/*
     --------------------------------
    |        REGISTER DEV ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_dev(config: &mut web::ServiceConfig){

    config.service(apis::dev::exports::reveal_role);
    config.service(apis::dev::exports::login);
    
    // other routs maybe ?
    // ...


}

/*
     --------------------------------
    |      REGISTER ADMIN ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_admin(config: &mut web::ServiceConfig){
    
    config.service(apis::admin::exports::login);
    config.service(apis::admin::exports::register_new_admin);
    config.service(apis::admin::exports::register_new_task);

    // other routs maybe ?
    // ...


}

/*
     --------------------------------
    |       REGISTER USER ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_user(config: &mut web::ServiceConfig){
    
    config.service(apis::health::exports::index);
    
    // other routs maybe ?
    // ...


}

/*
     --------------------------------
    |     REGISTER HEALTH ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_health(config: &mut web::ServiceConfig){
    
    // other routs maybe ?
    // ...


}

/*
     --------------------------------
    |       REGISTER MMQ ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_mmq(config: &mut web::ServiceConfig){
    
    // other routs maybe ?
    // ...


}