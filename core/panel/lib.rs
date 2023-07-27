


/*

    ------------- IMPORTANT NOTE ON MACROS -------------
    since macro processing in Rust happens after the construction of 
    the AST, as such, the syntax used to invoke a macro must be a proper 
    part of the language's syntax tree thus by adding a new code in this 
    crate the compiler needs to compile the whole things again, which 
    forces us to reload the workspace everytime.

    a TokenStream is simply the Rust codes which can be used to built the AST 
    like: RUST CODES ---> TOKEN STREAM ---> AST also the following are matters:
    sync generates : the AST from the passed in TokenStream (sequence of token trees)
    quote generates: Rust codes that can be used to generate TokenStream (sequence of token trees) and a new AST


    kinds: 
        decl_macro
        proc_macro
        proc_macro_derive
        proc_macro_attribute
    
    benefits:
        add a method to struct or check a condition against its fields
        convert trait into module to extend the trait methods
        extend the interface of a struct by changing the behaviour of its fields and methods
        create a DSL like jsx or a new keyword or a new lang
        build a new AST from the input TokenStream and return the generated TokenStream from a new Rust codes


*/



use proc_macro::TokenStream;
use syn::{parse, parse_macro_input, Attribute};
use quote::{format_ident, quote}; // alows us to generate and write rust codes



/*
    with the following proc macro we can do inspect and operate on the 
    api methods before generating the output or executing any extra
    logics before getting into the api body like actix #[get()] which
    checks the request path in the first place before sliding into 
    the request body, 
*/
#[proc_macro_attribute]
pub fn passport(args: TokenStream, input: TokenStream) -> TokenStream {

    /*
        1. we have to build the new AST from the `input` TokenStream to extend 
            the one that we already have by using syn to parse the args & input 
            tokens into a syntax tree, note that the type of TokenStream that 
            we want to parse it with syn to generate AST, must be specified, 
            like parsing a function TokenStream into ItemFn AST.

        2. generate a new TokenStream or tokens from step 1 using quote to generate 
            Rust codes which will generate a new TokenStream that can be used to build 
            a new AST, this will replace whatever `input` is annotated with this attribute 
            proc macro.
        
        3. return the token stream either generated by the quote or the passed in input.
    */

    let api_ast = syn::parse::<syn::ItemFn>(input.clone()).unwrap();
    let params = api_ast.attrs;

    /* return the generated token stream using quote */
    TokenStream::from(quote!(fn dummy(){}))
    // input 

}


#[proc_macro]
pub fn my_fn_like_proc_macro(input: TokenStream) -> TokenStream {

  input

}

#[proc_macro_derive(PassportDrive)]
pub fn my_derive_proc_macro(input: TokenStream) -> TokenStream {

  input

}