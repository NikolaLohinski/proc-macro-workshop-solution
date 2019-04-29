extern crate proc_macro;
extern crate proc_macro2;

use syn;
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn builder_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let item: syn::ItemStruct = syn::parse(input).expect("failed to parse object to item struct");

  let source_name = &item.ident; 
  let builder_name = syn::Ident::new(&format!("{}Builder", source_name), source_name.span());
  
  let builder_struct = make_builder_struct(&item, &builder_name);
  let struct_trait = make_struct_trait(&item, &builder_name);
  let setters_trait = make_builder_setter(&item, &builder_name);

  let gen = quote! {
    #struct_trait

    #builder_struct
    
    #setters_trait
  };

  gen.into()
}

fn make_builder_struct(source: &syn::ItemStruct, builder_name: &syn::Ident) -> proc_macro2::TokenStream {
  let mut field_tokens = Vec::new();
  for field in &source.fields {
    let name = &field.ident;
    let type_ = &field.ty;
    field_tokens.push(quote!{
        #name: Option<#type_>,
    });
  }
  quote! {
    pub struct #builder_name {
      #(#field_tokens)*
    }
  }
}

fn make_struct_trait(source: &syn::ItemStruct, builder_name: &syn::Ident) -> proc_macro2::TokenStream {
  let mut empty_fields = Vec::new();
  for field in &source.fields {
    let name = &field.ident;
    empty_fields.push(quote!{
        #name: None,
    });
  }
  let source_name = &source.ident;
  quote! {
    impl #source_name {
      pub fn builder() -> #builder_name {
        #builder_name {
          #(#empty_fields)*
        }
      }
    }
  }
}

fn make_builder_setter(source: &syn::ItemStruct, builder_name: &syn::Ident) -> proc_macro2::TokenStream {
  let mut setters = Vec::new();
  for field in &source.fields {
    let name = &field.ident;
    let type_ = &field.ty;
    setters.push(quote!{
        fn #name(&mut self, #name: #type_) {
          self.#name = Some(#name);
        }
    });
  }
  quote! {
    impl #builder_name {
      #(#setters)*
    }
  }
}