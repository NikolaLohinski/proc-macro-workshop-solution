extern crate proc_macro;

use proc_macro::TokenStream;
use syn::ItemStruct;
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let item: ItemStruct = syn::parse(input).expect("failed to parse object to item struct");
    let name = &item.ident;
    let gen = quote! {
      impl #name {}
    };
    gen.into()
}