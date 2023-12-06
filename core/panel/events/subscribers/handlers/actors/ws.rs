


/*   
    session actor: 
          an actor that will handle ws streams from client
          also it has message struct handler to communicate
          with server actor or other session actors
     server actor: 
          an actor that send message to session actors through
          actor message passing pattern using message structs 
*/
pub mod servers;
pub mod sessions;