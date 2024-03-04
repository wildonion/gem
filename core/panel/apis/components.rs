



/*  -------------------------
    access level based components, each component is an actor contains all the relative apis, 
    purpose of this design pattern is to facilitate the process of api communicating and 
    interaction without calling http request, therefore each api of the following components 
    can comunicate with each other by utilising the actor message passing logic as well as 
    doing pubsub push notif process using redis and actix borker, in essence if each component 
    would be a controller then with this design pattern we can communicate with each controller
    it's like calling smart contract methods from other contracts 
*/

pub mod admin;
pub mod user;
pub mod health;
pub mod public;