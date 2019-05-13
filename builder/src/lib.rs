extern crate proc_macro;
extern crate proc_macro2;

use syn;
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn builder_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let item: syn::ItemStruct = syn::parse(input).expect("failed to parse object to item struct");

  let source_name = &item.ident; 
  let builder_name = syn::Ident::new(&format!("{}Builder", source_name), source_name.span());
  
  let struct_impl = make_struct_impl(&item, &builder_name);
  let builder_struct = make_builder_struct(&item, &builder_name);
  let builder_impl = make_builder_impl(&item, &builder_name);

  let gen = quote! {
    use std::error::Error;

    #struct_impl

    #builder_struct
    
    #builder_impl
  };

  gen.into()
}

fn make_struct_impl(source: &syn::ItemStruct, builder_name: &syn::Ident) -> proc_macro2::TokenStream {
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
    #[derive(Clone)]
    pub struct #builder_name {
      #(#field_tokens)*
    }
  }
}

fn make_builder_impl(source: &syn::ItemStruct, builder_name: &syn::Ident) -> proc_macro2::TokenStream {
  let mut setters = Vec::new();
  for field in &source.fields {
    let name = &field.ident;
    let type_ = &field.ty;
    setters.push(quote!{
        fn #name(mut self, #name: #type_) -> Self {
          self.#name = Some(#name);
          self
        }
    });
  }

  let build_method = make_builder_build_method(source, builder_name);

  quote! {
    impl #builder_name {
      #(#setters)*
      
      #build_method
    }
  }
}

fn make_builder_build_method(source: &syn::ItemStruct, builder_name: &syn::Ident) -> proc_macro2::TokenStream {
  let mut guard = Vec::new();
  let mut output_values = Vec::new();
  for field in &source.fields {
    let name = &field.ident;
    guard.push(quote!{
        #name: Some(#name),
    });
    output_values.push(quote!{
      #name: #name,
    });
  }
  let source_name = &source.ident;
  quote! {
    pub fn build(mut self) -> Result<#source_name, Box<dyn Error>> {
      match self {
        #builder_name { #(#guard)* } => Ok(#source_name { #(#output_values)* }),
        _ => Err(From::from(format!("missing some fields to build {}", stringify!(#source_name)))),
      }
    }
  }
}