use pm::Span;
use proc_macro as pm;

use quote::quote;
use syn::{parse_macro_input, ItemStruct, Expr};

#[proc_macro_attribute]
///Generate a ctor static of this struct.
///By defeault, uses `Default` if the type implements it. You can pass an expression to the
///attribute to use it instead.
///```rust,ignore
///#[singleton] //using Default::default
///#[singleton(MyType::parse)] //using MyType::parse
///#[singleton(|| MyType::new())] //closures work too
pub fn singleton(attr: pm::TokenStream, item: pm::TokenStream) -> pm::TokenStream {
    let data = parse_macro_input!(item as ItemStruct);
    let attr_expr = syn::parse::<Expr>(attr);

    let default = syn::parse::<Expr>(quote! { Default::default }.into()).unwrap();
    let expr = attr_expr.unwrap_or(default);

    let struct_name = &data.ident;
    let static_name = syn::Ident::new(&struct_name.to_string().to_uppercase(), struct_name.span());
    let fn_name = syn::Ident::new(
        &format!("_{}_global_init", struct_name.to_string().to_lowercase()), 
        Span::call_site().into());
    

    let out = quote! {
        pub static #static_name: global_static::Global<#struct_name> = global_static::Global::new(#expr);
        #[global_static::ctor::ctor]
        fn #fn_name() {
            #static_name.init()
        }
        #data
    };

    

    out.into() 
}
