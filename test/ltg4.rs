



use crate::*;



async fn test(){

    trait InterfaceMe{}
    impl InterfaceMe for () {}


    pub struct Req;
    pub struct Res;
    let req = Req;
    let res = Res;
    pub struct Test<T, R: std::future::Future<Output=Res> + Send + Sync +'static> 
        where T: FnMut(Req, Res) -> R{
        pub f: T //// f is a FnMut closure which accepts Req and Res instances as its params 
    }
    let cb = |req, res| { async {res} /* the return type of the closure must be future object */};
    let instance = Test{f: cb};
    let response = (instance.f)(req, res).await;
    
    //// traits as a field (param) or return type must be behind a 
    //// pointer using Box or &dyn but as the type of a passed in 
    //// param the generic type of the param must be bounded to that trait.
    // 
    //// stack pinning can be a captured state of async block or 
    //// function which can be done using pin!{} which constructs 
    //// Pin<&mut T> and is cheaper than heap pinning or Box::pin()
    // fn run() -> impl Generator<Yield = i32, Return = ()>{} //// default type parameter
    // /// Runs a future to completion.
    // fn block_on<F: Future>(future: F) -> F::Output {
    //     let waker_that_unparks_thread = todo!();  
    //     let mut cx = Context::from_waker(&waker_that_unparks_thread);
    //     // Pin the future into the ram so it can be polled later whenever it gets ready
    //     let mut pinned_future = pin!(future);
    //     loop {
    //         match pinned_future.as_mut().poll(&mut cx) {
    //             Poll::Pending => thread::park(), //// block_on method will block the current thread by parking it
    //             Poll::Ready(result) => return result,
    //         }
    //     }
    // }


    pub async fn return_vec_of_box_traits<G>(c: 
            Box<dyn InterfaceMe + Send + Sync + 'static>, 
            //// if we want to use generic in rust we have to specify the generic name in function signature  
            //// since G is a closure that is bounded to FnMut we have to define it a mutable type 
            mut b: G) 
        -> Vec<Box<dyn InterfaceMe + Send + Sync + 'static>>
        where G: FnMut(u8) -> (){
 
            let mut n_c = 2; //// since the closure is bounded to FnMut thus we have to define teh cores as mutable since it'll get a mutable borrow
            b(n_c); //// we're calling the closure here and pass the mutable n_c param
            let mut vector_of_boxed = vec![];
            vector_of_boxed.push(c);
            vector_of_boxed

    } 
    //// first param of the `return_vec_of_box_traits` function
    //// is a type that accepts a Box of `InterfaceMe` trait 
    //// whence the `InterfaceMe` trait is implemented for () or
    //// the empty type, we can create a Box of () or Box::new(())
    //// and pass it as the first param, for the second param 
    //// we've passed a closure with empty return body
    //
    //// we'll pass the u8 value when we're calling the 
    //// closure but we can use it here and store it in 
    //// cores variable
    return_vec_of_box_traits(Box::new(()), |cores|{ 
        println!("number of cores is : {}", cores);
    }).await;


    //----------------------------
    let clsMe = |name: String| { //// we can also put the closure body inside a curly braces

        let mut val = "wildonion".to_string(); /* creating longer lifetime by binding the val into let */
        let mut boxed = Box::pin(&mut val);
        let ref_ = &mut boxed;
       
        Box::pin(async {
            name
        })
    };
    let clsMe = |name: String| Box::pin(async{ //// since the return type is a Pin<Box<T>> there is no need to put the Box::pin inside curly braces since it's a single line code logic
        name
    });
    //----------------------------

    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    struct Round{
        val_idx: u8,
        values: Vec<u8>
    }
    let announced_values: Vec<Round> = vec![];
    
    /* 
        closures can capture env vars so we can access them inside the closure method, with 
        function we can't do that, since functions have their own scopes, we could either pass 
        the type by value if we don't need its ownership (specially for heap data) or reference 
        if we don't want to lose its ownership inside the caller scope of the method also to mutate 
        the content of the type inside the function without mutating the actual type we 
        must pass a mutable reference to it like for mutating announced_values we must pass 
        the mutable reference to announced_values type to the is_duplicate_fn function, 
        since by mutating the mutable pointer of the main type the actual type will be mutated too, 
    */

    fn is_duplicate_fn(val: u16, val_idx: u16, announced_values: &mut Vec<Round>) -> bool{
        for av_idx in 0..announced_values.len(){
            if (announced_values[av_idx].values[val_idx as usize]) as u16 == val{
                return true;
            } else{
                return false;
            }
        }
        return false;
    }

    /*
        the following closure will borrow and capture the result_announced_values var
        as immutable, thus we can't push into the result_announced_values vector later
        on if we're going to use this method, since rust doesn't allow to borrow the 
        type as mutable if it's borrowed as immutable already in a scope, instead we 
        can use FnMut closure to capture vars mutablyو also announced_values must be 
        initialized in order the closure to be able to capture it into its env
    */
    let is_duplicate = |val: u16, val_idx: u16|{
        for av_idx in 0..announced_values.len(){
            if (announced_values[av_idx].values[val_idx as usize]) as u16 == val{
                return true;
            } else{
                return false;
            }
        }
        return false;
    };
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    /* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */
    
    clsMe("wildonion".to_string());
	
	
    pub struct Complex{
        pub callback: Box<dyn FnOnce(Option<String>) -> u8>,
        pub labeled_block: bool,
        pub long_block: Option<u8>,
        pub callback_result: u8,
    }
    
    let comp = Complex{
        callback: Box::new(
            |_: Option<String>| 13
        ),
        labeled_block: 'block:{
            if 22 % 2 == 0{
                break 'block true; // it'll break the 'labeled block with a true return
            } else{
                break 'block false; // it'll break the 'labeled block with a false return
            }
        },
        long_block: {
            let mut x = 0;
            while 2 % x > 2{
                x+=1;
            }
            let somed = Some(x);
            match somed{ // we must cover all the match arms if we have if in one of the arm 
                Some(n) if n == 2 => Some(n as u8), // in those case that n must be 2 
                Some(n) => Some(n), // if this arm was the first arm then above arm will be unreachable since this arm has no condition in it thus definitely will be the matched one
                None => None
            }
        },
        callback_result: ( // building and calling the closure at the same time inside the struct field
            |_| 254
        )(Some("wildonion".to_string())),
    };

    //// if let unpacking
    // if let Complex{ 
    //     callback, 
    //     labeled_block,
    //     long_block,
    //     callback_result 
    // } = comp{
    // 	println!("unpacking is ok!");
    // }
    //// let else example
    let Complex{ 
        callback, 
        labeled_block,
        long_block,
        callback_result 
    } = comp else{ // the else part is not needed since the unpacking process will be matched always
        panic!("can't unpack");
    }; // struct unpacking


    // let Complex{..} = com else{ // .. means all the fields
    //     panic!("can't unpack");
    // };

    pub async fn do_it<F>(callback: F) // callback is of type F
        -> u8 where 
                F: FnOnce(Option<String>) -> u8 + Send + Sync + 'static
        { // where F is a closure which is bounded to Send Sync traits and have a valid static lifetime
        callback(Some("wildonion".to_string())) // by calling the passed in closure we can have the u8 as the result of calling which must be returned from this function
    }
    do_it(|name|{
        let Some(some_u8_number) = Some(24) else{
            panic!("can't get out of Some");
        };
        some_u8_number // the some_u8_number scope is still valid in here and we can return
    }).await;
    
    ( // building and calling the closure at the same time; the return type of this closure is a future which must be awaited later on
        |age| async move{ 
            age
        }
    )(32).await;

    

    let names =  //// building and calling the async closure at the same time
        (|x| async move{ //// the return body is a future object which must be solved later using .await and move will move everything from the last scope into its scope  
            let names = (0..x)
                .into_iter()
                .map(|index|{
                    let name: String = "wildonion".to_string();
                    name
                })
                .collect::<Vec<String>>();
            names
        })(23).await;



    let statement = |x: u32| Some(2);
    let Some(3) = statement(3) else{ // in else part there must be panic message
        panic!("the else part");
    };

    // a function is created using () also
    // calling a function is done by using ()
    // thus by using ()() we're building and calling
    // the function at the same time
    let res = { // res doesn't have any type
        ( // building and calling at the same time inside the res scope
            |x| async move{
                x
            }
        )(34).await; 
    };


    // nodejs like function call
    fn sayHelloAgain<C>(call: u8, callback: C) // C is the callback type which is a FnOnce trait
        where C: FnOnce(Option<u8>, HashMap<String, String>){
        callback(None, HashMap::new());
    }


    sayHelloAgain(23, |n_c, m|{
        let inputs: Vec<Vec<f64>> = vec![vec![5.6, 5.3]];
        for index in 0..inputs.len(){
            let row = &inputs[index]; //// inputs in the first iteration will be moved from the memory thus we have to borrow it or clone it
        }
        let map = m;
        let none_call = n_c;

        pub struct Nft{
            pub id: u16,
            pub title: String,
            pub royalties: Vec<Royalty>,
        }
        pub struct Royalty{
            pub receiver: String,
            pub amount: u128,
        }
        let nfts: Vec<Nft> = Vec::new();
        nfts.into_iter().map(|nft| {
            for r in nft.royalties{
                let who = r.receiver;
                let much = r.amount;
            }
        });
    });

    let callback = |_| Some(1); // |_| means that the param name can be anything  
    let (
        |callback| callback // the return type is the callback which is a closure
    ) = match callback(..){ // callback(..) means that it'll take anything as the range - when we're do a matching on the callback(..) means that by calling the callback(..) we should get a closure in its return type which this is not the case hence this code is unreachable 
        |_| Some(2) => |_| Some(3), // |_| Some(2) is the other syntax for calling the x closure - the or pattern: it can also be _ | Some(2) since _ means the value can be anything thus one of side can only be executed (either _ or Some(2))  
        |_| _ => unreachable!(), // |_| _ is the other syntax for calling the x closure - the or pattern: it can also be _ | _ since _ means the value can be anything thus one of side can only be executed (either _ or _)
    };
    // the return type of calling callback(..) is not a closure hence we can't do a matching on closures and because of that the code will be unreachabled
    assert!(matches!(callback(..), |_| Some(4))); // it'll be unreachable since the first arm of the match is not match with this
    
}