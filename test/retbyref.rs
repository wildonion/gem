



use crate::*;

// lifetimes can be passed to method and struct signature to be bounded to generic types for borrowing them or slices

fn test(){

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

            #[derive(Default)]
            struct FuckMe{
                a: u8,
                b: String
            }

            let instanceFuckMe = FuckMe{
                a: 23,
                ..Default::default() //// fillint the remaining field with default values
            };        

            let FuckMe{a: first_input, ..} = instanceFuckMe;

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

            // we don't have string indices instead we have to access it using a range like [0..2] which gives us the first byte of the string
            // because string[1] means that returning the first char of the string that is 1 byte but if we have a utf16 string the first char 
            // is 2 bytes thus string[1] can't return the first char since it thinks that the every char of string is 1 byte hence rust doesn't
            // allow us to this in the first place because String will be coerced into slices or &str in compile time which we don't know where 
            // it will be placed which is either in heap, binary or stack thus the size it's unknown and because of this rust compiler can't know
            // the exact size of string and it's type in first place  


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

        // - it's ok to return pointer to a struct which has no fields cause it can allocate nothing on the stack and there is no data to be owned by the scope
        fn run_taker(taker: &mut Commander) -> &Commander{
            let instance = &Commander{}; //// instance allocate nothing on the stack since Commander has no fields
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

        
        /*
            
            we can return a reference from a method to a type that allocates
            nothing on the stack like returning Pack{} directly without storing 
            it inside a variable but we can't return a pointer to a type that is 
            owned by the that method since that type is a local variable which 
            has defined inside the method, allocated space on the stack and once 
            the method gets executed its lifetime will be dropped from the ram thus 
            its pointer will be remained a dangling pointer which rust doesn't allow 
            us to return the pointer to the local type in the first place in other 
            words a type can't be moved if it's behind a pointer.

            if we want to return a double pointer the first one can be 
            allocate nothing but in order to pointing to the first one 
            the first one must be allocate space on the ram thus the in 
            following we can't return ref_ since ref_ is a double pointer
            to the self in which self is a reference to the instance in 
            the first param of the method thus returning ref_ from the 
            function is not possible because &Pack allocate nothing on 
            the stack which is self but &&Pack must points to an allocated 
            space on the stack or pointing to &Pack or self on the stack
            or heap thus the self must be stored some where on the stack or 
            heap then do this but rust doesn't allow us to return a pointer 
            to a type which is owned by the function and is on the heap 
            since once the function gets executed the type will be dropped 
            and pointer will be converted into a dangling pointer.  
        
            Pack is zero sized type (ZST) and will be stored on the stack or 
            heap depends on how is being used at runtime
            
            we cannot return a &&Pack directly because the inner reference 
            (&Pack) is bound to the local scope of the function once the function returns, 
            the inner reference would be invalid, and Rust's borrow checker prevents us 
            from doing this to ensure memory safety.
        
        */
        //// in the following the return type is a double pointer
        //// which the first one must gets allocated on the satck 
        //// first in order to return a pointer to that and since
        //// we've allocated something on the ram (stack or heap) 
        //// which is owned by current function thus we can't return
        //// a pointer to the type which that type is owned by the 
        //// function.
        // fn as_ref__(&self) -> &&Pack{ 
        //     let ref ref_ = self; 
        //     // &self //// can't return &self since self is owned by the function also because self is borrowed
        //     ref_ //// can't return ref_ since self is owned by the function also because self is borrowed
        // }

        // ---------------- THUS             
        // - can't move out of a type if it's behind a pointer but we can pass by ref
        // - can't return pointer to a heap data which is owned by the function we can use Box instead but we're okey to return slice with a valid lifetime
        // ----------------
        //// can't move the type if it's behind a pointer and doesn't implement copy trait (heap data) 
        //// since we can borrow mutably and return ref to stack types from function but not heap data 
        //// thus any reference to the instance or the struct itself which contains a heap data field 
        //// is not possible because heap data types are not Copy and once the scope that contains them
        //// gets executed they will be dropped from the ram and any pointer to them will be converted 
        //// into the a dangling pointer which rust doesn't allow us to do this in the first place. 
        fn as_ref(&self) -> &Pack{
            let ref_ = self;
            //// here we're returning the self or ref_ which is an immutable pointer of Pack instance 
            // ref_
            self 
        }

        fn as_ref_(&self) -> &Pack{
            let ref ref_ = self; 
            let ref__ = &self; 
            ref__
        }

        // pub fn ref_to_str() -> HashMap<&str, &str>{ //// we can't return &str since we need a specific lifetime to do so
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
		
        /*
        
            if we try to return a reference to a local String or Vec or heap data 
            variables within a function, we will run into lifetime issues, since 
            the local variable is dropped when the function goes out of scope.
            thus we can return the them in their slice form like &str for String
            &[u8] for Vec with a specific lifetime or a lifetime which lives long
            enough if the function gets executed like 'static lifetime, note that 
            we can't return them behind a valid reference at all since they're owned
            by the function scope and no matter how they cant be used! 
        
        */
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

        pub fn closure_are_traits() -> impl FnOnce(String) -> String{ //// returning a closure from the function since closures are traits we can use -> impl ... syntax to implement the FnOnce for the return type 
            |name: String|{
                name
            }
        }

        pub fn run() -> impl std::future::Future<Output=u8>{ //// implementing the Future trait for the return type of the function by doing this we have to return an async block from the function
            async move{ //// returning an async block from the function
                26
            }
            // let res = run.await;
            // let res = Pack::run().await;
        }

        //// the following is another way of defining async method 
        // pub async fn run() -> u8{
        //     26
        //     // let res = run.await;
        //     // let res = Pack::run().await;
        // }

        pub async fn _run() -> u8{ //// above implementation is equivalent to this one 
            26

            // let res = run.await;
        }

        pub async fn unpack_self(&self) -> (){
            let Pack{..} = self; //// unpacking self into the struct itself, there is no explicit field naming and we filled all the fields using `..`
        }

	}

}