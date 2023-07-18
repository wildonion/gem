



/* 

    `ecq-{event_id}`

*/

use crate::events;

fn this(){
    let cq_instance: events::redis::ecq::CollaborationQueue = Default::default();
    let cq = events::redis::ecq::CollaborationQueue{..Default::default()}; // filling all the fields with default values 
}