#![deny(missing_docs)]

//! Crate providing a number of [Declarative
//! Macros](https://doc.rust-lang.org/reference/macros-by-example.html)
//! that can be used as part of the Public API of [Rusty
//! Junctions](https://crates.io/crates/rusty_junctions), to define the
//! various
//! [Junction](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html),
//! [Channels](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/channels/index.html),
//! and [Join
//! Patterns](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html)
//! that are used to declaratively define the desired concurrent computation.

/// An alternate syntax for defining Join Pattern channel constrains.
///
/// The [`when`](when) macro allows for a number of channels to be defined
/// using a list like syntax, opposed to the standard method chaining
/// provided by the default Public API of [Rusty
/// Junctions](https://crates.io/crates/rusty_junctions).
///
/// # Example
/// ## Standard API
/// By using the standard Public API we can define our computation using the following syntax.
/// ```rust
///
/// let junction = rusty_junctions::Junction::new();
/// let chan1 = junction.send_channel::<i32>();
/// let chan2 = junction.send_channel::<i32>();
/// let chan3 = junction.send_channel::<i32>();
///
/// junction.when(&chan1)
///     .and(&chan2)
///     .and(&chan3)
///     .then_do(|v1, v2, v3| println!("{v1}, {v2}, {v3}"))
/// ```
///
/// ## `when` Macro API
/// However, it is also possible to use this richer more expressive syntax, when creating the firing conditions for a Join Pattern.
/// ```rust
/// let junction = rusty_junctions::Junction::new();
/// let chan1 = junction.send_channel::<i32>();
/// let chan2 = junction.send_channel::<i32>();
/// let chan3 = junction.send_channel::<i32>();
///
/// when!(junction; chan1, chan2, chan3)
///     .then_do(|v1, v2, v3| println!("{v1}, {v2}, {v3}"))
/// ```
#[macro_export]
macro_rules! when {
    ( $junction:ident; $initial_channel:ident, $( $other_channels:ident ),* ) => {
        $junction.when(&$initial_channel)$( .and(&$other_channels) )*
    }
}

/// An internal macro used by the [`junction_dec`](junction_dec) macro.
///
/// The [`channel_def`](channel_def) macro is used internally by the
/// [`junction_dec`](junction_dec) macro. For this reason it needs to be
/// made available as part of the Public API of the [Rusty
/// Junctions](https://crates.io/crates/rusty_junctions) crate.
#[macro_export]
macro_rules! channel_def {
    (Send, $junction:ident, $name:ident, $type:ty) => {
        let $name = $junction.send_channel::<$type>();
    };
    (Recv, $junction:ident, $name:ident, $type:ty) => {
        let $name = $junction.recv_channel::<$type>();
    };
    (Bidir, $junction:ident, $name:ident, $type:ty) => {
        let $name = $junction.bidir_channel::<$type>();
    };
}

/// Define an entire
/// [`Junction`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
/// using a single block.
///
/// A
/// [`Junction`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
/// is the smallest unit of distribution for the Join Calculus.  This
/// syntax allows for an entire junction to be defined as a single block.
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
/// ## Single Junction Declarative Macro API
/// However, it is also possible to use this alternative (simpler?) syntax for defining the entire
/// [`Junction`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html).
/// ```rust
/// let (name, value, mut handle) = junction_dec! {
///     name as Send::String,
///     value as Send::i32,
///     |name, value| {
///         println!("Single Junction Declarative Macro API: {name} {value}");
///     },
/// };
/// value.send(1).unwrap();
/// name.send(String::from("Hello, World!")).unwrap();
/// // Explicitly stop the Controller
///
/// handle.stop();
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
/// # Disadvantages
/// * When using the [`junction_dec`](junction_dec) macro you are required
///   to bring the [`when`](when) and [`channel_def`](channel_def) into
///   the name space as well. This is due to a limitation of [Declarative
///   Macros](https://doc.rust-lang.org/reference/macros-by-example.html),
///   that they are not composed as single units, and are instead executed
///   in a function like manner albeit during compilation and not
///   runtime. A future implementation might utilise a Incremental TT
///   Muncher, and deeper recursion to avoid having calls to other macros
///   ([`when`](when) and [`channel_def`](channel_def)) which then need to
///   be included in the name space of the client.
///
/// * As you can see in the [Example](#Example) above the Controller
///   explicitly stopped, if we allowed it to be dropped from the inner
///   scope there would be no guarantee it would have time for the pattern
///   to fire.
///
#[macro_export]
macro_rules! junction_dec {
    ( $( $channel_name:ident as $mode:ident :: $type:ty),* $(,)+
      $( | $( $args:ident ),* | $function_block:block  ),* $(,)+ ) => {{
        let mut j = rusty_junctions::Junction::new();

        $( channel_def!($mode, j, $channel_name, $type); )*

        $(
            when!(j; $( $args ),* ).then_do(| $( $args ),* | $function_block );
        )*

        let controller_handle = j
            .controller_handle()
            .expect("Failed to get controller handle");

        ( $( $channel_name ),* , controller_handle)
    }};
}
