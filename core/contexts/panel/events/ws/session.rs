


use crate::constants::{WS_CLIENT_TIMEOUT, SERVER_IO_ERROR_CODE};
use crate::{misc::*, constants::WS_HEARTBEAT_INTERVAL};
use crate::*;
use actix::prelude::*;
use super::notifs::{
    role::{
        Message as RoleMessage, 
        RoleNotifServer, Disconnect as RoleNotifServerDisconnectMessage
    }
};




#[derive(Clone, Debug)]
pub(crate) struct WsNotifSession{
    pub id: usize, // unique session id
    pub hb: Instant, // client must send ping at least once per 10 seconds (CLIENT_TIMEOUT), otherwise we drop connection.
    pub notif_room: String, // user has joined in to this room 
    pub peer_name: Option<String>, // user id
    pub ws_role_notif_actor_address: Addr<RoleNotifServer> // the role notif actor server
}


impl WsNotifSession{

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>){ /* ctx also contains the instance of the WsNotifSession struct */
        /* actor is the WsNotifSession which can be accessible inside the closure */
        ctx.run_interval(WS_HEARTBEAT_INTERVAL, |actor, ctx|{
            if Instant::now().duration_since(actor.hb) > WS_CLIENT_TIMEOUT{
                
                info!("websocket client heartbeat failed, disconnecting!");
                actor.ws_role_notif_actor_address.do_send(RoleNotifServerDisconnectMessage{id: actor.id});
                ctx.stop(); /* stop the ws service */

                return;
            }
        });
        ctx.pong(b""); /* sending empty bytes back to the peer */
    }
}


impl Actor for WsNotifSession{
    type Context = ws::WebsocketContext<WsNotifSession>;
}

/* handle messages from RoleNotifServer, we simply send it to peer websocket */
impl Handler<RoleMessage> for WsNotifSession {
   
    type Result = ();

    fn handle(&mut self, msg: RoleMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/* event listener or streamer to receive ws message */
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsNotifSession{

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

        let msg = match msg{
            Ok(msg) => msg,
            Err(e) => {

                /* custom error handler */
                use error::{ErrorKind, ServerError::{ActixWeb, Ws}, PanelError};
                let msg_content = [0u8; 32];
                let error_content = &e.to_string();
                msg_content.to_vec().extend_from_slice(error_content.as_bytes());

                let error_instance = PanelError::new(*SERVER_IO_ERROR_CODE, msg_content, ErrorKind::Server(Ws(e)));
                let error_buffer = error_instance.write_sync(); /* write to file also returns the full filled buffer */

                ctx.stop();
                return;
            }
        };

        match msg{
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            },
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            },
            ws::Message::Text(text) => {


                todo!()

            },
            ws::Message::Binary(_) => info!("unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }

    }

}