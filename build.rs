


/* -----------------------------------------------------------------------------
    placing a file named build.rs in the root of a package will cause Cargo to 
    compile that script and execute it just before building the package.
*/
fn main() -> Result<(), Box<dyn std::error::Error>>{

    /* 
        this will build all proto files and convert them into
        rust codes so we can import them in our crates and build
        rpc servers and clients
    */
    tonic_build::compile_protos("proto/kyc.proto").unwrap();

    Ok(())

}