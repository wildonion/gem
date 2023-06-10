




/* LIFETIMES, BOX, PIN, TRAITS, GENERICS */

use crate::*;




async fn cls_fn() {
    fn return_cls() -> Box<dyn FnOnce(i32) -> i32>{ //// instances of FnOnce can be called, but might not be callable multiple times. Because of this, if the only thing known about a type is that it implements FnOnce, it can only be called once - FnOnce is a supertrait of FnMut
        Box::new(|x| x + 1)
    }    
    function_with_callback(return_cls()); // use .await to suspend the function execution for solving the future
}

async fn function_with_callback(cb: Box<dyn FnOnce(i32) -> i32>){
    cb(32);
    #[derive(Clone)]
    struct Request{
        pub user: u32,
        pub access: u32,
    }
    
    let res = run(move |req: Request|{
        println!("user {} has access {}", req.user, req.access);
    });
    
    
    fn run<C>(cls: C) where C: FnOnce(Request) + Send + 'static {
        let req = Request{user: 2893, access: 1};
        cls(req);
    }
}



async fn test(){


    cls_fn().await;

    // ---------------------------------------------------------------------------------------------
    // - generics, bounding to traits and lifetimes, Rc, RefCell, Weak, Arc, Mutex, DeRefMut, RefMut
    // =============================================================================================================================
    // closure coding - trait must be referenced by putting them inside Box or use with &dyn Trait if they want to be used as param or struct field
    // https://zhauniarovich.com/post/2020/2020-12-closures-in-rust/
    // https://blog.cloudflare.com/pin-and-unpin-in-rust/
    // https://fasterthanli.me/articles/pin-and-suffering
    // https://stackoverflow.com/questions/2490912/what-are-pinned-objects
    // https://medium.com/tips-for-rust-developers/pin-276bed513fd1
    // https://users.rust-lang.org/t/expected-trait-object-dyn-fnonce-found-closure/56801/2
    //// traits are not sized and their size depends on the 
    //// implementor like the struct size thus they must be 
    //// behind a pointer like Box<dyn or &dyn and we can put 
    //// the Boxed trait inside a return type then we can return 
    //// the instance of that type that this 
    //// trait is implemented for.
    // --------------------------------------------------------------------------
    //// the location of dynamic types in rust is on the heap and don't impl Copy trait  
    //// and their pointers, cap and length will be stored on the stack 
    //// also due to the expensive memory cost they must be either cloned, 
    //// borrowed or be in their sliced form like &str, &[u8], Box<dyn Trait> or &dyn Trait
    //// to move them in other scopes with losing ownership.
    //
    //// FnOnce: there might be multiple references of a type due to the borrowing rules 
    //// and all of them must be dropped if the closure wants to eat the type 
    //// since the type can't be available after moving into the closure 
    //// and we should avoid dangling pointer by dropping all the references 
    //// of the moved type.
    //// when an object or a value is moved into another value
    //// it'll relocate into a new position inside the ram and we can 
    //// prevent this from happening using Pin
    //
    //// Pin<P> are objects of pointer type P that are fixed in memory. 
    //// It occupies and keeps its position until it is dropped. In Rust, 
    //// basically all types are portable. In other words, we can move 
    //// an object of a type to another variable by-value. When the 
    //// ownership of an object is moved from one variable to another, 
    //// the object can be relocated. Pin<P> is a type surrounding a 
    //// pointer, we can use Pin<Box <T>> like Box<T> and similarly 
    //// Pin<&mut T> can be used as & mut T
    //
    //// type behind a pointer means that there is a pointer that 
    //// is pointing to the location of that type either inside
    //// the stack or heap also we have to put a pointer of the
    //// type that we want to pin it into the memory inside the Pin
    //// which can be done either by using Box (Box<dyn> or &dyn for dynamic sized) 
    //// or & like Pin<Box<dyn Trait>>, Pin<Box<T>>, Pin<&mut T> or Pin<&T>
    //
    //// Rust provides three different traits Fn , FnMut , and FnOnce that 
    //// can be used as trait bounds for closure arguments, each closure 
    //// implements one of these three traits, and what trait is automatically 
    //// implemented depends on how the closure captures its environment
    //
    //// a trait object (dyn Trait) is an abstract unsized or dynamic type, 
    //// it can't be used directly instead, we interact with it through a reference, 
    //// typically we put trait objects in a Box<dyn Trait> or use &dyn Trat, though 
    //// with futures we might have to pin the box as well means if we want to return 
    //// a future object first we must put the future inside the Box since Future is 
    //// a trait which must be behind a pointer and second we must pin the Box to prevent 
    //// it from being relocated inside the ram to solve it later, the reason that why we 
    //// must put the Box inside the Pin is because Pin takes a pointer of the type to pin 
    //// it and our pointer in our case must be either &dyn Future or Box<dyn Future> since 
    //// Future is a trait and trait objects are dynamic sized we must use dyn keyword thus 
    //// our type will be Pin<Box<dyn Future<Output=T>>>
    //
    //// since async blocks are of type Future trait in roder to return them
    //// as a type their pointer either Box<dyn Trait> or &dyn Trait must be
    //// pinned into the ram to let us solve them later because rust doesn't 
    //// have gc and it'll drop the type after it moved into the new scope or
    //// another type thus for the future objects we must pin them to ram and 
    //// tell rust hey we're moving this in other scopes but don't drop it because
    //// we pinned it to the ram to solve it in other scopes, also it must have
    //// valid lifetime during the the entire lifetime of the app.  
    //
    //// if we want to return a trait from a function or use it as a param in 
    //// struct fields or functions we must use the generic form like defining 
    //// a generic `T` and bound it to that trait using `where` or in function 
    //// signature directly or the trait must be behind a pointer since it's a dynamic 
    //// types thus we must put it either inside the Box<dyn Trait> or use &dyn Traut 
    //
    //// traits are abstract dynamic sized types which their size depends on the 
    //// implementor at runtime thus they must be behind a pointer using either
    //// Box<dyn Trait> or &dyn Trait.
    //// closures can be Copy but the dyn Trait are not. dyn means its concrete type(and its size) 
    //// can only be determined at runtime, but function parameters and return types 
    //// must have statically known size.
    //// dyn Trait types are dynamically sized types, and cannot be passed as parameters directly. 
    //// They need to be behind a pointer like &dyn Trait or Box<dyn Trait>.
    //// dynamic sized types like traits must be in form dyn T which is not an exact type, 
    //// it is an unsized type, we'd have to use some kind of reference or Box to address it
    //// means Trait objects can only be returned behind some kind of pointer.
    //// - for sharing data between threads safeyly the data must be inside Arc<Mutex<T>> and also must be bounded to the Send + Sync + 'static lifetime or have a valid lifetime across threads, awaits and other scopes when we move them between threads using tokio job queue channels
    //// - future objects must be Send and static and types that must be shared between threads must be send sync and static 
    //// - Box<dyn Future<Output=Result<u8, 8u>> + Send + Sync + 'static> means this future can be sharead acorss threads and .awaits safely
    type GenericResult<T, E> = std::result::Result<T, E>;
    type Callback = Box<dyn 'static + FnMut(hyper::Request<hyper::Body>, hyper::http::response::Builder) -> CallbackResponse>; //// capturing by mut T - the closure inside the Box is valid as long as the Callback is valid due to the 'static lifetime and will never become invalid until the variable that has the Callback type drop
    type CallbackResponse = Box<dyn std::future::Future<Output=GenericResult<hyper::Response<hyper::Body>, hyper::Error>> + Send + Sync + 'static>; //// CallbackResponse is a future object which will be returned by the closure and has bounded to Send to move across threads and .awaits - the future inside the Box is valid as long as the CallbackResponse is valid due to the 'static lifetime and will never become invalid until the variable that has the CallbackResponse type drop
    type SafeShareAsync = Arc<Mutex<std::pin::Pin<Box<dyn std::future::Future<Output=u8> + Send + Sync + 'static>>>>; //// this type is a future object which has pinned to the ram inside a Box pointer and can be shared between thread safely also it can be mutated by threads - pinning the Boxed future object into the ram to prevent from being moved (cause rust don't have gc and each type will be dropped once it goes out of its scope) since that future object must be valid across scopes and threads until we await on it 
    type SafeShareClosure = Arc<Mutex<Box<dyn FnOnce(hyper::Request<hyper::Body>) -> hyper::Response<hyper::Body> + Send + Sync + 'static>>>; //// this type is safe and sendable to share between threads also it can be mutated by a thread using a mutex guard; we have to use the &dyn keyword or put them inside the Box<dyn> for traits if we want to treat them as a type since they have no sepecific size at compile time thus they must be referenced by the &dyn or the Box<dyn> 
    pub trait InterfaceMe{}
    pub type BoxeFutureShodeh = Box<dyn std::future::Future<Output=BoxedShodeh>>;
    pub type BoxedShodeh = Box<dyn FnOnce(String) -> String + Send + Sync + 'static>;
    impl InterfaceMe for BoxedShodeh{} //// implementing for a boxed type
    impl InterfaceMe for (){} // we must impl InterfaceMe for () in order to be able to impl InterfaceMe for () (the return type) inside the test_() function
    

    let callbackhaste = move |function: fn() -> ()|{
        function()
    };
    fn runhaste(){}
    callbackhaste(runhaste);

    (||async move{})().await; // building, calling and awaiting at the same time
    let this = (||async move{})(); // building and calling closure at the same time
    this.await; // await on the this since the closure body is a future object
    let this = (||async move{}); // building the closure inside ()
    this().await; // calling the closure and await on it since the body of the closure inside () is a future object

    // a closure inside a Box with async body which will build, call and await on another closure with async body 
    let _ = Box::new( || async move{
        (
            || async move{
                34
            }
        )().await;
    });

    fn Ankir(name: impl InterfaceMe){ //// implementing the InterfaceMe trait for the passed in type means that we're bounding the passed in type to this trait

    }

    let network: Box<dyn FnMut(String) -> std::pin::Pin<Box<dyn std::future::Future<Output=String>>> + Send + Sync + 'static> =
        Box::new(|addr: String|{
            Box::pin(async move{
            addr
        })
    });
	
	
     
    
    // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
    'aSexyLabeledBlock:{
        type EmptyType = ();
        //// the type of func variable is
        //// a function which return an 
        //// empty type and we can initialize 
        //// it by putting the run() function
        //// inside of it. 
        fn run() -> (){
            ()
        }
        let func: fn() -> EmptyType = run;  
        //// if the we want to use closures as 
        //// a type explicity like returning them 
        //// from function or as a function param
        //// we must put them inside the Box since 
        //// closures are of type traits and we 
        //// must use traits behind a reference 
        //// because they are not sized. 
        //
        //// since cls is of type FnMut we must 
        //// define it as mutable
        let mut cls: Box<dyn FnMut(String) -> EmptyType>;
        cls = Box::new(|name|{
            ()
        });
        cls("wildonion".to_string());
        //// the following is a struct that takes two generics 
        //// the `J` is a FnMut closure which takes a function 
        //// that returns the generic `T` as its param and return 
        //// a Result with a Boxed trait which is bounded to Send 
        //// Sync traits and 'static lifetime. 
        //
        //// returning a trait inisde a Box in the error part of the result means 
        //// that Error trait must be implemented for the type (like: ... -> impl Trait 
        //// in function return) that has caused the error at runtime and if so, we can 
        //// return that type when we get error at runtime for example if the MyError 
        //// struct implements the Error trait we can put the instance of the MyError 
        /// struct inside the error part of the Result like : Err(my_error_instance)
        type ErrType = Box<dyn std::error::Error + Send + Sync + 'static>;
        pub struct TaskStruct<J, T> where J: FnMut(fn() -> T) -> Result<(), ErrType>{ //// generic `J` is a closure type that accept a function as its argument 
            job: J, //// the job itself
            res: T, //// job response
        }
        let mut task_ = TaskStruct{ //// since job field is of type FnMut thus we have to define the instance mutable in order to be able to call the job field
            job: {
                //// the passed in param is of type function since 
                //// the signature inside the struct accepts a function 
                |function: fn() -> String|{ //// building the closure with a param called function and type a function which returns String
                    function(); //// calling the function inside the closure body
                    Ok(())
                }
            },
            res: "response_message".to_string()
        };
        fn ret_string() -> String{
            "wildonion".to_string()
        }
        //// since the `job` field is a FnMut closure thus 
        //// we ahve to define the instance of the `TaskStruct`
        //// as mutable to be able to call the `job` field.
        //
        //// task_.job is of type FnMut closure in which 
        //// it accepts a function as its param thus we've
        //// defined res_string() function which returns a
        //// String to pass it to the task_.job closure
        //// finally we can call it like task_.job() by 
        //// passing the ret_string() function as the param.
        let mut job = task_.job; 
        // let res = job(ret_string);
        let res = (task_.job)(ret_string);

        //// NOTE - impl Trait` only allowed in function and inherent 
        ////        method return types, not in closure return.
        type GenericT = String;
        let callback_lobstr = |function: fn() -> GenericT| async move{
            let func_res = function();
            func_res //// it's a String
        };
        fn functionToPass() -> String{
            "wildonion".to_string()
        }
        callback_lobstr(functionToPass).await;

        //// the following is a closure that takes a
        //// closure as input param which takes a
        //// function as input param, since we can't
        //// use traits directly as generic type due
        //// to their unknown size at compile time we 
        //// must use them behind a pointer like Box<dyn Trait>
        //// or &dyn Trait; the return type of closures and 
        //// functions are empty type or (). 
        let callback = |mut func: Box<dyn FnMut(fn() -> ()) -> ()>|{
            //// we've passed the a_func function to this closure
            //// since func is a closure that takes a function 
            func(a_func); 
        };
        fn a_func(){}
        callback(Box::new(|a_func| {}));

    }
    // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++


    
	// ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
    trait Trait<T>{} //// can't bound generic in trait to other traits since it's not stable 
	trait Trait1{}
	struct StrucT(u16);
	impl Trait1 for StrucT{}
	impl<T> Trait<T> for StrucT{} 
	// type Event<StrucT> = fn() -> impl Trait1; //// `impl Trait` in type aliases is unstable
	// fn one<G>() -> impl Trait<G> + Trait1{
	//     StrucT(20892)
	// }
	// fn two<G>() -> impl Trait<G> + Trait1{
	//     StrucT(20892)
	// }
	// fn three<G>() -> impl Trait<G> + Trait1{
	//     StrucT(20892)
	// }
	// also: `impl Trait` only allowed in function and inherent method return types, not in `fn`
	// let events: Vec<fn() -> impl Trait1> = vec![one, two, three];
	type Event<StrucT> = fn() -> StrucT;
	fn one() -> StrucT{
	    StrucT(20892)
	}
	fn two() -> StrucT{
	    StrucT(20892)
	}
	fn three() -> StrucT{
	    StrucT(20892)
	}
	let events: Vec<Event<StrucT>> = vec![one, two, three];
	let ids = events
	    .into_iter()
	    .map(|e| e())
	    .collect::<Vec<StrucT>>();
	// ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
	
	
    // struct StructMyself{}
    // struct CustomErr{}
    // impl Interface for StructMyself{}
    // type GenericResultInja<E> = Result<impl Interface, E>; //// impl Trait can only be used inside the function return type and here is unstable
    // fn ret_instance() -> GenericResultInja<CustomErr>{
    //     StructMyself{}
    // } 

    



}