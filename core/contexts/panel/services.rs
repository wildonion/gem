





use crate::*;



/*
     --------------------------------
    |        REGISTER DEV ROUTES
    | -------------------------------
    |
    |

*/
pub fn init_dev(config: &mut web::ServiceConfig){

    config.service(apis::dev::reveal_role);
    config.service(apis::dev::index);
    
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
    
    config.service(apis::admin::index);

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
    
    config.service(apis::health::index);
    
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