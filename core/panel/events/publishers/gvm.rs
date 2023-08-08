


use crate::*;

/*
 
    followers weighted tree to understand the relationship between 
    peers to suggests events using graph virtual machine
    
    
    an state management app using actors and graph concepts, traits, 
    macros (ast, token stream) and pointers(Arc, Rc, Mutex, RefCell)

    
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
    https://bevyengine.org/learn/book/introduction/  
    https://godotengine.org/
    https://fyrox-book.github.io/introduction.html
    https://www.youtube.com/watch?v=yq-msJOQ4nU
    https://github.com/wildonion/cs-concepts
    https://doc.rust-lang.org/nomicon/index.html
    https://stackoverflow.com/questions/26271151/precise-memory-layout-control-in-rust
    https://docs.rust-embedded.org/book/
    https://crates.io/crates/hotham
    https://developers.google.com/protocol-buffers/docs/encoding
    https://capnproto.org/encoding.html
    https://ethereum.org/nl/developers/docs/evm/
    https://blog.subnetzero.io/post/building-language-vm-part-01/
    https://rust-hosted-langs.github.io/book/
    https://benkonz.github.io/building-a-brainfuck-compiler-with-rust-and-llvm/
    https://opensource.com/article/19/3/rust-virtual-machine
    https://medium.com/iridium-vm/so-you-want-to-build-a-language-vm-in-rust-part-09-15d90084002
    https://medium.com/clevyio/using-rust-and-nom-to-create-an-open-source-programming-language-for-chatbots-12fe67582af5
    https://cheats.rs/#behind-the-scenes

*/

