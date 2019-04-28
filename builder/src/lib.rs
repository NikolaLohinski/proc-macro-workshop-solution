extern crate proc_macro;

use proc_macro::TokenStream;
use syn;
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let item: syn::ItemStruct = syn::parse(input).expect("failed to parse object to item struct");

    let struct_name = &item.ident;
    let builder_struct_name = syn::Ident::new(&format!("{}Builder", struct_name), struct_name.span());

    let mut builder_struct_fields = Vec::new();
    let mut builder_struct_empty = Vec::new();
    for field in &item.fields {
      let ident = &field.ident;
      let ty = &field.ty;
      builder_struct_fields.push(quote!{
          #ident: Option<#ty>,
      });
      builder_struct_empty.push(quote!{
          #ident: None,
      });
    }
 
    let gen = quote! {
      impl #struct_name {
        pub fn builder() -> #builder_struct_name {
          #builder_struct_name {
            #(#builder_struct_empty)*
          }
        }
      }
      pub struct #builder_struct_name {
        #(#builder_struct_fields)*
      }
    };
 
    gen.into()
}