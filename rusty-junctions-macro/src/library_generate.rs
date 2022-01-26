use crate::{function_transform_from_module, function_types_from_module, pattern_from_module};
use quote::quote;
use rusty_junctions_utils::Module;
use syn::__private::TokenStream2;

pub fn library_generate_from_module(module: Module) -> TokenStream2 {
    let last_order = module.number();

    // Create the patterns
    let other_patterns = (1..last_order)
        .into_iter()
        .map(|n| pattern_from_module(Module::from_usize(n), false))
        .collect::<TokenStream2>();
    let final_pattern = pattern_from_module(Module::from_usize(last_order), true);

    // Create the function transforms
    let function_transforms = (1..=last_order)
        .into_iter()
        .map(|n| function_transform_from_module(Module::from_usize(n)))
        .collect::<TokenStream2>();

    // Create the function types
    let function_types = (1..=last_order)
        .into_iter()
        .map(|n| function_types_from_module(Module::from_usize(n)))
        .collect::<TokenStream2>();

    let output = quote! {
        mod function_transforms {
            #function_transforms
        }

        mod patterns {
            #other_patterns
            #final_pattern
        }

        pub mod functions {
            #function_types
        }
    };

    output.into()
}
