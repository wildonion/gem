


use crate::*;



async fn test(){

    // =============================================================================================================================
    // =============================================================================================================================
    // =============================================================================================================================
    //                                                GENERIC, LIFETIMES AND CLOSURES
	// =============================================================================================================================
    // =============================================================================================================================
    // =============================================================================================================================
    //// if we're using multiple traits in a crate 
    //// in order to use their GATs on a type we should
    //// cast the type to that trait like <Struct as Trait> 
    //// then call the GAT using `::`, <Struct as Trait>::SimpleGat 
    //// since their GATs might have a same name and based 
    //// on orphan rule we should name the GAT explicity 
    //// to avoid conflict and ambiguous calls, also we 
    //// have to make sure that the trait is 
    //// implemented for the type.
    // ➔ we can return the reference from function with the lifetime of the passed in args, with a static ref or a specific lifetiem
    // ➔ the generic type of a structure must be used for one of its field since compiler allocated some sapce for it and if we don't
    //    use it it'll be unused which the won't compile hence to fix the issue we must put the generic type inside a PhantomData struct
	// ➔ generic types in function signature can be bounded to lifetimes and traits so we can use the lifetime to avoid having dangling pointer of the generic type in function body and traits to extend the type interface
    // https://stackoverflow.com/questions/27831944/how-do-i-store-a-closure-in-a-struct-in-rust
    // NOTE - defaults for type parameters are only allowed in `struct`, `enum`, `type`, or `trait` definitions
    pub struct GenFn<T, F = fn() -> T>{ //// the default type parameter of generic F is a function pointe or fn() 
        pub one_field: T,
        pub sec_field: F,
        pub third_field: fn(u8) -> String, //// this field is of type function - https://stackoverflow.com/questions/41081240/idiomatic-callbacks-in-rust
    }
    fn ret_var() -> u8{
        23
    }
    fn ret_name(age: u8) -> String{
        "wildonion".to_string()
    }
    let instance = GenFn{
        one_field: "another_wildonion".to_string(),
        sec_field: ret_var,
        third_field: ret_name, //// setting a function as the value of the third field
    };

    struct Otp<Generic=i32>{
        pub data: Generic,
    }
    
    let otp_instance = Otp::<u8>{
        data: 89
    };
    
    let instance_otp = Otp{ // the default type param is i32 for the Generic if it's not passed
        data: 89
    };

    //////////////////////// ------------------------------------------------------
    
    pub enum Service{
        Available(u8),
        Off,
        On
    }
    pub struct Closure<F> //// we can't use default type param while we're using where clause
        where F: FnOnce(String) -> String{
        pub callback: F,
        pub service: Service
    }
    impl<F: FnOnce(String) -> String> Closure<F>{
        
        fn gettertor(&self) -> &impl FnOnce(String) -> String{ // return trait in here since closures are traits
            let cb = &self.callback;
            cb
        }

        fn settertor() -> fn() -> String{
            fn get_name() -> String{
                "wildonion".to_string()
            }
            get_name
        }  
    }
    
    //////// -----------------------------------------------
    //////// +++++++++++++++++++++++++++++++++++++++++++++++
    //////// -----------------------------------------------
    struct IpData<'i>{
        pub ip: &'i str,
    };
    impl<'i> IpData<'i>{
        fn new(ip: &'i str) -> IpData{ //// the lifetime of the passed in ip must be less or equal than the lifetime of the IpData struct the ip feild
            IpData::<'i>{
                ip,
            }
        }
    }
    let mut BoxedIp = Box::new(IpData::new("0.0.0.0")); //// Box is a pointer to a heap allocation data of type T
    let mutable_boxed = BoxedIp.as_mut(); //// call as_mut() on the type requires that the type must defined mutable 
    let ref_boxed = BoxedIp.as_ref();
    let bytes_boxed_data = ref_boxed.ip.as_bytes();

    //// bounding generic T and F to traits and lifetimes
    //// also traits must be behind pointer since they are
    //// heap data types (heap data types in rust must be in 
    //// form of borrowed type which means must be passed 
    //// into other methods and functions by putting them 
    //// behind a pointer or their slice types like &str for 
    //// String and &[u8] or [u8; 32] for Vec) which must 
    //// be in form Box<dyn Trait> or &dyn Trait also 
    //// their pointer are fat pointers  
    fn setIpHosting<'s, T, F>(input: T, output: Box<dyn std::error::Error + Send + Sync + 'static>, ip_addr: Box<IpData>) 
    -> &'s str //// return a reference always needs a valid lifetime such as the one which is behind &self or an specific one in function signature 
    where F: FnOnce(String) -> hyper::Response<hyper::Body> + Send + Sync + 'static, T: Send + Sync + 's {
        "test"
    }
    //////// -----------------------------------------------
    //////// +++++++++++++++++++++++++++++++++++++++++++++++
    //////// ----------------------------------------------- 

    #[derive(Debug)] // also it's possible to bound a type to trait using derive proc macro attribute
    pub struct DataAccount<'lifetime, T=u8> // default type parameter is u8 also it can be any T type 
        where T: Send + Sync + 'lifetime{ // bounding the generic type to the traits and lifetime
        pub data: &'lifetime T,
    }
    
    struct Ben<Genericam>{ // Genericam is the generic type that can be any type 
        pub new_data: Genericam, //// it can be any type
    }
    
    let another_instance = Ben{
        new_data: 32
    };
    let instance = Ben{
        new_data: DataAccount{
            data: &5
        }
    };
    let instance = Ben{
        new_data: DataAccount{
            data: &"wildonion".to_string()
        }
    };

    let data_in_there = instance.new_data.data;

    struct Structure<'lifetime, Generic> 
        where Generic: Send + Sync + 'lifetime{
            pub data: &'lifetime Generic,
    }
    trait Feature{
        type Output;
    }
    impl<'s> Feature for Structure<'s, u8>{
        type Output = u8;
    }
    impl<'s> Structure<'s, u8>{
        fn run(&mut self) -> &Ben<i32>{ //// DataAccount default type is u8 but we're saying that we want to pass String
            let mut bytes = [0u8; 32];
            /* 
                we can't borrow the bytes since it'll be 
                dropped at the end of the function and once
                the function gets executed 
            */
            // let data: Rc<RefCell<&'s mut [u8; 32]>> = Rc::new(RefCell::new(&mut bytes));
            let name = "wildonion".to_string();
            /*
                we can't return a pointer to the String 
                since it's a heap data structure which 
                will be dropped at the end of function 
                and if there is a pointer of that exists 
                we can't return it since the pointer may 
                be converted to a dangling pointer.
                also we can't return reference to local 
                and temp variable which are owned by the 
                function.
            */
            // let option_name = Some(name);
            // return &option_name;

            // following is of type: Ben<DataAccount<String>>
            // let instance = Ben{
            //     new_data: DataAccount{
            //         data: &"wildonion".to_string()
            //     }
            // };
            /*
                we can't return the instance since 
                it contains a field that has a temp
                value behind a reference which is owned 
                by the current function 
            */
            // instance

            // if we allocate something on the heap 
            // inside the function we can't return a 
            // pointer to it, but it's ok with slices 
            // following will allocate nothing on the stack 
            // thus we're returning a pointer to the 
            // struct itself directly to the caller
            // of the run() method.
            &Ben{
                new_data: 73
            }

        }
    }
    //////////////////////// ------------------------------------------------------

    /////////////////////////////////////////////////////////
    trait BorrowArray<T> where Self: Send + Sized{
        
        // GAT example with lifetime, generic and trait bounding
        type Array<'x, const N: usize> where Self: 'x;
        type Data: Send + Sync + 'static; // can't set Data equals to DataGat here in trait defenition
    
        fn borrow_array<'a, const N: usize>(&'a mut self) -> Self::Array<'a, N>;
    }


    // pub struct Mon;
    // pub struct Node<Mon>; //// using the Mon struct as the generic type inside the Node struct

    /////////////////////////////////////////////////////////
    // default type parameter example
    struct Valid(u8, u8);
    struct test<Output = Valid>{ // default type parameter
        name: Output,
        id: i32,
    }
    ///// ========================================================================
    trait SuperTrait: Give + See{}

    trait Give{
        fn take(&self) -> &i32;
    }
    
    trait See{
        fn what(&self) -> &String;
    }
    
    struct Who{
        a: i32,
        name: String,
    }
    
    impl See for Who{
        fn what(&self) -> &String{
            &self.name
        }
    }
    
    impl Give for Who{
        fn take(&self) -> &i32{
            &self.a // take() function doesn't own the `a` variable so we can return a reference to the type i32
        }
    }
    
    fn test_trait_0<T: Give + See>(item: &T){ // T is bounded to Give and See trait
        let val: &i32 = item.take();
        let name: &str = item.what().as_str();
        println!("the value of w.a is : {}", val);
        println!("the value of w.name is : {}", name);
    }
    
    fn test_trait_1(item: &(impl Give + See)){ // item is bounded to Give and See trait
        let val: &i32 = item.take();
        let name: &str = item.what().as_str();
        println!("the value of w.a is : {}", val);
        println!("the value of w.name is : {}", name);
    }
    
    fn test_trait_just(item: impl Give + See){ // item is bounded to Give and See trait
        let val: &i32 = item.take();
        let name: &str = item.what().as_str();
        println!("the value of w.a is : {}", val);
        println!("the value of w.name is : {}", name);
    }

    fn test_trait_2(item: Box<dyn SuperTrait>){ // item is bounded to SuperTrait cause SuperTrait is an object safe trait
        let val: &i32 = item.take();
        let name: &str = item.what().as_str();
        println!("the value of w.a is : {}", val);
        println!("the value of w.name is : {}", name);
    }
    
    fn test_trait_3<T>(item: &T) where T: Give + See{ // the generic T is bounded to Give and See trait
        let val: &i32 = item.take();
        let name: &str = item.what().as_str();
        println!("the value of w.a is : {}", val);
        println!("the value of w.name is : {}", name);
    }
    
    
    let w = Who{a: 64, name: "wildonion".to_string()};
    let p_to_w: *const Who = &w; // a const raw pointer to the Who truct
    println!("address of w is : {:p}", p_to_w);
    test_trait_0(&w);
    ///// ========================================================================

    // order must be lifetimes, then consts and types
	impl<'a, Pack: Interface + 'a> Into<Vec<u8>> for Unpack<'a, Pack, SIZE>{ //// based on orphan rule we have to import the trait inside where the struct is or bound the instance of the struct into the Into trait in function calls - we want to return the T inside the wrapper thus we can implement the Into trait for the wrapper struct which will return the T from the wrapper field
	    fn into(self) -> Vec<u8> {
            self.arr.to_vec()
	    }
	}

    
    pub const WO: &str = "widonion";
	pub const SIZE: usize = 325;
	pub type Context<'a, Pack> = Unpack<'a, Pack, SIZE>; //// Pack type will be bounded to Interface trait and 'l lifetime 
	pub struct Unpack<'l, T: Interface + 'l + Into<T>, const U: usize>{ //// T is of type Pack struct which is bounded to 'l lifetime the Into and the Interface traits and U (constant generic) must be a constant usize type - Unpack takes a generic type of any kind which will be bounded to a trait and a lifetime but it must be referred to a field or be inside a PhantomData since T and the lifetime will be unused and reserved by no variables inside the ram
	    pub pack: T, //// pack is a pointer or a reference and is pointing to T which is a generic type and bounded to a trait and a valid lifetime as long as the lifetime of the struct instance
	    pub arr: &'l [u8; U], //// U is a constant usize
	}

	pub struct Pack; //// we've allocated some space inside the stack for this struct when defining it which has long enough lifetime to initiate an instance from it using struct declaration and return a reference to that instance inside any function 
	pub trait Interface{}

	impl Interface for Pack{} //// is required for return_box_trait(), return_impl_trait() and return_none_trait() functions in order to work

	fn return_none_trait<T>() -> () where T: Interface{ // NOTE - `T` type must be bound to Interface trait

	}

    // by returning the impl Interface for the type that is being returned we can call the trait methods on the type when we're calling the following method since we're returning acutally kinda an instance of the type
	fn return_impl_trait() -> impl Interface { // NOTE - returning impl Trait from function means we're implementing the trait for the object that is returning from the function regardless of its type that we're returning from the function cause compiler will detect the correct type in compile time and implement or bound the trait for that type if the trait has implemented for that type (the trait MUST be implemented for that type)
	    Pack {}
	}

	fn return_box_trait() -> Box<dyn Interface + 'static> { // NOTE - returning Box<dyn Trait> from function means we're returning a struct inside the Box which the trait has implemented for and since traits have unknown size at compile time we must put them inside the Box with a valid lifetime like 'static
	    Box::new(Pack {})
	}
    pub struct Commander{}
    struct Command<'c, G: Send + Sync + 'static, const N: usize>{ // 'static lifetime is for G, 'c is for the cmd pointer which is a String slice
        pub shared: Arc<Mutex<G>>,
        pub cmd: &'c str
    }
    fn fuckMe() -> &'static str{
        "wildonion"
    }
    fn fuckMeOneMoreTime<'b>() -> &'b str{
        "wildonion"
    }
    
	

    // =============================================================================================================================
    // -SUCCESS-
    //  type Boxed = Box<dyn Trait + 'lifetime>;
    //  type Boxed = Box<&'a u64>;
    //  Generic : Trait + 'lifetime
    //  let var: &'lifetime Type;
    //  let var: &' Boxed = Box::new(||{});
    // -ERROR-
    //  Generic : Type + 'lifetime
    // >>> variable lifetime is how long the data it points to can be statically verified by the compiler to be valid at its current memory address

    ///--------------------------------------------
    // kinda async closure 
    async fn callbacker<C, T>(c: C) 
            where 
                C: futures::Future<Output=T>, //// Output is the default type parameter
                T: FnMut(String) -> String
    {
        c.await;
    }

    let c = async {
        |name: String|{
            name
        }
    };
    callbacker(c).await;
   ///--------------------------------------------

    trait Some{
        fn run(&self){}
    }
    impl Some for Boxed{
        fn run(&self){} 
    }

    type Boxy8<'a> = Box<&'a String>; //// we have to store a pointer to the String inside this Box with a valid lifetime of 'a 
    type Boxed = Box<dyn FnMut() + 'static + Send + Sync>; //// Boxed type can be shared between threads and .awaits safely - we must bound the type that wants to be a pointer or to be a referenced from a heap location like FnMut() closure to a valid lifetime like 'static
    let var: Boxed = Box::new(||{}); //// since the Some trait is implemented for Boxed type we can call the run() method on the isntance of this type also the closure is bounded to a static lifetime

    fn call<'a>(a: &'a mut Boxed) -> Boxed where Boxed: Some + 'a { //// in order to bind the Boxed to Some trait the Some trait must be implemented for the Boxed - can't bound a lifetime to a self-contained type means we can't have a: u32 + 'static
        // 'a lifetime might be shorter than static and describes how long the data it points to can be valid
        //...
        a.run(); //// a is a mutable pointer of type a Boxed with 'a lifetime - since we have &self in the first param of the run() method for the Some trait we can call the run() method using dot notation
        Box::new(||{})
    }

    //// we can't remove the 'a lifetime from the passed in parameter since a pointer to name doesn't live long enough to return it from the function
    //// lifetime bounding is for those types that are a reference or a pointer to other types or are borrowing the ownership of a type due to this fact if T was bounded to a lifetime means it must be a pointer to a type (which is &'a T in our case in function param) with a valid lifetime 
    fn ref_me<'a, T>(name: &'a T) -> &'a T where T: ?Sized{ //// since the trait `Sized` is not implemented for `str` or those types that have unknown size at compile time we've bounded the T to the 'a lifetime and ?Sized trait in order to pass unknown size types like str to the function
        let get_name: &'a T = &name; //// since T is bounded to 'a lifetime in order to return a reference to type T we have to define the var to be of type &'a T
        get_name
    }

    let name = "narin";
    let res = ref_me::<&str>(&name); //// we have to pass a reference to the name since the function param is of type &T which in our case will be &&str - the generic type can be str and &str since it's bounded to ?Sized trait
    // =============================================================================================================================
    
}