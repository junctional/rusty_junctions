use proc_macro2::{Ident, Span};
use quote::quote;
use rusty_junctions_utils::Module;
use syn::__private::TokenStream2;

pub fn function_types_from_module(module: Module) -> TokenStream2 {
    let module_name = module.ident();
    let number_of_arguments = module.number();

    let message_ident = Ident::new("Message", Span::call_site());
    let messages = std::iter::repeat(message_ident)
        .take(number_of_arguments)
        .collect::<Vec<Ident>>();

    let arguments = quote! { #( crate::types::#messages ,)*  };

    let output = quote! {
        pub mod #module_name {
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
