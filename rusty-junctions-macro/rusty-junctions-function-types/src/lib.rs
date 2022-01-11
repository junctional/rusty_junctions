use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitInt, Token,
};

struct FunctionTypesInput {
    module_name: Ident,
    number_of_arguments: usize,
}

impl Parse for FunctionTypesInput {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if input.is_empty() {
            panic!("Invalid input to macro function_types");
        }

        let module_name: Ident = input.parse()?;
        let _semi_colon_token: Token![;] = input.parse()?;
        let number_of_arguments: usize = input.parse::<LitInt>()?.base10_parse::<usize>()?;

        Ok(FunctionTypesInput {
            module_name,
            number_of_arguments,
        })
    }
}

#[proc_macro]
pub fn function_types(input: TokenStream) -> TokenStream {
    let FunctionTypesInput {
        module_name,
        number_of_arguments,
    } = parse_macro_input!(input);

    let message_ident = Ident::new("Message", Span::call_site());
    let messages = std::iter::repeat(message_ident)
        .take(number_of_arguments)
        .collect::<Vec<Ident>>();

    let arguments = quote! { #( #messages ,)*  };

    let output = quote! {
        pub mod #module_name {
            use std::{
                any::Any,
                sync::mpsc::Sender,
                thread::{JoinHandle, Thread},
            };
            use crate::types::Message;

            /// Trait to allow boxed up functions that take three `Message`s and return
            /// nothing to be cloned.
            pub trait FnBoxClone: Fn( #arguments ) -> () + Send {
                fn clone_box(&self) -> Box<dyn FnBoxClone>;
            }

            impl<F> FnBoxClone for F
            where
                F: Fn( #arguments ) -> () + Send + Clone + 'static,
            {
                /// Proxy function to be able to implement the `Clone` trait on
                /// boxed up functions that take three `Message`s and return nothing.
                fn clone_box(&self) -> Box<dyn FnBoxClone> {
                    Box::new(self.clone())
                }
            }

            impl Clone for Box<dyn FnBoxClone> {
                fn clone(&self) -> Box<dyn FnBoxClone> {
                    (**self).clone_box()
                }
            }

            /// Type alias for boxed up cloneable functions that take three `Message`s and
            /// return nothing. Mainly meant to increase readability of code.
            pub type FnBox = Box<dyn FnBoxClone>;
        }
    };

    output.into()
}
