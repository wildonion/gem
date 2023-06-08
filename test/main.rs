


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
use log::{error, info};





mod llu;
mod layering;
mod traits;



pub static mut ARR: [u8; 3] = [0 as u8; 3]; //// filling the array with zeros 3 times
pub static SEEDS: &[&[u8]; 2] = &["wildonion".as_bytes(), &[233]];



// we can access all the following in exports module using exports::*
pub mod exports{
    pub struct Test;
    pub async fn run(){}
}



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
        
        
    `let` will store on the stack which may get new address later (we can pin it to the ram to avoid of changing its address), 
    `static` and `const` will store on the data segment which will allocate nothing and have fixed address on the stack during execution 
    also every type has a lifetime inside the stack including the heap data pointers
			    
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
	




    Ok(())



}
