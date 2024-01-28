use pm::Span;
use proc_macro as pm;

use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn singleton(_attr: pm::TokenStream, item: pm::TokenStream) -> pm::TokenStream {
    let data = parse_macro_input!(item as ItemStruct);

    let struct_name = &data.ident;
    let static_name = syn::Ident::new(&struct_name.to_string().to_uppercase(), struct_name.span());
    let fn_name = syn::Ident::new(
        &format!("_{}_global_init", struct_name.to_string().to_lowercase()), 
        Span::call_site().into());

    let out = quote! {
        pub static #static_name: global_static::Global<#struct_name> = global_static::Global::new(Default::default);
        #[global_static::ctor::ctor]
        fn #fn_name() {
            #static_name.init()
        }
        #data
    };

    out.into() 
}
