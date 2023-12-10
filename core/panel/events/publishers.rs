



/* 
    >----------------------------------------------------------------------------------------------
    | publishers are writers who will send data notif to the specified 
    | channel (tcp, rpc, mpsc) so subscribers be able to catch them 
    | by subscribing to them so they can parse and decode the data 
    |

    followings are redis publisher actors using redis 
    actix actor to publish topics and notifs 
*/

pub mod ecq;
pub mod role;
pub mod mmr;
pub mod pg;