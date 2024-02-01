use pm::Span;
use proc_macro as pm;

use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemStruct, Expr, Ident, ItemFn, spanned::Spanned};

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
    let attr_expr = syn::parse::<Expr>(attr.clone());

    let default = syn::parse::<Expr>(quote! { Default::default }.into()).unwrap();
    let expr = match attr_expr {
        Ok(tree) => tree,
        Err(_) if attr.is_empty() => default,
        Err(e) => return e.to_compile_error().into(),
    };

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


#[proc_macro_attribute]
///Generate a ctor static with this function.
///```rust,ignore
///#[singleton_fn] //using MAKE_THING as name
///#[singleton_fn(MY_STATIC)] //using MY_STATIC as name 
///fn make_thing() -> Thing;
///```
pub fn singleton_fn(attr: pm::TokenStream, item: pm::TokenStream) -> pm::TokenStream {
    let data = parse_macro_input!(item as ItemFn);
    let attr_ident = syn::parse::<Ident>(attr).ok();

    let item_name = &data.sig.ident;
    let struct_name = match &data.sig.output {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => quote! { #ty },
    };

    let static_name = match attr_ident {
        Some(ident) => ident,
        None => syn::Ident::new(&item_name.to_string().to_uppercase(), item_name.span()),
    };
    let fn_name = syn::Ident::new(
        &format!("_{}_global_init", static_name.to_string().to_lowercase()), 
        Span::call_site().into());

    quote!{ 
        pub static #static_name: global_static::Global<#struct_name> = global_static::Global::new(#item_name);
        #[global_static::ctor::ctor]
        fn #fn_name() {
            #static_name.init()
        }
        #data
    }.into()
}
