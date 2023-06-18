


use crate::*;


async fn test(){


    pub struct Queue<T> where T: FnMut() -> (){
        pub task: T
    }

    let mut queues = vec![];
    for i in 0..10{
        queues.push(Queue{
            task: ||{

            } 
        });
    }

// ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++   
    struct Nft;
    struct Account<'info, GenericData>{
    pub account: GenericData,
    pub name: &'info str,
    }

    impl<'info, GenericData> Account<'info, GenericData>{

    pub fn run(&self) -> &[u8]{
        let a: &[u8] = &[1];
        a
    }

    }

    let account = Account::<'_, Nft>{account: Nft, name: "wildonion"};
    let arr = account.run();



    const _: fn() = || {
    fn run(){}
    };


    struct Nft5;
    struct Instance5<G>{
        pub nft: G,
    }
    fn run_with_5<G>(nft: G) -> Instance5<G>{

        let instance = Instance5::<G>{nft};
        instance
    }

    run_with_5(Nft);

    //-------------------------------------
    //-------------------------------------
    //-------------------------------------
    struct Process{cpu_core: u8}
    let task = async{
        let main = |pid| async move{
            Process{cpu_core: pid as u8}
        };
        // unapacking the fetched process from the main() 
        let pid = 10;
        match main(pid).await{
            Process{cpu_core} if cpu_core > 2 =>{
                4u8
            },
            Process{cpu_core} if cpu_core == 0 =>{
                0u8
            },
            _ => {
                10u8
            }
        }
    };
    task.await; // or block_on(task)
    // rust don't have gc thus by moving the type into 
    // other scopes i'll be dropped, we can either clone 
    // them or borrow them to move their pointers, for case 
    // of future objects in order to move them as a type 
    // between other threads we must pin them into the ram 
    // and since they are future objects which are of type traits 
    // and traits have no known size at compile time thus we have 
    // to pin their pointer into them ram and their pointers be like 
    // Box<dyn Future<Output=u8>> or &dyn Future<Output=u8>
    // also they can be returned from the function using 
    // ... -> impl Future<Output=u8> style in function signature
    //////
    // or inside the Box<Future<Output=u8>>
    // to pin the future objects into the ram 
    // we should pin their box into the ram 
    // since they are traits and traits must be
    // in their borrowed form which can be done
    // using Box<dyn Trait> or &dyn Trait.
    fn runner() -> impl std::future::Future<Output=u8>{
            
        async{
            23
        }
        
    }
    fn _run() -> Box<dyn std::future::Future<Output=u8>>{
            
        Box::new(async{
            23
        })
        
    }
    //-------------------------------------
    //-------------------------------------
    //-------------------------------------

    pub const N: usize = 4;
    struct Response;
    struct Api;
    impl Api{
        
        pub fn get_user(path: String) -> Response{
            Response{}
        }
        
        pub fn add_user(path: String) -> Response{
            Response{}
        }
        
        pub fn add_nft(path: String) -> Response{
            Response{}
        } 
        
        pub fn get_nft(path: String) -> Response{
            Response{}
        } 
        
    }


    // arrays cannot have values added or removed at runtime; 
    // their lengths are fixed at compile time thus we've defined
    // a fixed size of N for Apis type.
    // if the array has a size then there is no need to use it
    // behind a pointer since it's size will specified at compile time
    // otherwise it's must be behind a pointer which will be a slice
    // of vector since all dynamic sized types will be coerced to 
    // their slice form at compile time.
    type Apis<'p> = [fn(String) -> Response; N]; //// adding N api function inside the array, we don't need to put it behind a pointer since it has a fixed size at compile time
    let apis: Apis;
    apis = [
        Api::get_user,
        Api::add_user,
        Api::add_nft,
        Api::get_nft
    ];
    
    
    // closures are of type traits hence they are dynamic
    // abstract size and in order to use them as a type
    // we have to put them behind a reference like Box<dyn
    // or &dyn
    // let regiser_trait = |apis: &[&dyn FnMut() -> Response]|{ //// vector of FnMut closure apis which is of type trait
    //     for api in apis{
    //         api("some-path");
    //     }
    // };
    
    let regiser_fn = |apis: [fn(String) -> Response; 4]|{ //// vector of function apis with a fixed size of N elements
        for api in apis{
            api("some-path".to_string());
        }
    };
    
    regiser_fn(apis);
    

// ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++   

}