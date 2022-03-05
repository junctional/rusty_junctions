#![deny(missing_docs)]

//! Crate providing a number of [Procedural
//! Macros](https://doc.rust-lang.org/reference/procedural-macros.html)
//! that can be used as part of the Public API of [Rusty
//! Junctions](https://crates.io/crates/rusty_junctions), to define the
//! various
//! [Junction](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html),
//! [Channels](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/channels/index.html),
//! and [Join
//! Patterns](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html)
//! that are used to declaratively define the desired concurrent computation.

use crate::junction::Junction;
use proc_macro::{self, TokenStream};
use syn::{__private::TokenStream2, parse_macro_input};

mod junction;

/// Define an entire
/// [`Junction`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
/// using a single block.
///
/// A
/// [`Junction`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
/// is the smallest unit of distribution for the Join Calculus.  This
/// syntax allows for an entire junction to be defined as a single block.
///
/// # Safety
/// This macro provides a number of essential checks on the given input to
/// the [junction](junction!) macro and provides detailed error messages
/// to inform the user of the correct usage.  Below are some of the checks
/// that have been implemented:
/// * Invalid Channel Mode
/// * Join Pattern containing an undefined Channel
/// * Join Pattern with no input Channels
/// * Join Pattern containing more than one bidirectional or receiving
///   channels
///
/// # Example
/// ## Standard API
/// Suppose we use the standard Public API we can define our computation using the following syntax.
/// ```rust
/// let junction = rusty_junctions::Junction::new();
/// let name = junction.send_channel::<String>();
/// let value = junction.send_channel::<i32>();
/// junction.when(&name).and(&value).then_do(|name, value| {
///     println!("Standard API: {name} {value}");
/// });
/// value.send(0).unwrap();
/// name.send(String::from("Hello, World!")).unwrap();
/// ```
///
/// ## Single Junction Procedural Macro API
/// However, it is also possible to use this alternative (simpler?) syntax for defining the entire
/// [`Junction`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html).
/// ```rust
/// junction! {
///     name as Send::String,
///     value as Send::i32,
///     |name, value| {
///         println!("Single Junction Procedural Macro API: {name} {value}");
///     },
/// };
/// value.send(2).unwrap();
/// name.send(String::from("Hello, World!")).unwrap();
/// ```
///
/// # Advantages
/// * The main advantage is that we have a convenient and less verbose
///   syntax for defining the
///   [`Junction`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html).
///
/// * This syntax allows for the entire junction to be defined in a single
///   block. Defining all of the intermediate values in a nested scope,
///   preventing them from being accessed from elsewhere in the client.
///
/// * This procedural version of the [`junction`](junction!) macro
///   constructs the
///   [Junction](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
///   in the given scope.
///
/// * This macro is self contained and requires no other imports to be
///   made, unlike its declarative counterpoint.
///
/// * As you can see in the [Example](#Example) above the Controller
///   explicitly stopped, if we allowed it to be dropped from the inner
///   scope there would be no guarantee it would have time for the pattern
///   to fire.
///
#[proc_macro]
pub fn junction(input: TokenStream) -> TokenStream {
    let junction: Junction = parse_macro_input!(input);
    let output: TokenStream2 = junction.into();
    output.into()
}
