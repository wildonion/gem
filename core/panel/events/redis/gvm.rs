



// followers weighted tree to understand the relationship between 
// peers to suggests events using graph virtual machine

// parallel graph and tree walking (shared ref and mutably) using 
// rc and arc weak and strong ref counting, shared ownership and 
// interior mutability, refcell, mutex, tokio spawn, jobq channels, 
// actors, and select event loop

// Single Thread    Multithread
// Rc               --> Arc
// RefCell          --> RwLock || Mutex

// https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1305
// https://github.com/wildonion/uniXerr/blob/a30a9f02b02ec7980e03eb8e31049890930d9238/infra/valhalla/coiniXerr/src/schemas.rs#L1213
// https://developerlife.com/2022/02/24/rust-non-binary-tree/#naive-approach-using-weak-and-strong-references
// https://developerlife.com/2022/03/12/rust-redux/
// https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr/src/engine