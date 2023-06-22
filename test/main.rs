


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




/* ltg: lifetimes + traits + generics */
mod llu;
mod layering;
mod ltg1;
mod ltg2;
mod ltg3;
mod ltg4;
mod ltg5;
mod ltg6;
mod prisoners;






#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{

    

    Ok(())



}
