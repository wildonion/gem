


use crate::*;


async fn test(){

    trait InterfaceMe{}
    impl InterfaceMe for () {}
    pub type BoxeFutureShodeh = Box<dyn std::future::Future<Output=BoxedShodeh>>;
    pub type BoxedShodeh = Box<dyn FnOnce(String) -> String + Send + Sync + 'static>;

    // -----------------------------------
	// handling a recursive async function
	// -----------------------------------
	// https://rust-lang.github.io/async-book/07_workarounds/04_recursion.html
	// NOTE - Future trait is an object safe trait thus we have to Box it with dyn keyword to have kinda a pointer to the heap where the object is allocated in runtime
	// NOTE - a recursive `async fn` will always return a Future object which must be rewritten to return a boxed `dyn Future` to prevent infinite size allocation in runtime from heppaneing some kinda maximum recursion depth exceeded prevention process
	//// the return type can also be ... -> impl std::future::Future<Output=usize>
	//// which implements the future trait for the usize output also BoxFuture<'static, usize>
	//// is a pinned Box under the hood because in order to return a future as a type
	//// we have to return its pinned pointer since future objects are traits and 
	//// traits are not sized at compile time thus we have to put them inside the 
	//// Box or use &dyn to return them as a type and for the future traits we have
	//// to pin them into the ram in order to be able to solve them later so we must 
	//// return the pinned Box (Box in here is a smart pointer points to the future)
	//// or use impl Trait in function return signature. 
	//
	//// async block needs to be pinned into the ram and since they are traits of 
	//// the Future their pointer will be either Box<dyn Trait> or &dyn Trait, 
	//// to pin them into the ram to solve them later.
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
	//// BoxFuture<'fut, ()> is Pin<alloc::boxed::Box<dyn Future<Output=()> + Send + Sync + 'fut>>
	pub const CHARSET: &[u8] = b"0123456789";
    pub fn async_gen_random_idx(idx: usize) -> futures_util::future::BoxFuture<'static, usize>{ // NOTE - pub type BoxFuture<'a, T> = Pin<alloc::boxed::Box<dyn Future<Output = T> + Send + 'a>>
	    async move{
		if idx <= CHARSET.len(){
		    idx
		} else{
		    gen_random_idx(rand::random::<u8>() as usize)
		}
	    }.boxed() //// wrap the future in a Box, pinning it
	}
	pub fn ret_boxed_future() -> std::pin::Pin<Box<dyn futures::future::Future<Output=()>>>{ //// Pin requires the pointer to the type and since traits are dynamic types thir pointer can be either &dyn ... or Box<dyn...>
	    Box::pin(async move{
		()
	    })
	}

    //// recursive random index generator
    pub fn gen_random_idx(idx: usize) -> usize{
        if idx < CHARSET.len(){
            idx
        } else{
            gen_random_idx(rand::random::<u8>() as usize)
        }
    }

    //--------------------------------------------------------------------
    // EXAMPLE - Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>
    // NOTE - closure types are traits so when we want 
    //        to have a field of type closure we have 
    //        to use the generic and bound that generic 
    //        to the Fn trait in struct signature or use 
    //        where clause or put the field inside the 
    //        Box<dyn Trait> or &dyn Trait. 
    // implementing trait or bounding it to generics using: 
    //      - bounding it to generics (where in struct and function or function and struct signature) like where T: FnMut() -> () or struct Test<T: FnMut() -> ()>{pub d: T}
    //      - in function return and param type (: impl Trait and -> impl Trait)
    //      - derive like macro on top of the struct or types
    //      - directly by using impl Trait for Type in other scopes
    // returning traits from the function or use it as a function param by:
    //      - Box<dyn Trait>
    //      - &dyn Trait  
    //      - impl Trait
    //      - where T: Trait
    //--------------------------------------------------------------------
    struct UseramHa;
    // the following is wrong since we're using trait bounds
    // thus Arc<Mutex<User>> must be trait
    // type Data = dyn Arc<Mutex<User>> + Send + Sync + 'static; 
    
    // we can use a generic type which referes to this type
    // and bound the Send + Sync + 'static in function or struct
    // signature 
    type UseramHaData = Arc<Mutex<UseramHa>>; 
    
    // in here the field `d` is of type UseramHaData 
    // which is bounded to some trait in struct signature
    struct UserDataHa where UseramHaData: Send + Sync + 'static{
        d: UseramHaData
    }
    struct TestMeWhere<F>
        where F: FnMut(String) -> String{ // setting a FnMut closure in struct field using where
        pub method: F,
    }
     
    struct TestMeBound<F: FnMut(String) -> String>{ // setting a FnMut closure in struct field using generics
        pub method: F,
    }

    struct TestMeBox{ // setting a FnMut closure in struct field using Box<dyn>
        pub method : Box<dyn FnMut(String) -> String>
    }

    //// fn() -> T is not a trait type it's a function type
    //// thus there is no need to be behind a reference like
    //// Box<dyn> or &dyn
    struct TestMeFunc<T, F = fn() -> T>{ // setting a function pointer in struct field using generics
        pub method: F,
        pub t_type: T, // T must refer to a field, or be a `PhantomData` otherwise must be removed
    }

    pub struct Server<'e, E>{
        pub address: String, //// the peer_id of the sevrer
        pub weights: u16,
        //// this field contains an array of events of type 
        //// function each of which returns the generic `E`
        //// also since the array is a slice form of Vec 
        //// we need to use it behind a reference because 
        //// [] is not sized thus we've passed the lifetime
        //// 'e to the struct signature.  
        pub events: &'e [fn() -> E], 
    }
    
    struct TestMeFunc1<T>{ // setting a function pointer in struct field directly  
        pub method: fn() -> T,
        pub t_type: T, // T must refer to a field, or be a `PhantomData` otherwise must be removed
    }
    
    //// since the return type have a reference
    //// thus we have to use a valid lifetime 
    //// for that because we can't return a 
    //// reference from function which is owned
    //// by that function thus we've used the 
    //// 'static lifetime.
    //
    //// returning traits as type requires to 
    //// put them inside the box or use &dyn 
    //// with a valid lifetime but we can use
    //// them as generic in struct fields and 
    //// function params directly using where
    //// clause or inside the function or struct 
    //// signature; if we want to return the trait
    //// we must to return an instance of its 
    //// implementor since they are abstract 
    //// dynamic sized types and don't have 
    //// size at compile time and we can't just
    //// simply return a trait inside function 
    //// because everything in rust must be sized.
    //
    //// if we want to return the trait behind 
    //// a pointer like &dyn we must use a valid
    //// lifetime before &dyn alos bound that
    //// trait to that lifetime too.
    struct Test10{}
    impl InterfaceMe for Test10{} 
    fn test_10() -> &'static (dyn InterfaceMe + 'static){ //// here we're returning the trait behind a pointer or &dyn with a 'static lifetime thus the trait itself must be bounded to the 'static lifetime too
        &Test10{} //// since we're returning a reference we need to put the instance behind a reference
    }

    fn test_11<'validlifetime>() -> &'validlifetime (dyn InterfaceMe + 'validlifetime){ //// here we're returning the trait behind a pointer or &dyn with 'validlifetime lifetime thus the trait itself must be bounded to the 'validlifetime lifetime too
        &Test10{} //// since we're returning a reference we need to put the instance behind a reference
    }
    
    fn test<'l, T>() where T: FnMut(String) -> String + Send + Sync + 'static + 'l{
        
        () // or simply comment this :)
        
    }
    fn _test<'l, T: FnMut(String) -> String + Send + Sync + 'static>(){
        
        () // or simply comment this :)
                
    }
    // we can impl a trait for the return type so we can call 
    // trait methods on the return type also type aliases cannot 
    // be used as traits so test_<'l, T: BoxedShodeh> is wrong
    // also the return type is () and we're impl InterfaceMe for 
    // the return type in function signature thus InterfaceMe 
    // trait must be implemented for () before doing this.
    fn test_<'l>(param: BoxeFutureShodeh) -> impl InterfaceMe{ // or impl Future<Output=Boxed> the default type param output is of type Boxed
        
        () // or simply comment this :)
        
    } 
    //// the return type is Box<impl InterfaceMe>
    //// means that the instance of the InterfaceMe
    //// implementor must be inside the Box and since 
    //// InterfaceMe is implemented for () we can 
    //// put it inside the Box like Box::new(())
    fn test_n() -> Box<impl InterfaceMe>{ /* we can return any type that implements the InterfaceMe trait, () does this! */
        Box::new(())
    }
    fn test_1<'lifetime, C 
                // : FnOnce(String) -> String + Send + Sync + 'static + 'lifetime // or we can use this syntax instead of where
                >(c: C) // the passed in param is of type C which is a generic type which is bounded to the FnOnce trait
        -> (std::pin::Pin<Box<dyn std::future::Future<Output=Box<C>>>>,  //// we must put the generic C inside the Box not its equivalent which is a closure bounded to FnMut trait
            impl std::future::Future<Output=u8>) //// the return type is a tuple in which the second one impl a trait for the returned type
        where C: FnOnce(String) -> String + Send + Sync + 'static + 'lifetime //// the whole `FnOnce(String) -> String` is the trait defenition returns String type which we're bounding it to other traits and lifetimes
    { 
        (
            //// we can't have the following async{Box::new(c)}
            //// inside the Pin since Pin accept a pointer of the 
            //// passed in type and we can't simply borrow the async{}
            //// block to put it inside the Pin also we can't have &async{}
            //// thus we should put the async{} block inside the Box 
            //// since Box is a smart pointer that has a valid lifetime
            //// on its own. 
            //
            //// &async{} can't be unpinned since async{} is of type 
            //// Future<Output=<WHATEVERTYPE>> which is a trait and traits
            //// are abstract dynamic size which can't be sized at compile time
            //// and they need to be in form &dyn Trait or Box<dyn Trait> thus
            //// async{} is a dynamic size type which must be behind a pointer 
            //// with dyn keyword with a valid lifetime in order to be unpinned 
            //// and this can only be coded and referenced syntatically using Box
            //// which we can put the Box::new(async{}) inside the Pin or use Box::Pin
            //// which returns a pinned Box. 
            // std::pin::Pin::new(&async{Box::new(c)}); // this can not be unppined
            Box::pin(
                async{ // async blocks are future objects
                    //// we have to put the passed in param in here 
                    //// since the type inside the Box must be the 
                    //// generic C itself not the something like closure, 
                    //// |name: String| name explicity!
                    Box::new(c) 
                }
            ),
            async{
                78
            }
        )
    }
    
}