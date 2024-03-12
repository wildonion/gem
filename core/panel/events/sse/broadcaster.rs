


// https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/broadcast.rs
// https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/main.rs


// add broadcaster struct to app state so we can share it between threads
// broadcast new clp event

#[derive(Debug, Clone, Default)]
pub struct Broadcaster{ 

}

impl Broadcaster{

    pub fn new() -> Self{

        todo!()
        
    }
    
    pub async fn add_client(){

    }

    pub async fn broadcast(){
         
    } 
    
}