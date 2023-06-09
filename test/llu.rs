
use crate::*;



pub static mut ARR: [u8; 3] = [0 as u8; 3]; //// filling the array with zeros 3 times
pub static SEEDS: &[&[u8]; 2] = &["wildonion".as_bytes(), &[233]];



// we can access all the following in exports module using exports::*
pub mod exports{
    pub struct Test;
    pub async fn run(){}
}



//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////

pub async fn unsafer(){

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
    also every type has a lifetime inside the stack including the heap data pointers and tha't why we can't return a pointer to 
    the heap data which are owned by the function since once the function gets executed its data will be dropped from the ram
    and the pointer that we've just returned it will be a dangling pointer.
			    
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
    /*

        when a type is going to dropped at the end of a block or scope 
        there it can't be a borrow of that type or basically if the 
        type is behind a pointer there it can'e be moved
    
    */
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
    println!(">>>>>>>>> pointer {:?}", another_name);
    println!(">>>>>>>>> pointer {:?}", name);


    /////////////////////////////////////////////
    // if we pass a mutable pointer to the type 
    // to the method calls then by mutating the 
    // pointer the value of that type outside the 
    // method will be mutated too.

    /*  if we pass by reference means that we're borrowing the type 
        (not moving it) and if we pass a mutable reference means that
        if we mutate that reference inside other scopes like methods 
        the actual type which is the owner of the reference will be 
        mutated too in its initialized scope.
    */
    /////////////////////////////////////////////

    fn s5(mut name: &mut String){
        *name = "main".to_string();
        
    }

    fn int(mut int: &mut u32){
        *int = 23;
    }

    let mut man = "wildonion".to_string();
    let mut num = 21;
    
    let mut mut_pointer_to_man = &mut man;
    // let another_man = man; // can't move man here since it's borrowed and is behind a pointer
    s5(mut_pointer_to_man);
    println!("{}", man);
    
    int(&mut num);    
    println!("{}", num);

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
    
    let mut a = String::from("wildonion");
    let mut b = &mut a;
    *b = String::from("changed"); // now a has changed too 
    
    println!("b {}", b);
    println!("a {}", a);
    
    
    
    
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