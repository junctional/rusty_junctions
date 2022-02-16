mod function_transform;
mod function_types;
mod join_pattern_derive;
mod library_generate;
mod partial_pattern_derive;
mod pattern_generation;
mod module;

use module::Module;
use proc_macro::{self, TokenStream};
use syn::{parse_macro_input, DeriveInput};

use crate::{
    function_transform::function_transform_from_module, function_types::function_types_from_module,
    join_pattern_derive::join_pattern_from_derive, library_generate::library_generate_from_module,
    partial_pattern_derive::partial_pattern_from_derive, pattern_generation::pattern_from_module,
};

#[proc_macro]
pub fn function_transform(input: TokenStream) -> TokenStream {
    let module: Module = parse_macro_input!(input);
    let output = function_transform_from_module(module);
    output.into()
}

#[proc_macro]
pub fn function_types(input: TokenStream) -> TokenStream {
    // let module: FunctionTypesInput = parse_macro_input!(input);
    let module: Module = parse_macro_input!(input);
    let output = function_types_from_module(module);
    output.into()
}

#[proc_macro_derive(JoinPattern)]
pub fn join_pattern(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let output = join_pattern_from_derive(input);
    output.into()
}

#[proc_macro_derive(TerminalPartialPattern)]
pub fn terminal_partial_pattern(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let output = partial_pattern_from_derive(input, true);
    output.into()
}

#[proc_macro_derive(PartialPattern)]
pub fn partial_pattern(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let output = partial_pattern_from_derive(input, false);
    output.into()
}

#[proc_macro]
pub fn library_generate(input: TokenStream) -> TokenStream {
    let module: Module = parse_macro_input!(input);
    let output = library_generate_from_module(module);
    output.into()
}
