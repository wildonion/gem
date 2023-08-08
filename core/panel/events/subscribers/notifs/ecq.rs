



/* 

    `ecq-{event_id}`

*/

use crate::events;

fn this(){
    let cq_instance: events::publishers::ecq::CollaborationQueue = Default::default();
    let cq = events::publishers::ecq::CollaborationQueue{..Default::default()}; // filling all the fields with default values 
}