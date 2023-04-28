


/* 
                                                → RUST OWNERSHIP AND BORROWING RULE EXPLAINED
					                    https://github.com/wildonion/cs-concepts/blob/main/rust.rules

        since dynamic types like Vec, String and traits are on the heap, by passing them into new scopes, by default they will be moved by rust compiler 
        from the heap in order to free the memory location that they've just allocated to free up some huge space at runtime and this is why rust doesn't
        have garbage collector and use borrowing and ownership rule instead of that hence due to this nature it'll let us have a safe concurrency and 
        it'll change the way of coding a little bit since you have to use tokio job queue channels to pass and share Arc<Mutex<T>>: Send + Sync + 'static
        between threads in order to access them later outside the threads and other scopes also if we move a heap data type the lifetime of that will be 
        dropped due to not having garbage collector feature in which we can't borrow the type after it has moved; the solution is to borrow the ownership 
        of them from where they are stored either by using their borrowed or sliced form like &str and &[] for String and Vec respectively (we should know 
        that str and [] are unknown sized types and they must be behind a pointer like &str or &[u8; 64] which is a pointer to a fixed size for []), cloning 
        and move that clone between other scopes which is expensive or by taking a reference to them using as_ref() method or putting & behind them to create 
        a pointer which will point to the location of their heap area hence we borrow the type and always pass by reference because its size can't be known 
        at compile time like str and [] or we don't want to lose its ownership, also is good to know that dynamic heap data pointers fat ones since extra 
        bytes which has been dedicated to their length inside the heap are in their pointers also their pointers must have valid lifetime across scopes and 
        threads in order to avoid dangling pointer issue since we can't return a pointer from a scope which is owned by that scope or in essence we can't 
        return a pointer to a heap data from function if the underlying type is owned by the fuction since once the function gets executed all its data will 
        be dropped from the ram thus returning a pointer to them to the caller is useless since the pointer might be a dangling pointer, of course this is 
        not true about the slices or stack data types like &str, &[] or the types that has no allocation in stack inside the function (like creating struct 
        using Struct{} without storing it inside a variable), to fix the issue we can return them with a valid lifetime defined in struct, enum fields or 
        function signatur or by putting them inside the Box (with dyn keyword for trait) which is a smart pointer (smart pointers are wrapper around the 
        allocation T which manages the allocation by borrowing the ownership of T) and have a valid lifetime in itself; as_ref() will convert the type into 
        a shared reference by returning the T as &T which we can't move out of it when it's being used by other scopes and threads in other words moving out 
        of a shared reference or moving the heap data that is behind a pointer it's not possible since by moving the type its lifetime will be dropped from 
        the ram thus its pointer will point to no where, we must either clone the type, use its borrow (its pointer) or its dereferenced pointer (note that 
        dereferencing will move the type out of the pointer if Copy trait is not implemented for that thus for heap data can't dereference the pointer if the 
        underlying data doesn't implement the Copy trait which we must clone it to prevent from moving) to pass to other scopes otherwise we CAN'T dereference 
        or move it at all because Clone is a supertrait of the Copy trait; also we MUST know this that inside a scope multiple immutable references of a type 
        or instance can be there but only one mutable reference must be used for that instance for example inside a method struct we can have multiple immutable 
        reference of the self but only one mutable reference of the self can be used, this rule allows rust to have safe concurreny and thread safe channels 
        like mpsc in which we can move a shareable data like Arc<Mutex<T>>: Send + Sync + 'static (the type must be cloneable, lockable and bounded to Send, 
        Sync traits and have 'static lifetime to be valid across threads) between threads that can be read by multiple producer or multiple threads (immutable 
        references) at the same time but only a single consumer or a single thread (mutable reference) can use that data also the receiver side of the channel 
        is not shareable since Clone trait is not implemented for that but the sender side can be cloned and shared between threads.

*/



use std::mem::size_of_val;
use std::str;
use std::{slice, mem};
use std::collections::HashMap;
use std::{cmp::Eq, hash::Hash};
use std::fmt::Display;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::fs;
use std::{sync::{Arc, Mutex}, iter::Cloned};
use borsh::{BorshDeserialize, BorshSerialize};
use futures_util::FutureExt;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fmt;








pub static mut ARR: [u8; 3] = [0 as u8; 3]; //// filling the array with zeros 3 times
pub static SEEDS: &[&[u8]; 2] = &["wildonion".as_bytes(), &[233]];







#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    
struct Cacher<U, T> where T: FnMut(U) -> U{
    closure: T,
    map: HashMap<U, U>,
    result: Option<U>,
}

impl<U, T> Cacher<U, T> where T: FnMut(U) -> U, U: Eq + Hash + Display + Copy{
    fn new(_: U, closure: T) -> Cacher<U, T>{
        Cacher{
            closure,
            map: HashMap::new(),
            result: None,
        }
    }

    fn value(&mut self, arg: U) -> U {
        match self.result{
            Some(v) => v,
            None => {
                let result = self.map.entry(arg).or_insert((self.closure)(arg));
                self.result = Some(*result);
                *result
            }
        }
    }
}


fn generate_workout(intensity: u32, random_number: u32) {
    let mut a_simple_var: u8 = 34;
	let callback = move |num: u32| -> u32 {
            a_simple_var = 56;
            println!("a simple var just moved here");
            println!("calculating slowly...");
            num+1 // we can add one to the num because this closure can mutate its environment vairable and it moves them to its scope!
        
      };
      
    let mut expensive_result = Cacher::new(34, callback);
    if intensity < 25 {
        println!("Today, do {} pushups!", expensive_result.value(intensity));
        println!("Next, do {} situps!", expensive_result.value(intensity));
    } else {
        if random_number == 3 {
            println!("Take a break today! Remember to stay hydrated!");
        } else {
            println!(
                "Today, run for {} minutes!", expensive_result.value(intensity)
            );
        }
    }
}


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



//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////

pub async fn generic(){
	
	   {
            'outer: loop{ // outter labeled block 
                println!("this is the outer loop");
                'inner: loop{ // inner labeled block 
                    println!("this is the inner loop");
                    // break; // only the inner loop

                    break 'outer;
                }

                println!("this print will never be reached"); //// this is an unreachable code
            }




            let mut counter = 0;
            let result = loop{
                counter += 1;
                if counter == 10{
                break counter * 2;
                }
            };
            println!("counter is {}", counter);
            println!("result is {}", result);

	    }
	

        // ------------------------------ testing trait Copy and Clone for closure ------------------------------
        let outside_num = 353;
            let callback = move |num: i32| {
                let got_outside_num = outside_num;
                let copy_of_num = num; //// trait Copy is implemented for i32 thus has trait Clone so we don't need to clone it and we can also access it after it's moved into copy_of_num var 
            };

        // ------------------------------ testing trait Copy and Clone for i32 and String/str ------------------------------
        let name = String::from("wildonion");
        let name_slice = &name[0..3]; // pointing to an offset on the heap by borrowing some parts of the name String
        let anot_cop_of_slice = name_slice; // this is ok cause the Copy trait is implemented for &T which is &str in here
        // NOTE - we have still access to name_slice in here
        // ...
        // this is no ok cause name is on the heap with a saved reference to the heap on the stack also it doesn't implement Copy trait
        // the Clone trait is implemented for that because of double free pointer issue at runtime and the implementation of drop trait.
        // let another_name = name;
        // println!("name is droped {:?}", name); 
        let another_name = name.clone(); // we used the clone method here to copy the whole the reference on the stack and the whole data on the heap as well 
        let another_name = &name; // this is ok cause the Copy trait is implemented for &T which in our case is &String which is coerced &str or string slice which is saved somewhere in the memory(heap, stack or binary)
        let number: i32 = 3534;
        let another_number = number; // this is ok cause the number it's on the stack thus the drop trait is not implemented for that(still got the number even it's moved) so we can copy the whole number variable into another_number

        // ------------------------------ testing trait Copy and Clone for u8 and Vec<u8> ------------------------------
        // u8 implements Copy
        let x: u8 = 123;
        let y = x;
        // x can still be used
        println!("x={}, y={}", x, y);

        // Vec<u8> implements Clone, but not Copy
        let v: Vec<u8> = vec![1, 2, 3];
        let w = v.clone();
        //let w = v // This would *move* the value, rendering v unusable.

        // ------------------------------ testing trait Copy and Clone for structs ------------------------------
        #[derive(Debug, Clone, Copy)]
        pub struct PointCloneAndCopy {
            pub x: f64,
        }

        #[derive(Debug, Clone)]
        pub struct PointCloneOnly {
            pub x: f64,
        }

        fn test_copy_and_clone() {
            let p1 = PointCloneAndCopy { x: 0. };
            let p2 = p1; // because type has `Copy`, it gets copied automatically.
            println!("{:?} {:?}", p1, p2);
        }

        fn test_clone_only() {
            let p1 = PointCloneOnly { x: 0. };
            // let p2 = p1; // because type has no `Copy`, this is a move instead. to avoid moving we can clone the p1
            // println!("{:?} {:?}", p1, p2);
        }

	

        // reading image pixels or bytes which is utf8 and each pixel is between 0 up to 255
        // ...
        if let Ok(bytes) = fs::read("/home/wildonion/Pictures/test.jpg"){
            println!("image bytes >>>> {:?}", bytes);
        }

	

        'outer: for x in 0..5 {
            'inner: for y in 0..5 {
                println!("{},{}", x, y);
                if y == 3 {
                    break 'outer;
                }
            }
        }


    // 	::::::::::iterator for struct::::::::::
	struct Alternate {
	    state: i32,
	}

	impl Iterator for Alternate {
	    type Item = i32;

	    fn next(&mut self) -> Option<i32> {
            let val = self.state;
            self.state = self.state + 1;

            // if it's even, Some(i32), else None
            if val % 2 == 0 {
                Some(val)
            } else {
                None
            }
	    }
	}

	let mut iter = Alternate { state: 0 };

	// we can see our iterator going back and forth
	assert_eq!(iter.next(), Some(0));
	assert_eq!(iter.next(), None);
	assert_eq!(iter.next(), Some(2));
	assert_eq!(iter.next(), None);




    // =============================================================================================================================
    //// in order to change the content of a type using its pointer we have to define the pointer as mutable
    /*
	
        let mut my_name = "Pascal".to_string();
        my_name.push_str( " Precht");
        let last_name = &my_name[7..];
        
        
        
                         buffer
                        /    capacity
                       /    /   length
                      /    /   /
                    +–––+–––+–––+
        stack frame │ • │ 8 │ 6 │ <- my_name: String
                    +–│–+–––+–––+
                      │
                    [–│–––––––– capacity –––––––––––]
                      │
                    +–V–+–––+–––+–––+–––+–––+–––+–––+
               heap │ P │ a │ s │ c │ a │ l │   │   │
                    +–––+–––+–––+–––+–––+–––+–––+–––+

                    [––––––– length ––––––––]
                    
                    
                    
        Notice that last_name does not store capacity information on the stack. 
        This is because it’s just a reference to a slice of another String that manages its capacity. 
        The string slice, or str itself, is what’s considered ”unsized”. 
        Also, in practice string slices are always references so their type will always be &str instead of str.
                    
                    

                    my_name: String   last_name: &str
                    [––––––––––––]    [–––––––]
                    +–––+––––+––––+–––+–––+–––+
        stack frame │ • │ 16 │ 13 │   │ • │ 6 │ 
                    +–│–+––––+––––+–––+–│–+–––+
                      │                 │
                      │                 +–––––––––+
                      │                           │
                      │                           │
                      │                         [–│––––––– str –––––––––]
                    +–V–+–––+–––+–––+–––+–––+–––+–V–+–––+–––+–––+–––+–––+–––+–––+–––+
               heap │ P │ a │ s │ c │ a │ l │   │ P │ r │ e │ c │ h │ t │   │   │   │
                    +–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+–––+
                    
                    

        string literals are a bit special. They are string slices that refer to “preallocated text” 
        that is stored in read-only memory as part of the executable. In other words, 
        it’s memory that ships with our program and doesn’t rely on buffers allocated in the heap.
        that said, there’s still an entry on the stack that points to that preallocated memory when the program is executed:

        
        let my_name = "Pascal Precht";
        
        
                    my_name: &str
                    [–––––––––––]
                      +–––+–––+
        stack frame   │ • │ 6 │ 
                      +–│–+–––+
                        │                 
                        +––+                
                            │
            preallocated  +–V–+–––+–––+–––+–––+–––+
            read-only     │ P │ a │ s │ c │ a │ l │
            memory        +–––+–––+–––+–––+–––+–––+
        
        
                    
			    
	*/
	let first_name = "Pascal"; // str - &str is a reference to String some where in the heap
    let last_name = "Precht".to_string(); // turn to String
    let another_last_name = String::from("Precht");
    greet(first_name); // first_name is &str by default
    greet(&last_name); // last_name is passed by reference
    greet(&another_last_name); // another_last_name is passed by reference

    fn greet(name: &str) {
        println!("Hello, {}!", name);
    }

        
    let name = String::from("erfan"); // String
    let another_name = "another erfan"; // str
    // let combined = name + &another_name;
    // name.push_str(&another_name); // name moved due to above operator
    // println!("{}", combined);
    // println!("{}", name); // error - borrowed after move
    println!("{}", another_name);

    let sample_string = String::from("wildonion");
    let bytes = sample_string.bytes(); // turn a string into buffer (asccii)
    println!("[..] two first bytes of the string are : {}", &sample_string[0..2]); // byte indices
    println!("[..] the string bytes : {:?}", bytes);

    let text = "hello hello from wildonion here double again again wildonion";
    let mut map = HashMap::new();
    for word in text.split_whitespace(){
        let count = map.entry(word).or_insert(0); // return a mutable reference inserted or the old value
        *count += 1; // updating the old value by dereferencing it, cause count is a mutable reference of the value 
    }

    println!("{:?}", map);

    
    let simulated_user_specified_value = 10;
    let simulated_random_number = 7;
    generate_workout(simulated_user_specified_value, simulated_random_number);
	



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
    /////////////////////////////////////////////////////////////////
    //// since rust doesn't have gc thus by moving a type into a new scope 
    //// its lifetime will be dropped unless it implements the Copy trait 
    //// otherwise if it's a heap data we must either clone it or borrow 
    //// it thus we must alwasy pass by reference to the data and note that 
    //// the heap data can be in their borrowed form like &[u8] and &str 
    //// also we can return a pointer to the type from a method by using 
    //// a specific or the self lifetime, note that we can also put the 
    //// sized array behind a pointer like Option<&[u8; 64]>
    /////////////////////////////////////////////////////////////////
    // NOTE - every pointer in rust must have valid lifetime thus if 
    //        we want to use & in return type of a method or struct 
    //        field we should have a valid lifetime for that.
    // https://stackoverflow.com/a/57894943/12132470
    // https://stackoverflow.com/questions/37789925/how-to-return-a-newly-created-struct-as-a-reference
    //// since rust doesn't have gc thus using value in other scopes we must notice that:
    ////     - value will be moved by default if it's a heap data and their previous lifetime will be dropped
    ////     - value will be copied by default if it's a stack data and we have them in other scopes
    ////     - note that we can't borrow the value after it has moved
    ////     - note that we can't move the value if it 
    ////            - is behind a shared pointer or borrowed since the pointer of that might convert into a dangling pointer once the value gets dropped
    ////            - doesn't implement the Copy trait
    ////     - note that we borrow the value because 
    ////            - its size can't be known at compile time
    ////            - don't want to lose its ownership later
    //// which in order to not to lose the ownership of heap data we can either pass their 
    //// clone or their borrowed form or a pointer of them, note that if we clone them the main 
    //// value won't be updated since clone will create a new data inside the heap also heap 
    //// data sized can be in their borrowed for or behind a pointer like &str for String and 
    //// &[u8] or &[0u8; SIZE] for Vec if we care about the cost of the app.  
    //
    //// based on borrowing and ownership rules in rust we can't move a type into new scope when there
    //// is a borrow or a pointer of that type exists, rust moves heap data types by default since it 
    //// has no gc rules means if the type doesn't implement Copy trait by moving it its lifetime will 
    //// be dropped from the memory and if the type is behind a pointer rust doesn't allow the type to 
    //// be moved, the reason is, by moving the type into new scopes its lifetime will be dropped 
    //// accordingly its pointer will be a dangling one in the past scope, to solve this we must either 
    //// pass its clone or its borrow to other scopes. in this case self is behind a mutable reference 
    //// thus by moving every field of self which doesn't implement Copy trait we'll lose the ownership 
    //// of that field and since it's behin a pointer rust won't let us do that in the first place which 
    //// forces us to pass either its borrow or clone to other scopes. 
	impl Pack{ ////// RETURN BY POINTER EXAMPLE ////// 


	    fn new() -> Self{


            pub struct TupleStudent(pub String, pub u8);

            let stu_info = TupleStudent("wildonion".to_string(), 26);
            let TupleStudent(name, age) = stu_info;

            let name = Some("wildonion".to_string());
            struct User{
                username: String,
                age: u8,
            }

            let user = User{
                username: match name.clone(){ //// clone it here to be able use it in another_user instance
                    Some(name) => name, 
                    None => "".to_string(),
                },
                age: 26,
            };

            let another_user = User{
                username: match name{
                    Some(name) => name,
                    None => "".to_string(),
                },
                ..user //// filling the remaining fields with other User instance
            };

            // let User{username, age} = user; //// unpacking struct
            let User{username: name, age: sen} = user; //// unpacking struct with arbitrary field names
            // let User{..} = user; //// unpacking struct with `..` since we don't care about all fields
            
            let hello = "Здравствуйте";
            let s = &hello[0..2];
            // every index is the place of an element inside the ram which has 1 byte size which is taken by that element
            // in our case the first element takes 2 bytes thus the index 0 won't return 3 
            // cause place 0 and 1 inside the ram each takes 1 byte and the size of the
            // first element is two bytes thus &hello[0..2] which is index 0 and 1 both returns 3 
            // and we can't have string indices in rust due to this reason!


            ///////////////////////////////////////////// ENUM MATCH TEST
            #[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
            enum Chie{
                Avali(u8),
                Dovomi(String),
                Sevomi,
                Chaharomi{ //// enum variant can also be a struct
                    name: String,
                    age: u32,
                }
            }


            let ine = Chie::Avali(12); //// the Dovomi variant is never constructed cause we've used the first variant  

            match ine{
                Chie::Avali(value) if value == 23 => { //// matching on the Avali arm if the value was only 23
                println!("u8 eeee");

                },
                Chie::Dovomi(value) if value == "wildonion".to_string() => { //// matching on the Dovomi arm if the value was only "wildonion" string
                println!("stringeeee");
                },
                Chie::Chaharomi{name, ..} => { //// we only care about name and the rest of field will be filled by `..`

                },
                Chie::Chaharomi{name, age} => { //// using its own fields' names for unpacking on struct arm

                },
                Chie::Chaharomi{name: esm, age: sen} => { //// we can also give another names to the current struct fields using `:` for unpacking on struct arm

                },
                Chie::Chaharomi{name: esm, ..} => { //// we can also give another names to the current struct fields using `:` for unpacking on struct arm also we don't care about age field thus we can fill it up using `..`

                },
                _ => { //// for Sevomi fields
                println!("none of them");
                }
            }

            // --------------- CODEC OPS ON ENUM ---------------
            let encoded = serde_json::to_vec(&Chie::Sevomi); ////// it'll print a vector of utf8 encoded JSON
            let decoded = serde_json::from_slice::<Chie>(&encoded.as_ref().unwrap()); //// as_ref() returns a reference to the original type

            let encoded_borsh = Chie::Sevomi.try_to_vec().unwrap(); ////// it'll print 2 cause this the third offset in memory
            let decoded_borsh = Chie::try_from_slice(&encoded_borsh).unwrap();

            /////////////////////////////////////////////
            Pack{}
	    }

        // we can return a pointer to struct 
        // by returning &Struct{} from the method
        // since by doing this we're allocating 
        // nothing on the stack and the allocation
        // will be done once the caller gets the 
        // returned data from the function.
	    fn ref_struct(num_thread: &u8) -> &Pack{ //// returning ref from function to a pre allocated data type (not inside the function) Pack struct in our case, is ok
            let instance = Pack::new(); //// since new() method of the Pack struct will return a new instance of the struct which is allocated on the stack and is owned by the function thus we can't return a reference to it or as a borrowed type because it's owned by the function scope
            // &instance //// it's not ok to return a reference to `instance` since `instance` is a local variable which is owned by the current function and its lifetime is valid as long as the function is inside the stack and executing which means after executing the function its lifetime will be dropped
            let instance = &Pack{}; //// since we're allocating nothing on the stack inside this function thus by creating the instance directly using the the Pack struct and without calling the new() method (which is already lives in memory with long enough lifetime) we can return a reference to the location of the instance of the pack from the function and the reference will be stored inside the caller (where this function has called)
            instance //// it's ok to return a reference to `instance` since the instance does not allocate anything on the stack thus taking a reference to already allocated memory with long enough lifetime is ok since the allocated memory is happened in struct definition line
	    }

        // struct Taker{}
        // fn run_taker_mut(taker: &mut Taker) -> &mut Taker{
        //     //// for mutable reference the underlying type must be mutable
        //     //// thus rust will allocate mut a temp Taker first in the ram 
        //     //// (stack or heap depends on the types of the Taker struct) and when 
        //     //// we want to return &mut Taker it'll return a mutable pointer
        //     //// to the temp value inside the ram which is owned by the current 
        //     //// function but it's ok to return &Traker since rust allocate no
        //     //// space inside the ram for this and directly will return the Taker
        //     //// struct on the fly to the caller
        //     let oochik = &mut Taker{}; 
        //     oochik
        //     // or
        //     // &mut Taker{} 
        // } 


	    // NOTE - argument can also be &mut u8
	    pub fn ref_str_other_pointer_lifetime(status: &u8) -> &str{ //// in this case we're good to return the pointer from the function or copy to the caller's space since we can use the lifetime of the passed in argument, the status in this case which has been passed in by reference from the caller and have a valid lifetime which is generated from the caller scope by the compiler to return the pointer from the function
            let name = "wildonion";
            name //// name has a lifetime as valid as the passed in status argument lifetime from the caller scope 

	    }

        fn run_taker(taker: &mut Commander) -> &Commander{
            let instance = &Commander{};
            instance
            // or
            // &Commander{} 
        }
        pub fn ref_to_str<'a>() -> &'a str{ //// we can't return &str since we need a lifetime to do so
            let name = "wildonion";
            name
        }

        fn ret_taker_mut(taker: &mut Commander) -> &mut Commander{
            taker //// we're good to return a pointer to the taker since is not owned by the function 
        }  

        // pub fn ref_to_str() -> HashMap<&str, &str>{ //// we can't return &str since we need a lifetime to do so
        //     let names = HashMap::new();
        //     names.insert("wildonion", "another_wildonion");
        //     names
        // }

        //// in here we're actually implementing the trait for the return type
        //// also the return type must implement the Interface trait in order 
        //// to be able to return its instance,  
        pub fn ref_to_trait(&self) -> &dyn Interface{
            &Pack{}
        }

        pub fn ref_to_trait__(&self) -> &dyn Interface{
            self //// can't return self in here 
        }

	    // NOTE - first param can also be &mut self; a mutable reference to the instance and its fields
	    // NOTE - this technique is being used in methods like as_mut() in which it'll return a mutable
        //        reference to the data using the self parameter lifetime.
        pub fn ref_to_str_other_self_lifetime(&self) -> &str{ //// in this case we're good to return the pointer from the function or send a copy to the caller's space since we can use the lifetime of the first param which is &self which is a borrowed type (it's a shared reference means that other methods are using it in their scopes) of the instance and its fields (since we don't want to lose the lifetime of the created instance from the contract struct after calling each method) and have a valid lifetime (as long as the instance of the type is valid) which is generated from the caller scope by the compiler to return the pointer from the function
            let name = "wildonion";
            name //// name has a lifetime as valid as the first param lifetime which is a borrowed type (it's a shared reference means that other methods are using it in their scopes) of the instance itself and its fields and will borrow the instance when we want to call the instance methods
	    }

	    // NOTE - 'a lifetime has generated from the caller scope by the compiler
	    pub fn ref_to_str_specific_lifetime<'a>(status: u8) -> &'a str{ //// in this case we're good to return the pointer from the function or copy to the caller's space since we've defined a valid lifetime for the pointer of the return type to return the pointer from the function which &'a str
            let name = "wildonion";
            name //// name has a lifetime as valid as the generated lifetime from the caller scope by the compiler and will be valid as long as the caller scope is valid
	    }

        // NOTE - use 'static lifetime in order to be able to return &str from the function since rust doesn't allow to return reference by default unless the return type has a valid and defined lifetime
	    // NOTE - 'static lifetime will be valid as long as the whole lifetime of the caller scope (it can be the main function which depends on the whole lifetime of the app)
	    pub fn ref_to_str_static() -> &'static str{
            let name = "wildonion";
            name //// name has static lifetime valid as long as the whol lifetime of the caller scope which can be the main function which will be valid as long as the main or the app is valid
	    }
		
        // fn ret<'a>(name: String) -> &'a Vec<String>{
        //     //// this type is owned by the current function 
        //     //// thus if there is any pointer of this type 
        //     //// exists we can't return that pointer since 
        //     //// once the function gets executed all the types
        //     //// inside the function will be dropped from the ram 
        //     //// and any pointer to them will be dangled.
        //     //
        //     //// we can't return a pointer to the String 
        //     //// from the function since Strings or Vecs
        //     //// are heap data types and once the function 
        //     //// gets executed their lifetime will be dropped
        //     //// from the ram to free the allocations and 
        //     //// because of this returning a pointer to them 
        //     //// might be a dangling pointer which rust doesn't
        //     //// allow us to do this in the first place.
        //     // let names = vec!["wildonion".to_string()];
        //     // &names
        // }

	    //// ERROR - can't return a reference to heap allocated data structure from function due to their unknown size at compile time and they are temprary value
	    // pub fn ref_to_string<'s>() -> &'s String{
	    //     let name = &"wildonion".to_string();
	    //     name //// ERROR - we can't return this or &"wildonion".to_string() since they are temporary value due to the fact that heap data structure's size are not specific at compile time and they are some kina a temporary value thus heap data structures can't be returned in their borrowed form from the function since their size are not specific at compile time therefore by taking a pointer to the location of them we might have dangling pointer later once their location gets dropped during the function lifetime body 
	    // }

	    pub fn ref_to_num<'n>() -> &'n i32{
            let num = 23;
            // &num //// ERROR - we can't return this since the num is owned by the current function and returning the reference to the local variable which is owned by the function is denied
            &23 //// we can return &23 since we did allocate nothing on the stack inside the function (which this can be done by creating a local variable inside the function) and we're just returning a pointer to the location of a number directly   

	    }

        // NOTE - here we couldn't return its &str since this is 
        //        owned by the function and its lifetime will be dropped once the function 
        //        gets executed thus we can't return a pointer to &str or its utf8 bytes 
        //        because its pointer might be a dangling one in the caller space since 
        //        we don't have that String anymore inside the function! this is different
        //        about the &str in the first place cause we're cool with returning them
        //        because they are behind a pointer and kinda stack data types.
        pub const fn test(name: &String) -> &str{ // we can return &str in here sicne we're using the lifetime of the passed in param which is &String thus it's ok to use that reference (the reference to the passed in String) to return a &str (since its lifetime is valid as long as the passed in param is valid)
            WO // we must return const value from the constant function
        }

        pub fn run() -> impl std::future::Future<Output=u8>{ //// implementing the Future trait for the return type of the function by doing this we have to return an async block from the function
            async move{ //// returning an async block from the function
                26
            }

            // let res = run.await;
        }

        pub async fn _run() -> u8{ //// above implementation is equivalent to this one 
            26

            // let res = run.await;
        }

        pub async fn unpack_self(&self) -> (){
            let Pack{..} = self; //// unpacking self into the struct itself, there is no explicit field naming and we filled all the fields using `..`
        }

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
    

	
    enum MyResult<R, E>{
        Result(R),
        Err(E),
    }


    fn run(id: u8) -> MyResult<u8, i32>{
        MyResult::Result(1) 
        // if there was something went wrong we can return MyResult::Err(1);
        // ...
    } 

	
    





    
    // =============================================================================================================================
    //-------------------------- let else
    fn get_count_item(s: &str) -> (u64, &str) {
        let mut it = s.split(' ');
        let (Some(count_str), Some(item)) = (it.next(), it.next()) else {
            panic!("Can't segment count item pair: '{s}'");
        };
        let Ok(count) = u64::from_str_radix(count_str, 10) else {
            panic!("Can't parse integer: '{count_str}'");
        };
        (count, item) // we can return them since their scopes didn't dropped when we're using let else
        
        // -------- using if let
        // --------------------------------
        // let (count_str, item) = match (it.next(), it.next()) {
        //     (Some(count_str), Some(item)) => (count_str, item),
        //     _ => panic!("Can't segment count item pair: '{s}'"),
        // };
        // let count = if let Ok(count) = u64::from_str(count_str) {
        //     count
        // } else {
        //     panic!("Can't parse integer: '{count_str}'");
        // };
        // --------------------------------
        
    }
    assert_eq!(get_count_item("3 chairs"), (3, "chairs"));
    // =============================================================================================================================




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

    fn Ankir(name: impl Interface){ //// implementing the Interface trait for the passed in type means that we're bounding the passed in type to this trait

    }

    let network: Box<dyn FnMut(String) -> std::pin::Pin<Box<dyn std::future::Future<Output=String>>> + Send + Sync + 'static> =
        Box::new(|addr: String|{
            Box::pin(async move{
            addr
        })
    });
	
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
    //      - in function return and param type (: impl Trait or -> impl Trait)
    //      - derive like macro on top of the struct or types
    //      - directly by using impl Trait for Type in other scopes
    // returning traits from the function or us it as a function param by:
    //      - Box<dyn Trait>
    //      - &dyn Trait  
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
    fn test_n() -> Box<impl InterfaceMe>{
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
        Box::pin(async {
            name
        })
    };
    let clsMe = |name: String| Box::pin(async{ //// since the return type is a Pin<Box<T>> there is no need to put the Box::pin inside curly braces since it's a single line code logic
        name
    });
    //----------------------------

    
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
    // =============================================================================================================================

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

    // ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
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







//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////

pub async fn unsafer(){

    // https://stackoverflow.com/questions/41823321/how-to-get-pointer-offset-of-an-enum-member-in-bytes
    // https://fasterthanli.me/articles/peeking-inside-a-rust-enum
    // https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#the-misconceptions
    // https://stackoverflow.com/questions/57754901/what-is-a-fat-pointer
    ////////////////////////////////////////////////////////////////
    //                      fat pointer examples
    ////////////////////////////////////////////////////////////////
    // accessing the value of a pointer is unsafe and we have to use unsafe{*pointer}
    // pointers to the dynamic size types like traits and slices like &str and &[T] are fat pointers 
    // they contains extra bytes (or usize) to store the length of the referenced type and because 
    // of this the address of the other objects and pointers are different with the traits addresses since 
    // the address of traits contains 16 hex chars which will be 16 * 4 bits or 8 bytes long hence the pointer 
    // of none traits and traits are not equals.
    struct W(i32);
    let w = W(324);
    let a = &w as &dyn Sync; //// casting the &w to a reference to the Sync trait and since traits have no size thus we have to use &dyn or *const dyn Trait for raw pointer casting; we can cast here since trait Sync is implemented for the W struct (if we want to cast to a trait the trait must be implement it for the type)
    let b = &w.0 as &dyn Sync; //// casting the &w.0 to a reference to the Sync trait and since traits have no size thus we have to use &dyn or *const dyn Trait for raw pointer casting; we can cast here since trait Sync is implemented for the W struct (if we want to cast to a trait the trait must be implement it for the type) 
    // it'll return false since `a` is pointing to the location of `w` which is an instance of the `W` 
    // and `b` is pointing to the location of 123 which these are located in two different areas inside the stack.
    // also objects and pointers have equal addresses but traits have different ones
    println!("{}", std::ptr::eq(a, b)); //// comparing two raw pointers by they address (if they are raw compiler will coerce them into raw like *const T)
    let a = &w as &dyn Sync; //// Sync trait has implemented for the W so we can cast its instance to the trait and `a` will be a fat pointer
    let b = &w as *const dyn Sync;
    let c = &w as *const W as *const dyn Sync as *const u8;
    //// 0x00007ffe673ace7c is not the same as 0x7ffe673ace7c ////
    println!("{:#?}", c); //// 0x00007ffe673ace7c : this pointer is 8 bytes (every 2 char in hex is 1 byte and every char is 4 bits thus 16 chars has 64 bits or 8 bytes long) long since it's pointing to the location of a trait
    println!("{:#?}", &c); //// 0x7ffe673ace7c : this will print the address of &w or c which is casted to a *const u8 it has no extra size at the begening like the above address since it's not a dynmic size type like above Sync trait but the rest is the same of above trait address    
    println!("{:p}", c); //// 0x7ffe673ace7c : this will print the address of &w or c which is casted to a *const u8 it has no extra size at the begening like the above address since it's not a dynmic size type like above Sync trait but the rest is the same of above trait address  
    println!("{:p}", &c); //// 0x7ffe673ace80 :  this is the address of the c pointer or &c (address of a the c pointer inside the stack)


    struct Wrapper { member: i32 }
    trait Trait {}
    impl Trait for Wrapper {}
    impl Trait for i32 {}
    let wrapper = Wrapper { member: 10 };

    //// raw pointers and objects have equal addresses in their size
    assert!(std::ptr::eq(
        &wrapper as *const Wrapper as *const u8, //// to convert an instance to the u8 we have to first convert it to the struct itself first
        &wrapper.member as *const i32 as *const u8 //// to convert the number to the u8 we have to first convert it to the i32 itself first
    ));

    //// `Trait` has different implementations since their pointers will store extra size about their length (because their size will be specified at runtime and depends on their implementor) so they have 8 bytes long
    assert!(!std::ptr::eq(
        &wrapper as &dyn Trait, //// casting the wrapper to the Trait trait we can do this, since Trait is implemented for the underlying type or Wrapper struct (if we want to cast to a trait the trait must be implmeneted for the type) 
        &wrapper.member as &dyn Trait,
    ));
    assert!(!std::ptr::eq(
        &wrapper as &dyn Trait as *const dyn Trait, //// we have to first cast the wrapper into the &dyn Trait then into the raw pointer of the dyn Trait
        &wrapper.member as &dyn Trait as *const dyn Trait,
    ));

    assert!(std::ptr::eq(
        &wrapper as &dyn Trait as *const dyn Trait as *const u8, //// casting wrapper to &dyn Trait first then to raw pointer of dyn Trait finally to raw pointer of u8
        &wrapper.member as &dyn Trait as *const dyn Trait as *const u8,
    ));
    ////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////

    use std::cell::RefCell;
    use std::rc::Rc;
    let datarefcell: Rc<RefCell<&'static [u8; 64]>> = Rc::new(RefCell::new(&[0u8; 64]));
    //// mutably borrow the type inside of RefCell will be done using RefMut struct
    //// which takes the type and a valid lifetime for borrowing since every borrow or 
    //// pointer of a type needs to be valid with a lifetime until it gets dropped
    let lam = **datarefcell.borrow_mut(); //// double dereference to get the [0u8l 64] which has 64 bytes data 


    ///// -------------- unsafe: changing the vaule in runtime using its pointer -------------- /////
    let v = vec![1, 2, 3];
    // let raw_parts = v.into_raw_parts(); //// getting the pointer, len and capacity of the vector only in unstable rust! 
    let mut v = std::mem::ManuallyDrop::new(v); // a wrapper to inhibit compiler from automatically calling T’s destructor, this wrapper is 0-cost
    let pointer = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    unsafe{
        for i in 0..len as isize{
            std::ptr::write(pointer.offset(i), 4 + i);
        }
    
        let rebuilt = Vec::from_raw_parts(pointer, len, cap);
        assert_eq!(rebuilt, [4, 5, 6]);
    }
    
    let mut a = "another_wo".to_string();
    // changing the value of 'a' by changing the offset of its u8 raw pointer
    let new_a = unsafe{
        //         ptr      len  capacity
        //       +--------+--------+--------+
        //       | 0x0123 |      2 |      4 |
        //       +--------+--------+--------+
        //             |
        //             v
        // Heap   +--------+--------+--------+--------+
        //       |    'a' |    'n' | uninit | uninit |
        //       +--------+--------+--------+--------+
        let ptr_of_a = a.as_mut_ptr(); // *mut u8 - a pointer to the code ascii of the String
        let len = a.len();
        let cap = a.capacity();
        std::ptr::write(ptr_of_a.offset(5), 117); // 117 is the code ascii of `u`
        String::from_raw_parts(ptr_of_a, len, cap)
        // let mut changed_offset = ptr_of_a.offset(5);
        // String::from_raw_parts(changed_offset, len, cap)
        
    };

    println!("new `a` is {}", new_a);

    ///// -------------- union, enum and struct -------------- /////
    //// offset is the size in bytes and in order to get
    //// the starting offset of a type or struct instance 
    //// we can get a raw pointer (since smart pointer in rust 
    //// will be coerced to raw pointers at compile time) 
    //// to the instance then cast that pointer to usize 
    //// which is the size in bytes of the instance pointer itself
    //
    //// a pointer contains the memory address of an obejct
    //// and it has either 32 or 64 bits (depends on the os arc)
    //// size hence we can get it's size by casting it into the 
    //// usize type that contains the size of that pointer in bytes
    //// inside the stack
 
    struct Object{
        a: u8, //// we can fill this in a hex form
        b: u16, //// we can fill this in a hex form
        c: u32, //// we can fill this in a hex form
    }

    let obj = Object{
        //// since `a` field is of type u8 thus we have to fill 
        //// it with only two chars in hex since every 4 bits 
        //// in base 2 is a hex char; the 0xaa is 170 in decimal
        //// 0xaa is one byte or 8 bits
        a: 0xaa, 
        //// since `b` field is of type u16 thus we have to fill 
        //// it with only four chars in hex since every 4 bits 
        //// in base 2 is a hex char; the 0xaa is 48059 in decimal
        //// 0xbbbb is two bytes or 16 bits
        b: 0xbbbb, 
        //// since `c` field is of type u32 thus we have to fill 
        //// it with only eight chars in hex since every 4 bits 
        //// in base 2 is a hex char; the 0xcccccccc is 3435973836 in decimal
        //// 0xcccccccc is two bytes or 32 bits
        c: 0xcccccccc
    };

    //// usize is an unsigned size which is big enough
    //// to store any pointer and in 32 bits arch is 4 bytes
    //// and in 64 bits is 8 bytes also each usize contains 
    //// the size in bytes in either 32 or 64 bits format
    //
    //// base is the usize pointer of the object itself 
    //// which contains the size of the starting offset 
    //// in bytes in memory, we've just cast the pointer 
    //// to the location of the obj instance into the usize
    //// to get the size of its pointer in bytes which is the 
    //// starting offset of all its fields
    let base = &obj as *const _ as usize; //// we're considering the pointer of the obj instance as the starting point of the offset by converting its pointer into usize 
    let a_off = &obj.a as *const _ as usize - base; //// this is the `a` field offset by subtracting its usize pointer (cast its *const pointer to usize) from the base offset
    let b_off = &obj.b as *const _ as usize - base; //// this is the `b` field offset by subtracting its usize pointer (cast its *const pointer to usize) from the base offset
    let c_off = &obj.c as *const _ as usize - base; //// this is the `c` field offset by subtracting its usize pointer (cast its *const pointer to usize) from the base offset
    println!("base: {}", base); 
    println!("a: {}", a_off as usize - base);
    println!("b: {}", b_off as usize - base);
    println!("c: {}", c_off as usize - base);

    enum MultiEnum{
        A(u32),
        B(f32, u64),
        C{x: u8, b: u16},
        D
    }

    let both = MultiEnum::B(2.5, 4);
    let strc = MultiEnum::C{x: 24, b: 25};

    #[repr(u32)]
    enum Tag{I, F}
    #[repr(C)]
    union U{
        i: i32,
        f: f32,
    }

    #[repr(C)]
    struct Value{
        tag: Tag,
        u: U,
    }

    fn is_zero(v: Value) -> bool{
        unsafe{
            match v{
                Value{tag: Tag::I, u: U{i: 0}} => true,
                Value{tag: Tag::F, u: U{f: num}} if num == 0.0 => true,
                _ => false,
            }
        }
    }

    ///// -------------- casting using raw pointer and transmute -------------- /////

    fn foo() -> i32{
        0
    }
    let pointer_to_func = foo as *const ();
    let func = unsafe{ // transmute the raw pointer of the function back into the function with i32 signature
        std::mem::transmute::<*const (), fn() -> i32>(pointer_to_func)
    };
    assert_eq!(func(), 0);


    let num_ = 10;
    let num_ptr: *const u8 = &num_; // ptr of num_
    let num = 10 as *const i32; // turn num into a constant raw pointer 
    let deref_raw_pointer_num = unsafe{&*num}; // dereferencing the raw pointer
    

    let mut name_ptr: *const u8;
    name_ptr = std::ptr::null(); // fill it with null pointer
    let name: &str = "wildonion";
    name_ptr = name.as_ptr(); // fill it with name bytes



    let c_const_pointer = 32 as *const i16;
    let c_mut_pointer = 64 as *mut i64;
    let c_const_pointer = c_mut_pointer.cast_const(); // casting the c raw mutable pointer into a constant one
    let thing1: u8 = 89.0 as u8;
    assert_eq!('B' as u32, 66);
    assert_eq!(thing1 as char, 'Y');
    let thing2: f32 = thing1 as f32 + 10.5;
    assert_eq!(true as u8 + thing2 as u8, 100);


    ///// ------------------------------------------------------------------------------- /////
    ///// &T and &mut T will be coerced into raw pointers *const T or *mut T respectively
    ///// ------------------------------------------------------------------------------- /////
    let mut name = String::from("wildonion");
    println!("`name` value before changing the value of `mut_smart_pointer_to_name` pointer >>>> {}", name);
    println!("`name` has the address >>>> {:p}", &name);
    // NOTE - trait Copy is not implemented for &mut T because if we had the Copy trait for them we could have multiple copy of them 
    //        in every where and they could change the data of what they have borrowed for thus the reference count of each variable 
    //        would be out of control inside the current thread, those references don't want a copy of data, they want to own and mutate 
    //        the data inside what they have borrowed thus we can't mutate the variable itself while its mutable pointer is changing the data inside of it.
    let mut raw_mut_pointer_to_name = &mut name as *mut String;
    let mut another_raw_mut_pointer_to_name = &mut name as *mut String;
    let mut mut_smart_pointer_to_name = &mut name; // copying `name` into `mut_smart_pointer_to_name` - we can only have one mutable reference or borrower in a same scope
    println!("`mut_smart_pointer_to_name` has the address >>>> {:p}", &mut_smart_pointer_to_name);
    println!("`raw_mut_pointer_to_name` value is the address of `name` >>>> {:p}", raw_mut_pointer_to_name);
    println!("`another_raw_mut_pointer_to_name` value is the address of `name` >>>> {:p}", another_raw_mut_pointer_to_name);
    println!("`raw_mut_pointer_to_name` address is >>>> {:p}", &raw_mut_pointer_to_name);
    println!("`another_raw_mut_pointer_to_name` address is >>>> {:p}", &another_raw_mut_pointer_to_name);
    // NOTE - can't assign to `name` in this scope cause it's borrowed by `mut_smart_pointer_to_name`
    // NOTE - can't mutate the `name` when there is another variable or a pointer pointing to the `name`
    // NOTE - `mut_smart_pointer_to_name` pointer has the ability to change the value of `name`  
    *mut_smart_pointer_to_name = "another_wildonion".to_string(); // we can rewrite the data which `mut_smart_pointer_to_name` is refering to, cause `name` is mutable - change the value of a smart pointer by dereferencing it
    println!("name value after changing the value of its smart pointer or borrower >>>> {}", name);
    println!("`name` address after changing its value using `mut_smart_pointer_to_name` pointer >>>> {:p}", &name);
    // NOTE - we can assign to `name` after dereferencing the `mut_smart_pointer_to_name` pointer
    name = "third_wildonion".to_string(); // raw pointers will change also
    println!("`raw_mut_pointer_to_name` value after changing name value >>>> {}", unsafe{&*raw_mut_pointer_to_name});
    println!("`another_mut_pointer_to_name` value after changing name value >>>> {}", unsafe{&*another_raw_mut_pointer_to_name});
    // NOTE - can't mutate the `name` when it's behind the mutable pointer, the only way of changing its vlaue is changing its pointer value
    // NOTE - cannot assign to `name` if the following is uncommented because `name` is borrowed using a pointer
    //        and we are trying to print that pointer when we are changing the `name` value at the same time otherwise
    //        can't assign to `name` when we are using its mutable pointer, since based on multiple mutable references can't
    //        be exist at the same time thus we can't have the `name` and its pointer at the same time for writing and reading.
    // println!("`mut_smart_pointer_to_name` value after changing `name` value >>>> {}", mut_smart_pointer_to_name); 



    let mut a = String::from("wildonion");
    println!("value of `a` before changing >>>> {}", a);
    println!("the address of `a` >>>> {:p}", &a);
    let mut c = &mut a as *mut String; // mutable raw pointer to `a` - coercing &mut String into *mut String
    println!("`c` value >>>> {}", unsafe{&*c}); // `c` has the same value of `a` - we have to take a reference to dereferenced raw pointer cause *c is of type String which is not bounded to trait Copy thus we have to take a reference to it to move out of unsafe block
    println!("`c` contains the address of `a` >>>> {:p}", c);
    println!("`c` address >>>> {:p}", &c);
    a = String::from("another_wildonion"); // changing `a` will change the `c` value also
    println!("value of `a` after changing >>>> {}", a);
    println!("`c` value after changing `a` >>>> {}", unsafe{&*c});
    println!("`c` contains the address of `a` >>>> {:p}", c);
    println!("`c` address after changing `a` >>>> {:p}", &c);
    unsafe{*c = String::from("third_wildonion");} // changing `c` will change the `a` value also cause `a` is a mutable variable and `c` is a pointer to the `a`
    println!("`c` value after changing >>>> {}", a);
    println!("value of `a` after changing `c` >>>> {}", a);
    println!("`c` contains the address of `a` after changing its value >>>> {:p}", c);
    println!("`c` address after changing its value >>>> {:p}", &c);



    // NOTE - changing the value of the varibale using its pointer or its shallow copy and both the pointer and the object must be defined mutable
    // NOTE - making a deep copy from the object is done by cloning the object using clone() method (trait Clone must be implemented) to prevent double free pointer issue from happening
    let mut a = 32;
    let mut b = &mut a as *mut i32;
    println!("`b` value >>>> {}", unsafe{*b});
    println!("`a` address [{:p}] == `b` address [{:p}]", &a, b);
    a = 3535; //// `b` will be changed
    println!("`b` value >>>> {}", unsafe{*b});
    unsafe{*b = 2435;} //// `a` will be changed
    println!("`a` value >>>> {}", a);
    let deref_pointer = unsafe{&*b}; //// a pointer to the dereferenced const raw pointer to the `a`



    
    let mut g = 24;
    let mut v = &mut g;
    let mut m = &mut v;
    **m = 353; // it'll change the `v` first, since `v` is a mutable reference to `g` by changing its value the value of `g` will be changed
    println!("`v` after changing `m` >>>> {}", v);
    println!("`g` after changing `m` >>>> {}", g);
    g = 2435; // changing `g` value inside another scope and lifeimte
    println!("`g` after changing >>>> {}", g);
    // NOTE - can't mutate the `g` when it's behind the mutable pointer, the only way of changing its vlaue is changing its pointer value
    // NOTE - cannot assign to `g` if the following is uncommented because `g` is borrowed using a pointer
    //        and we are trying to print that pointer when we are changing the `g` value at the same time otherwise
    //        can't assign to `g` when we are using its mutable pointer, since based on multiple mutable references can't
    //        be exist at the same time thus we can't have the `g` and its pointer at the same time for writing and reading.
    // println!("`v` value after `g` value >>>> {}", v);
    // println!("`m` value after `g` value >>>> {}", m);



    let var = 242;
    let mut pointer_var = &var; // this immutable reference or borrower
    println!("`var` is not changed cause it's not mutable {}", var);
    // let changed = &2243253;
    // *pointer_var = *changed; // ERROR - even though `pointer_var` is mutable but can't rewrite the data which `pointer_var` is refering to cause `var` is not mutable
    // println!("`pointer_var` is not changed cause the data it referes (`var`) to it's not mutable {}", pointer_var);


    let mut x = 0;
    let mut y = &mut x; //// mutable borrow to x
    
    *y = 2; // since y is a reference in order to change its value we have to dereference it first
    
    // can't use both immutable and mutable borrow at the same time 
    // one of the following must be commented in order to compile correctly
    // since in logging the type will be passed in its borrowed form to 
    // the println!() macro thus in the first line of the following
    // x is passed immutably or &x to the println!() on the other hand in
    // the second line y is a mutable pointer of x means its a &mut x
    // thus we have two pointers of a same type one is immutable and the other
    // is mutable rust doesn't allow us to have immutable pointer when we're borrowing
    // it as mutable: cannot borrow `x` as immutable because it is also borrowed as mutable
    // println!("x is {}", x); // immutable logging
    println!("y is {}", y); // mutable pointer to x logging 


    let mut a = 32;
    println!("`a` address before change is ===== {:p}", &a);
    println!("`a` value before change is ===== {}", a);
    let mut b: *const i32 = &a; // const raw pointer to the location of a 
    println!("`b` address before changing `a` value is ===== {:p}", &b);
    a = 34; // b will change 
    println!("`a` value after change is ===== {}", a);
    println!("`a` address after change is ===== {:p}", &a);
    println!("`b` value after changing `a` is ===== {}", unsafe{&*b});
    println!("`b` address after changing `a` value is ===== {:p}", &b);
    println!("`a` address inside the `b` ===== {:p}", b);
    // unsafe{*b = 56}; // ERROR - even though `b` and `a` is mutable but can't rewrite the data which `b` is refering to cause its type is const and pointing to a const variable which can't be changed by this pointer
    // println!("`b` value after change is ===== {}", unsafe{&*b});
    // println!("`a` address inside the `b` after changing `b` value is not the same as the old one ===== {:p}", b);
    // println!("`a` value after changing `b` is ===== {}", a);
    // println!("`b` address after changing `b` value is ===== {:p}", &b);
    // a = 235;
    // println!("`b` value after changing `a` is ===== {}", unsafe{&*b});




    // ----------------
    //  BORROWING RULE
    // ---------------
    //// we can borrow in order to prevent from moving but 
    ////        - can't later move the type into new scope since it has a borrow, we can clone it or move its borrow 
    ////        - can't mutate the borrowed if it's not a mutable borrow 
    ////        - only one mutable borrow can be exist in a scope
    ////        - mutating the borrow needs the underlying data to be mutable too 
    //// in general we can't move a type into other scopes if 
    //// there is a pointer of it since by doing this the pointer
    //// might be converted into a dangling pointer and this is because
    //// rust doesn't support gc and by moving a dynamic size like string and vector
    //// into other scopes their lifetime will be dropped and in order to this 
    //// can be happened there must be no reference of them in pervious scopes.
    //
    //// also we can use Rc, Weak pointers to break some cycles like having a 
    //// field in struct of type struct itself by putting the field inside the 
    //// Rc which means that the filed has a strong reference  counter of all the type 
    //// that reference the field which doesn't allow to drop the field unless 
    //// its strong reference counter reaches zero, but this is not the case for Weak.
    //
    //// `pointer_to_name` can't be dereferenced because we can't move out of it to put it inside other types  
    //// since it's behind a shared reference of type `String` which doesn't implement 
    //// Copy trait also rust doesn't allow us to move a type into other scopes or types 
    //// when there is a borrowed type of that type is being used by other scopes since 
    //// if we move that its lifetime will be dropped and its pointer will be a dangling 
    //// pointer, pointing to a location which doesn't exist which is dropped, thus if 
    //// it's inside an Option we can't move out of it too since methods of Option<T> 
    //// is the same as `T` methods means that Copy is not implemented for `Option<String>` 
    //// we can either move it between threads and scopes by borrowing it or cloning it.
    //
    //// a shared reference can be in use by other scopes and threads thus moving out of 
    //// a shared referenced type requires one of the dereferencing methods which is either 
    //// copying (the Copy trait must be implemented), borrowing it using & or cloning 
    //// (Clone trait must be implemented) it or dereference it by using `*` otherwise 
    //// we can' move out of it in our case `whitelist_data` which is of type String, can't be moved 
    //// since `String` type doesn't implement Copy trait we have to either borrow it or clone it. 
    //
    //// if there is a pointer of the heap data exists we can't return it or move it into the other scope 
    //// since by moving heap data they will be dropped from the ram and their lifetime will be no longer exists
    //// thus in some case except method calls we can use its borrow or slice type or clone the reference to dereference or 
    //// convert it to owned using .to_owned() 

    let name = "wildonion".to_string();
    let pointer_to_name = &name; 
    // let another_name = name; //// Error in here: we can't move name into another type since it's borrowed above and its pointer might be being in used by other scopes thus rust doesn't allow us to this to avoid dangling pointer issues
    let another_name = &name; //// can't move name into another_name thus based on above we have to either clone or borrow it 
    //// if we want to use the pointer of the name vairable
    //// in other scopes like printing it we can't move name 
    //// variable into another_name variable since in the 
    //// following we're using its pointer which is borrowed 
    //// using the pointer_to_name variable that might be a 
    //// dangling pointer once the name variable moved into 
    //// the another_name variable thus rust doesn't allow us
    //// to move the name if its pointer is being used by 
    //// other scopes.
    println!(">>>>>>>>> pointer {:?}", pointer_to_name);



    ///// ------------------------------------------------------------------------ /////
    
    #[derive(Debug)]
    struct Test{
        a: String,
        b: *const String,
    }

    impl Test{
        fn new(txt: &str) -> Self{
            Test{
                a: String::from(txt),
                b: std::ptr::null(), // b is a const raw pointer to a String
            }
        }

        fn init(&mut self){
            // self.b = &self.a as *const String;
            let self_ref: *const String = &self.a;
            self.b = self_ref;
        }

        fn a(&self) -> &str{
            &self.a
        }

        fn b(&self) -> &String{
            assert!(!self.b.is_null(), "call Test::init first");
            unsafe{&(*self.b)} // expected `&String` in return signature and because of that we dereferenced the `b` with & - `b` has the address of `a`, in order to get its value we have to dereference it which has the value same as `a`
        }
    }
    
    
    let mut test1 = Test::new("test1");
    println!("\n======== BEFORE INIT ========");
    println!("test1 `b` null pointer before init {:p}", test1.b);
    test1.init();
    let mut test2 = Test::new("test2");
    println!("test2 `b` null pointer before init {:p}", test2.b);
    test2.init();
    println!("\n======== BEFORE SWAP ========");
    println!("test1 `a` address {:p}", &test1.a);
    println!("test1 `b` address same as `a` address {:p}", test1.b); // same as `a` address
    println!("test1 `b` address itself {:p}", &test1.b); // different from `a` address cause this is the b address itself
    
    
    println!("test2 `a` address {:p}", &test2.a);
    println!("test2 `b` address same as `a` address {:p}", test2.b); // same as `a` address
    println!("test1 `b` address itself {:p}", &test2.b); // different from `a` address cause this is the b address itself
    
    
    println!("`a` and `b` for test1 - {}, {}", test1.a(), test1.b());
    println!("`a` and `b` for test2 - {}, {}", test2.a(), test2.b());
    
    
    
    println!("\n======== CHANGING THE a VALUE OF TEST1 ========");
    test1.a = "another_test1".to_string(); //  `b` will change cause is a pointer to the location of `a`
    println!("`a` and `b` for test1 - {}, {}", test1.a(), test1.b());
    println!("`a` and `b` for test2 - {}, {}", test2.a(), test2.b());
    
    
    
    std::mem::swap(&mut test1, &mut test2); // move test2 into test1
    println!("\n======== AFTER SWAP ========");
    println!("test1 `a` address remain the same {:p}", &test1.a);
    println!("test1 `b` address same as test2 `b` before swapping  {:p}", test1.b); // same as `a` address
    println!("test2 `a` address remain the same {:p}", &test2.a);
    println!("test2 `b` address same as test1 `b` before swapping {:p}", test2.b); // same as `a` address
    println!("`a` and `b` for test1 - {}, {}", test1.a(), test1.b());
    println!("`a` and `b` for test2 - {}, {}", test2.a(), test2.b());
    


    
    // NOTE - both `b` variables' value will remain the same, only their address are changed
    // test1.a -> 0x7ffd85579fc0 = "test1"    // test1.a -> 0x7ffd85579fc0 = "test2" 
    // test1.b -> 0x7ffd85579fc0 = "test1"    // test1.b -> 0x7ffd8557a058 = "test1"
        
        
    // test2.a -> 0x7ffd8557a058 = "test2"    // test2.a -> 0x7ffd8557a058 = "test1"
    // test2.b -> 0x7ffd8557a058 = "test2"    // test2.b -> 0x7ffd85579fc0 = "test2"
    
    
    

    ///// ------------------------------------------------------------------------ /////
    for i in 0..3{
        // the address of `a` will remain the same in each iteration
        // cause the allocated stack for this app inside 
        // the loop uses the same address or same location for a new variable 
        // that is built in each iteration.
        // //////////////////////////////////////////////////////////////////////////
        // if you want to move the location of a variable to another location  
        // inside the stack put the value of that variable inside another variable
        // by doing this the new defined variable has a new location and new address
        // inside the memory but with a same value as the old variable.
        let mut a: &i32 = &34;
        println!("address of a in memory is same as the old => {:p}", &a);
        a = &242354;
    }
    ///// ------------------------------------------------------------------------ /////


    
    // size of the &str is equals to its bytes and more less than the size of the String 
    // which is 24 bytes usize (8 bytes or 64 bits on 64 bits arch) for each of len, pointer and capacity 
    let name = "wildn🥲oion";
    let string_name = name.to_string();
    let byte_name = name.as_bytes();    
    println!("size name -> {:#?}", size_of_val(name));
    println!("size string name -> {:#?}", size_of_val(&string_name));
    println!("size byte name -> {:#?}", size_of_val(byte_name));
    
    
    
    
    
    ///// ------------------------------------------------------------------------ /////
    // python like inline swapping
    ///// ------------------------------------------------------------------------ /////
    let var_a = 32;
    let var_b = 535;
    let mut a = &var_a; //// a is a pointer with a valid lifetime to the location of var_a type and it contains the address and the data of that type
    let mut b = &var_b; //// b is a pointer with a valid lifetime to the location of var_b type and it contains the address and the data of that type
    ///// inline swapping : a, b = b, a -> a = b; b = a and under the hood : a = &var_b, b = &var_a
    a = &var_b; //// pointer of var_a must points to the location of var_b and after that it can access the data inside var_b 
    b = &var_a; //// pointer of var_b must points to the location of var_a and after that it can access the data inside var_a




    ///// ------------------------------------------------------------------------ /////
    //          encoding an instance into utf8 using unsafe from_raw_parts
    ///// ------------------------------------------------------------------------ /////
    // NOTE - unsafe block for serializing doesn't work like serde due to the need of padding and memory mapping operations which borsh and serde are handling                            
    // NOTE - encoding or serializing process is converting struct object into utf8 bytes
    // NOTE - decoding or deserializing process is converting utf8 bytes into the struct object
    // NOTE - from_raw_parts() forms a slice or &[u8] from the pointer and the length and mutually into_raw_parts() returns the raw pointer to the underlying data, the length of the vector (in elements), and the allocated capacity of the data (in elements)
    // let signed_transaction_serialized_into_bytes: &[u8] = unsafe { //// encoding process of new transaction by building the &[u8] using raw parts of the struct - serializing a new transaction struct into &[u8] bytes
    //     //// converting a const raw pointer of an object and its length into the &[u8], the len argument is the number of elements, not the number of bytes
    //     //// the total size of the generated &[u8] is the number of elements (each one has 1 byte size) * mem::size_of::<Transaction>() and it must be smaller than isize::MAX
    //     //// here number of elements or the len for a struct is the size of the total struct which is mem::size_of::<Transaction>()
    //     slice::from_raw_parts(deserialized_transaction_borsh as *const Transaction as *const u8, mem::size_of::<Transaction>()) //// it'll form a slice from the pointer to the struct and the total size of the struct which is the number of elements inside the constructed &[u8] array; means number of elements in constructing a &[u8] from a struct is the total size of the struct allocated in the memory
    // };
    


}





Ok(())



}
