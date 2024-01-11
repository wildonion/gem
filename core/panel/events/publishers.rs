



/* 
    >----------------------------------------------------------------------------------------------
    | publishers are writers who will send data notif to the specified 
    | channel (tcp, rpc, mpsc) so subscribers be able to catch them 
    | by subscribing to them so they can parse and decode the data 
    |

    followings are redis publisher actors using redis 
    actix actor to publish/fire/emit/broadcast topics and notifs 
*/

pub mod role;
pub mod mmr;
pub mod pg;
pub mod user;