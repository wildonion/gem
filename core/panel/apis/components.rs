



/*  -------------------------
    access level based components, each component is an actor contains all the relative apis, 
    purpose of this design pattern is to facilitate the process of api communicating and 
    interaction without sending http request, therefore each api of the following components 
    can comunicate with each other by utilising the actor message passing logic as well as 
    doing pubsub push notif process using redis and actix borker, in essence if each actor would 
    be a controller then with this design pattern we can communicate with each controller remotely 
    it's like calling smart contract methods from other contracts, in a high insight every api 
    node actor can communicate with each other inside a cluster through rpc p2p gossipsub and 
    redis pubsub to aware each others of joining new nodes


         __________________________________ CLUSTER ____________________________________
        |                                                                               |
  admin component node actor                                                    user component node actor 
                |                                                                           |
                 ----------                                                       ---------- 
                           |                                                     |
                            --------remotely------- || --------locally ----------
                                        |                         |
                                        |                         |
                                        |                         |
                          rpc,p2pgossipsub,redispubsub        broker,mpsc
*/  

pub mod admin; // admin component node actor 
pub mod user; // user component node actor 
pub mod health; // health component node actor 
pub mod public; // public component node actor 