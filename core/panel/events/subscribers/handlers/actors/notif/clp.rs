

    

/* 

  see pg.rs

  pointers (&mut, shared ref, rc, arc, mutex, refcell, rwolock, as ref, can't move, can't ret)
  traits (box &dyn, closures, impl Trait, extend the behavior of types)
  generics (bounding to traits and lifetimes)
  enum, ram concepts, stack, heap, cap, length, pointer 
  static lazy arc mutex rwlock map as an in memory thread safe database using multithreading and streaming concepts with tokio::spawn,mutex,rwlock,mpsc,select,tcp
  rusty ltg(&mut,box,pin,trait,macro,generic) and shared state data to make them shareable between routes and threads using Arc<Mutex<Db>> + Send + Sync + 'static with mpsc
  arc will be used to share the pointer of the type between threads safely cause it's a atomically-reference-counted shared pointer

  trigger, publish, fire, emit event means that we'll send a packet on an specific condition to a channel
  so subscribers can subscribe to and stream over that packet (mpsc receiver, file, mulipart, payload, tcp based packets) 
  using actor, tokio::spawn, while let some to fill the buffer then decode and map to struct or serde json 
  value and from other parts send message to the actor to get the decoded data
  tcp based tlps: sqlx and tokio rustls wallexerr, actix actor,ws,http, redis hadead, tonic, tokio

*/

use crate::*;


struct Struct<'valid, G>{
  pub data: &'valid G
}

impl<'g, G: Clone + Default + Send + Sync + 'static> Event for Struct<'g, G>{
  type Room<'valid> = std::sync::Arc<tokio::sync::Mutex<G>>;

  fn get_room<'valid>(&mut self) -> Self::Room<'valid> {

      fn get_name() -> String{ String::from("") }
      let callback = |func: fn() -> String|{
          func();
      };
      callback(get_name);
      
      let d = self.data.clone();
      std::sync::Arc::new(
          tokio::sync::Mutex::new(
             d 
          )
      )
  }
}

trait Event{
  type Room<'valid>: 
  'valid + ?Sized + Default + Clone + Send + Sync + 'static; // we can bound the Room GAT to traits in here
  
  fn get_room<'g>(&mut self) -> Self::Room<'g>;

}

// let mut struct_instance = Struct::<String>{
//   data: &String::from("")
// };
// let thetype = struct_instance.get_room();