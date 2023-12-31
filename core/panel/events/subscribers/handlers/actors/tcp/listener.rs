

use actix::prelude::*;
use actix::Actor;
use tokio::io::AsyncReadExt; // for reading from socket asyncly allows us to call .read() method
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt; // for writing to socket asyncly allows us to call .write_all() method
use log::{info, error};


pub struct TcpListenerActor{
    pub addr: String,
}

impl Actor for TcpListenerActor{
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context){

        info!("TcpListenerActor -> started listening");

        let (listener_sender, listener_reciever) = 
            std::sync::mpsc::channel::<TcpListener>();

        let address = self.addr.clone();
        tokio::spawn(async move{

            let api_listener = tokio::net::TcpListener::bind(address).await;
            listener_sender.send(api_listener.unwrap());
        });

        let received_listener = listener_reciever.recv().unwrap();
        self.listen(received_listener);
        
    }
    
}

impl TcpListenerActor{

    pub fn new(addr: &str) -> Self{
        TcpListenerActor{addr: addr.to_string()}
    }

    pub fn listen(&mut self, api_listener: TcpListener){

        tokio::spawn(async move{

            while let Ok((mut api_streamer, addr)) = api_listener.accept().await{

                tokio::spawn(async move {

                    /* this buffer will be filled up with incoming bytes from the socket */
                    let mut buffer = vec![]; // or vec![0u8; 1024] // filling all the 1024 bytes with 0

                    while match api_streamer.read(&mut buffer).await { /* streaming over socket to fill the buffer */
                        Ok(rcvd_bytes) if rcvd_bytes == 0 => return,
                        Ok(rcvd_bytes) => {
                
                            let string_event_data = std::str::from_utf8(&buffer[..rcvd_bytes]).unwrap();
                            info!("📺 received event data from peer: {}", string_event_data);

                            let send_tcp_server_data = String::from("write me into the socket");
                            if let Err(why) = api_streamer.write_all(&send_tcp_server_data.as_bytes()).await{
                                error!("❌ failed to write to api_streamer; {}", why);
                                return;
                            } else{
                                info!("🗃️ sent {}, wrote {} bytes to api_streamer", send_tcp_server_data.clone(), send_tcp_server_data.len());
                                return;
                            }
                        
                        },
                        Err(e) => {
                            error!("❌ failed to read from api_streamer; {:?}", e);
                            return;
                        }
                        
                    }{}
            
                });
            }{}
        });

    }

}