


use crate::*;



async fn test(){

    trait Interface{}


    // **************************************************************************
    // **************************************************************************
    // **************************************************************************
    // --------------------------------------------------------------------------
    ///////////////////////// CLOSURE TYPES EXAMPLE /////////////////////////////
    // we cant only use impl Trait syntax in function return type
    // calling the boxed closure traits by unboxing or dereferencing the boxed value
    // --------------------------------------------------------------------------
    let workers = 10;
    type Job = Box<dyn Fn() -> () + Send + Sync>;
    for worker in 0..workers{
        let job: Job = Box::new(||{});
        let thread = std::thread::spawn(move ||{
            // job()
            (*job)() //// job ca be a socket connection
        });
    }
    struct RunnerTar<C=std::pin::Pin<Box<dyn Interface>>>{
        data: C ////
    }
    //// since we have a closure inside the Box which is of 
    //// type trait, thus we can call it in a different ways
    //// like the following
    //
    //// if we have closure inside the Box since Box is a 
    //// pointer we can deref it first like (*boxed)() 
    //// then call it or we can call it directly like boxed() 
    let mut d_boxed = Box::new(||{
        let name = String::from("wildonion");
        name
    });
    //// we can call the d_boxed since the closure trait
    //// is behind a pointer which is Box in our case thus 
    //// we can call the closure directly by calling the 
    //// d_boxed type, means we're calling the Boxed type
    //// which is a pointer to a closure
    d_boxed(); 
    //// since d_boxed is a Boxed type which is a pointer 
    //// to a heap data which is a closure tratit (traits are ?Sized) thus in order
    //// to call the closure directrly we can deref the Boxed type 
    //// then call the closure. 
    (*d_boxed)();
    //// as_mut() will convert the d_boxed() into 
    //// the type inside the Box which is &mut dyn FnMut(String) -> String
    //// then we can call the trait using ()
    d_boxed.as_mut()(); 

    // we can't have Pin<Box<impl Future<Output = i32>>> 
    // the impl Timpl Trait will be added by compiler 
    // - return traits from method using -> impl TraitLikeClosure, Box<dyn Trait> or &'valid dyn Trait which implements trait for the return type
    // - use traits like closures in struct field using where or Box and method param using impl Trait
    // - pin async block into the ram so we can await on it in future 
    // - since async blocks are future objects we must put them behind a pointer thus we must pin the boxed future object
    // - also the future object must be valid across threads thus we must bound the Future object to Send + Sync + 'static
    let dejavo = Box::pin(async move{
        32
    });

    dejavo.await;


    struct Link<'link, D, F: Send + Sync> 
    where D: Send + Sync,
    F: FnOnce() -> String{
        pub data: F,
        pub link: &'link D
    }
    impl<D: Send + Sync, F: Send + Sync> Link<'_, D, F> 
        where F: FnOnce() -> String{
        
        fn run() -> impl FnMut() -> u32{
            ||{
                {
                    let item_remaining = 32; 
                    item_remaining
                }
            }
        }

        /* we can impl traits for the method param and bound its type to that trait */
        fn start(cmd: impl FnOnce() -> ()){
            cmd();
        }

        fn another_start_here(cmd: impl Fn(String) -> ()){
            
        }

        /*  

            if we want to take a mutable pointer to a future object then the future object 
            must implements the Unpin trait, because pinning will lock the object into the ram
            and makes its location stable in there and doesn't allow to move or mutate that object
            also it doesn't allow to obtain Box<T> or &mut T thus we must unpin the object first
            the following is the general explanation of this:

            By default, all types in Rust are movable. Rust allows passing all types by-value, 
            and common smart-pointer types such as Box<T> and &mut T allow replacing and moving 
            the values they contain: you can move out of a Box<T>, or you can use mem::swap. 
            Pin<P> wraps a pointer type P, so Pin<Box<T>> functions much like a regular Box<T>: 
            when a Pin<Box<T>> gets dropped, so do its contents, and the memory gets deallocated. 
            Similarly, Pin<&mut T> is a lot like &mut T. However, Pin<P> does not let clients 
            actually obtain a Box<T> or &mut T to pinned data, which implies that you cannot use 
            operations such as mem::swap


            future objects are traits and can be implemented for other types to convert those
            types into a future objects, also they can return any type as their result of solving
            process by default rust moves heap data types when we go to new scopes unless we borrow 
            them thus for future objects which are of type traits and traits are heap data types 
            we must pin their Box into memory since we don't know when they will be solved and 
            where the scope will be hence, we must bound the pointer of the pinned object to the 
            Unpin trait if we want to move it or take a pointer to it 

            we can get the result of impl Future<Output=String> simply by awaiting on the future 
            object but we can't await on an immutable pointer to a future object or &impl Future<Output=String>
            because :
                Unpin can makes type to be moved safely after being pinned for example if we want to 
                use mem::replace on T which has been pinned into the ram, the T must be bounded to 
                Unpin, also mem::replace works for any !Unpin data and any &mut T not just when T: Unpin
                in essence we can't use mem::replace on a pinned object like the pointer of future objects
                cause we can't get &mut T from it which means we must unpin the future object first then
                put it behind a mutable reference finally we can use mem::replace over that, remember that 
                using mem::replace over types requires that both of them be behind a &mut pointer since 
                this method will replace the location of both types in ram which a mutable process, 
                based on above notes, awaiting on an immutable pointer to a future object requires some 
                kinda replacing and moving operations which forces the object to behind a mutable pointer
                to it's type and be bounded to Unpin trait if we need to access the pinned value outside 
                of the current scope that we're awaiting on the object 

        */
        async fn create_component(async_block: &mut (impl futures::Future<Output=String> + std::marker::Unpin)){
            
            let a = async_block;
            a.await;

            trait HealCheck{
                fn healtcheck(&self){}
            }
            async fn do_health_check(hc: impl HealCheck + Send + Sync + 'static){
                hc.healtcheck(); /* we can call the HealthCheck trait method since the hc is bounded to this trait already */
            }
         
         
            let nature = async{
                'out:{
                    break 'out ();
                }

                'outer: loop{ // outter labeled block 
                    println!("this is the outer loop");
                    'inner: loop{ // inner labeled block 
                        println!("this is the inner loop");
                        // break; // only the inner loop
            
                        break 'outer;
                    }
            
                    println!("this print will never be reached"); //// this is an unreachable code
                }
            
            
                'outer: for x in 0..5 {
                    'inner: for y in 0..5 {
                        println!("{},{}", x, y);
                        if y == 3 {
                            break 'outer;
                        }
                    }
                }
            };

        }

        /* ------------------------------------------ 
            in the following example:
            we can't use impl Trait in function param 
            since we have to give an explicit type to the 
            describe which rust doesn't accept impl Trait 
            in fn() pointer param, compiler will set the type 
            of describe to fn col<impl Trait>() later
        ------------------------------------------ */

        const fn resize() -> i32{
            32
        }
        pub const FUNC: fn() -> i32 = {
            const fn resize() -> i32{
                32
            }   

            resize
        }; 

        fn callmehore(){
            fn colided<Z>(cls: Z) -> () 
            where Z: FnOnce(String) -> ()
            {
                let name = "wildonion".to_string();

                let callback = cls(name);
                let res = match callback{
                    () => false, /* matches any value this will be matched since the return type of closure is () */
                    _ => true, /* _ means any value */
                    |_| () => true, /* |_|() matches any result of calling the callback */
                    |_| () | _ => false, /* matches any result of calling the callback or _ */
                    _ | _ => true /* matches any value or any value */
                };
            }
            
            let describe = colided;
            describe(|name: String|{
                ()
            });

            /* 
                bounding the param type to closure trait directly 
                without using where but by using impl Trait, 
                in this case we didn't store the bolided into 
                a new type thus we can call it directly with 
                no problem
            */
            fn bolided(cls: impl FnOnce(String) -> ()){
                let name = "wildonion".to_string();
                cls(name);
            }
            bolided(|name: String|{
                ()
            });

        }

        //////-------------------------------------------------------------------------------
        ////// we can't return impl Trait as the return type of fn() pointer or as its param
        //////-------------------------------------------------------------------------------
        // type Function = fn() -> impl futures::Future<Output=String>;
        // async fn create_component_method<L>(async_block: fn() -> impl futures::Future<Output=String>) {
        //     async_block().await;
        // }
        //////--------------------------------------------------------------
        ////// we can't return impl Trait as the return type of traits 
        //////--------------------------------------------------------------
        // async fn create_component_method<L>(async_block: L) where L: Fn() -> impl futures::Future<Output=String>{
        //     async_block().await;
        // }
        //////--------------------------------------------------------------
        ////// we can't .await after calling async_block() since future objects 
        ////// in Box must be pinned to the ram to be valid across scopes for later solves
        //////--------------------------------------------------------------
        // async fn create_component_method<L>(async_block: L) where L: Fn() -> Box<dyn futures::Future<Output=String>>{
        //     let res = async_block();
        // }
        //////--------------------------------------------------------------
        ////// generic type L is a trait which its return type is an async block, the reason
        ////// we put the future object inside a pinned boxed is because we can't simply return
        ////// a trait as the return type of another trait thus we have to put it inside the Box
        ////// or behind a valid pointer like &'valid dyn Trait (because traits are dynamic sized types)
        ////// also in order to solve the boxed future after calling the async_block trait, the future 
        ////// object must be valid across scopes since we don't know the exact place of its solving 
        ////// which it might be inside different scopes other than where it has initialized and because
        ////// of this reason we must also pin the pointer of the future object into the ram to prevent 
        ////// its location from moving (or replacing by another type) until we await on it to get the result
        ////// since once we move into other scopes its lifetime will be checked by the borrow checker
        ////// and if it has already pinned the rust can't move it and drop its lifetime
        /// 
        ////// since future objects are traits and traits don't have fixed size (since they're heap data types) thus we must
        ////// put them inside the Box and pin that Box to the ram, by pinning them we can go to other scopes without losing 
        ////// their ownership (since the're pinned to ram and can't be moved) and await on the pinned boxed future whenever
        ////// and where ever we want 
        //////--------------------------------------------------------------
        async fn create_component_method<L>(async_block: L) where L: Fn() -> std::pin::Pin<Box<dyn futures::Future<Output=String>>>{
            let res = async_block().await;
        }
    }

    // ||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
    struct Useram{
        pswd: String
    }
    impl Useram{
        pub fn get_token(&self) -> &str{
            "token"
        }
    }
    let user = Useram{pswd: "wildonion".to_string()};
    /* 
        since we're putting the return type of the Box which is
        another Box contains a future object inside the Pin thus 
        all the types inside the second Box must be live long anough
        and be valid across .await until the value gets pinned from
        the ram. we can solve this by moving the that type into the 
        async block or the future object. 
    */
    let token: Box<dyn FnOnce() -> 
        Arc<std::pin::Pin<Box<dyn std::future::Future<Output=String> + Send + Sync + 'static>>> //// a shared pinned box object 
            + Send + Sync + 'static> = 
            Box::new(|| Arc::new(Box::pin(
                    /* 
                        by using move keyword we can move the user into this scope so it can 
                        have a valid lifetime across .await since the following async block will
                        be pinned into the ram which all the types inside the async block must be 
                        valid until the future object gets unpinned.  
                    */
                    async move{
                        user.get_token().to_string()
                    }
                ))
            );
    /* 
        here we can't deref the token object to call it 
        and it MUST be remained behind a pointer since 
        by derefing it we'll get a trait object which 
        has defined by ourselves in the code, not the 
        compiler, that it's size is not known at compile 
        time thus we can't allow it to be inside the Box
        or behind a pointer to use it
    */
    // let get_token = (*token)().await; 
    let get_token = token(); /* here token is callable since it's just only behind a heap data pointer which is Box */
    // ||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||


    //// closure traits can't be defined as a type
    //// since they are heap data which their size
    //// are unknown at compile time and must be 
    //// behind a pointer like &'valid dyn or inside the 
    //// Box with a valid lifetime 
    // type ClosureTrait = FnOnce(String) -> String; 
    struct Run<F, T = fn() -> String> //// T has a default type parameter
        where F: FnOnce(String) -> String{
        data: F,
        another_data: T,
        third_data: fn() -> ()
    }
    trait InterfaceExt{}
    impl<F, T> InterfaceExt for Run<F, T> where F: FnOnce(String) -> String{}
    
    // -> impl Trait only allowed in function not in trait return type
    // since we can't impl a trait for the return type of another trait!!
    fn runYours() -> impl FnOnce(String) -> String{ //// return closure using -> impl Trait 
        |name: String|{
            name
        }
    } 

    fn runOurs() -> Box<impl FnOnce(String) -> String>{
        Box::new(
            |name:String|{
                name
            }
        )
    }

    fn runYours_() -> &'static dyn FnOnce(String) -> String{ //// return closure using -> &dy Trait
        &|name: String|{
            name
        }
    }

    fn run_() -> impl InterfaceExt{
        fn catch(name: String) -> String{name}
        fn catch_me(){}
        let instance = Run{
            data: |you|{
                you
            },
            another_data: catch,
            third_data: catch_me
        };
        /* 
            returning the instance of the Run struct 
            since the return type is InterfaceExt means
            we must return a type that this trait is already 
            implemented for it, we can't return the trait 
            directly with this syntax inside the function
            signature, we have to put it inside the Box
            which has its own lifetime or put it behind 
            &dyn with a valid lifetime like 'static 
        */
        instance 
    }

    //// Box<dyn Trait> has its own lifetime since Box
    //// since Box has its own lifetime but &dyn Trait 
    //// needs a valid lifetime like 'static
    fn run__() -> Box<dyn FnOnce(String) -> String>{ //// return closure using -> Box<dyn Trait>
        Box::new(
            |name: String|{
                name
            }
        )
    }

    fn start<'lifetime, F>(cls: F) -> () where F: FnOnce(String) -> String + 'lifetime{ /* ... */ } 
    fn start_(cls: Box<dyn FnOnce(String) -> String>){ /* ... */ }
    fn start__(cls: fn() -> String){ /* ... */ }
    fn start___(cls: impl FnOnce(String) -> String){ /* ... */ }
    // **************************************************************************
    // **************************************************************************
    // **************************************************************************
    
}