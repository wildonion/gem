


use crate::*;



#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Notif{
    /*
                        EVENTS

        `tasks`, `task-verification-responses`, 
        `twitter-bot-response`, `ecq-{event_id}`, 
        `mmr-{event_id}`, 
        `reveal-role-{event_id}`
    */
    pub event: String,
    pub emitted_at: i64,
}

impl Actor for Notif{
    type Context = ws::WebsocketContext<Notif>;
}

/* implementing the event handler for the Notif actor */
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Notif{

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        
        match msg{
            Ok(ws::Message::Text(text)) => {

                /*
                    it is possible to convert a request's Payload to a stream of ws::Message with 
                    a web::Payload and then use stream combinators to handle actual messages
                */

                self.event = text.to_string();
                
                // todo - 
                // schedule a message to be executed in other actors using tokio cronjob, loop{} with tokio time and actors 
                // loop { reids subs to topics then send ws response to the client } 

                ctx.text(text)
                
            },
            _ => (),
        }
    }

}