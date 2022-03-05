#![deny(missing_docs)]

//! Crate providing the essential functionality for the compile time
//! generation of the
//! [`rusty_junctions`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
//! crate to an arbitrary pattern arity.
//!
//! The intention of this crate is to be called from within the root of
//! the
//! [`rusty_junctions`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
//! crate library in order to generate all of the essential components:
//! * The [`PartialPattern`](PartialPattern) and
//!   [`TerminalPartialPattern`](TerminalPartialPattern)
//!   [`patterns`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html)
//!   that leverage the compiler to provide the type safety guarantees
//!   provided to the clients at compile time, preventing a number of
//!   difficult to debug runtime issues.
//!
//! * When applying the `then_do` method to a
//!   [`PartialPattern`](PartialPattern) or a
//!   [`TerminalPartialPattern`](TerminalPartialPattern) we are required
//!   to pass in a closure. The `function_transform` are used to transform
//!   the types of the closure to accept
//!   [`Message`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/types/struct.Message.html)
//!   as the arguments.
//!
//! * The type of the transformed closures also depend on the arity of the
//!   associated
//!   [`pattern`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html),
//!   so we are required to generate this programmatically. This is done
//!   internally using the `function_types` Macro.
//!
//! * Finally the [`JoinPattern`](JoinPattern) derive macro which is used
//!   to derive the implementation of the `JoinPattern` trait from the
//!   [`PartialPattern`](PartialPattern) or
//!   [`TerminalPartialPattern`](TerminalPartialPattern).
//!
//! See the function level documentation for further details.

mod function_transform;
mod function_types;
mod join_pattern_derive;
mod library_generate;
mod module;
mod partial_pattern_derive;
mod pattern_generation;

use module::Module;
use proc_macro::{self, TokenStream};
use syn::{parse_macro_input, DeriveInput};

use crate::{
    function_transform::function_transform_from_module, function_types::function_types_from_module,
    join_pattern_derive::join_pattern_from_derive, library_generate::library_generate_from_module,
    partial_pattern_derive::partial_pattern_from_derive, pattern_generation::pattern_from_module,
};

/// Generate the
/// [`rusty_junctions`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/index.html)
/// crate to an arbitrary pattern arity.
///
/// # Arguments
/// The input [`TokenStream`](TokenStream) is a integer value representing
/// the required maximum pattern arity.
#[proc_macro]
pub fn library_generate(input: TokenStream) -> TokenStream {
    let module: Module = parse_macro_input!(input);
    let output = library_generate_from_module(module);
    output.into()
}

/// Derive the implementation of the `JoinPattern` trait from the
/// [`PartialPattern`](PartialPattern) or
/// [`TerminalPartialPattern`](TerminalPartialPattern).
#[proc_macro_derive(JoinPattern)]
pub fn join_pattern(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let output = join_pattern_from_derive(input);
    output.into()
}

/// Derive the implementation of the a Terminal
/// [`pattern`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html)
/// from a given `struct`.
///
/// A [`TerminalPartialPattern`](TerminalPartialPattern) is the final
/// [`pattern`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html)
/// that can be created, it has the highest possible arity.  Therefore, it
/// is not possible to extend it with the combinators like a
/// [PartialPattern](PartialPattern) can be.
#[proc_macro_derive(TerminalPartialPattern)]
pub fn terminal_partial_pattern(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let output = partial_pattern_from_derive(input, true);
    output.into()
}

/// Derive the implementation of the a Partial
/// [`pattern`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html)
/// from a given `struct`.
///
/// A [`PartialPattern`](PartialPattern) is able to be extended using the
/// standard Join Calculus combinators (`and`, `and_bidir`, and
/// `and_recv`), creating larger and larger arity
/// [`patterns`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html).
#[proc_macro_derive(PartialPattern)]
pub fn partial_pattern(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let output = partial_pattern_from_derive(input, false);
    output.into()
}
