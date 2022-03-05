use crate::Module;
use crate::{function_transform_from_module, function_types_from_module, pattern_from_module};
use quote::quote;
use syn::__private::TokenStream2;

pub fn library_generate_from_module(module: Module) -> TokenStream2 {
    let arity = module.number();

    // Create the partial patterns
    let partial_patterns = (1..arity)
        .into_iter()
        .map(|n| pattern_from_module(Module::from_usize(n), false))
        .collect::<TokenStream2>();

    // Create the terminal patterns for the highest arity
    let terminal_partial_pattern = pattern_from_module(Module::from_usize(arity), true);

    // Create the function transforms
    let function_transforms = (1..=arity)
        .into_iter()
        .map(|n| function_transform_from_module(Module::from_usize(n)))
        .collect::<TokenStream2>();

    // Create the function types
    let function_types = (1..=arity)
        .into_iter()
        .map(|n| function_types_from_module(Module::from_usize(n)))
        .collect::<TokenStream2>();

    let output = quote! {
        mod function_transforms {
            #function_transforms
        }

        mod patterns {
            #partial_patterns
            #terminal_partial_pattern
        }

        pub mod functions {
            #function_types
        }
    };

    output.into()
}
