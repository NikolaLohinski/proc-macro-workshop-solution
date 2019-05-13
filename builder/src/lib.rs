extern crate proc_macro;
extern crate proc_macro2;

use syn;
use quote::quote;
use std::error::Error;

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
    let type_ = match extract_option_type(&field.ty) {
      Ok(t) => t,
      Err(_) => field.ty.clone(),
    };
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

fn make_builder_impl(source: &syn::ItemStruct, builder_name: &syn::Ident) -> proc_macro2::TokenStream {
  let mut setters = Vec::new();
  for field in &source.fields {
    let name = &field.ident;
    let type_ = match extract_option_type(&field.ty) {
      Ok(t) => t,
      Err(_) => field.ty.clone(),
    };
    setters.push(quote!{
        fn #name(mut self, #name: #type_) -> Self {
          self.#name = Some(#name);
          self
        }
    });
  }

  let build_method = make_builder_build_method(source);

  quote! {
    impl #builder_name {
      #(#setters)*
      #build_method
    }
  }
}

fn make_builder_build_method(source: &syn::ItemStruct) -> proc_macro2::TokenStream {
  let mut guards = Vec::new();
  let mut values = Vec::new();

  let source_name = &source.ident;

  for field in &source.fields {
    let name = &field.ident;
    values.push(quote!{ #name: #name, });

    let guard = match extract_option_type(&field.ty) {
      Ok(_) => quote!{ let #name = self.#name; },
      Err(_) => quote!{
        let #name = match self.#name {
          Some(value) => value,
          None => return Err(From::from(format!("missing {} to build {}", stringify!(#name), stringify!(#source_name)))),
        };
      },
    };
    guards.push(guard);
  }
  quote! {
    fn build(mut self) -> Result<#source_name, Box<dyn Error>> {
      #(#guards)*
      Ok(#source_name { #(#values)* })
    }
  }
}

fn extract_option_type(type_: &syn::Type) -> Result<syn::Type, Box<dyn Error>> {
  match type_ {
    syn::Type::Path(typepath) if typepath.qself.is_none() && path_is_option(&typepath.path) => {
      let type_params = match &typepath.path.segments.iter().next() {
        Some(segment) => &segment.arguments,
        None => return Err(From::from("failed to get first segment arguments")),
      };
      let parameters = match type_params {
          syn::PathArguments::AngleBracketed(params) => params,
          _ => return Err(From::from("no angle brackets")),
      };
      let generic_arg = match parameters.args.iter().next() {
        Some(arg) => arg,
        None => return Err(From::from("failed to get generic argument")),
      };
      match generic_arg {
          syn::GenericArgument::Type(ty) => Ok(ty.clone()),
          _ => return Err(From::from("impossible to extract type")),
      }
    }
    _ => Err(From::from("not an option"))
  }
}

fn path_is_option(path: &syn::Path) -> bool {
  if path.leading_colon.is_none() && path.segments.len() == 1 {
    return match path.segments.iter().next() {
      Some(segment) => &segment.ident == "Option",
      None => false,
    };
  }
  return false
}