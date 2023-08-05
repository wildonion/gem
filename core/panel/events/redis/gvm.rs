


use crate::*;

/*
 
    followers weighted tree to understand the relationship between 
    peers to suggests events using graph virtual machine

    
    game and stem like vm engine uisng graph and actor concepts (tokio) 
    which are in cs-concepts repo, also parallel graph and tree walking 
    (shared ref and mutably) using rc and arc weak and strong ref counting, 
    shared ownership and interior mutability, refcell, mutex, tokio spawn, 
    jobq channels, actors, and select event loop


    write more proc macros like vm inside the lib.rs 
    and function like in misc.rs

    
    share ownership between threads using Arc by borrowing the ownership using pointers like & clone 
    share ownership between scopes using Rc by  borrwoing the ownership using pointers like & and clone
    Rc is not safe to be used between threads but Arc can be used to share the type between multiple 
    threads safely without having race conditions, also if we want to mutate an immutable type at runtime
    we must use RefCell which is a single threaded smart pointer and for mutating type between multiple 
    threads we must use Mutex or RwLock to avoid deadlocks situations.
    
    Single Thread    Multithread             Usage
    Rc               --> Arc                 make the type shareable between scopes and threads
    RefCell          --> RwLock || Mutex     make the type mutable safe at runtime in scopes and threads

    https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1305
    https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1213
    https://developerlife.com/2022/02/24/rust-non-binary-tree/#naive-approach-using-weak-and-strong-references
    https://developerlife.com/2022/03/12/rust-redux/
    https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr/src/engine
    https://bevyengine.org/learn/book/introduction/  
    https://godotengine.org/
    https://fyrox-book.github.io/introduction.html

*/

