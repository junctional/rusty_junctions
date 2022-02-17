#![deny(missing_docs)]

//! Crate providing all of the macro backed functionality provided to the
//! [`rusty_junctions`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
//! crate.
//!
//! This crates is used as a facade re-exporting the underlying
//! declarative and procedural macro crates, allowing for a single
//! dependency to encapsulate the current organisational restrictions that
//! are enforced when using procedural macros.
//!
//! This crate is comprised of two modules [`client`](client) and
//! [`library`](library). Each of which providing the functionality for
//! there respective areas.
//!
//! See the module level documentation for further details.

/// The [`client`](client) module provides all of the macro functionality
/// used by a client of the
/// [`rusty_junctions`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/index.html)
/// crate.
///
/// Namely it provides a number of macros that can be used to conveniently
/// define the
/// [`patterns`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html),
/// [`channels`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/channels/index.html),
/// and
/// [`Junction`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
/// that are used to declaratively express the concurrent computation.
///
/// See the crate level documentation for further details.
pub mod client {
    pub use client_api_macro::{channel_def, junction_dec, when};
    pub use client_api_proc_macro::junction;
}

/// The [`library`](library) module provides all of the macro
/// functionality used when generating the
/// [`rusty_junctions`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/index.html)
/// crate.
///
/// Namely it provides a number of macros that can be used to conveniently
/// generate the
/// [`patterns`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/patterns/index.html),
/// [`channels`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/channels/index.html),
/// and
/// [`Junction`](https://docs.rs/rusty_junctions/0.1.0/rusty_junctions/struct.Junction.html)
/// that are used to declaratively express the concurrent computation.
///
/// See the crate level documentation for further details.
pub mod library {
    pub use library_generation_proc_macro::{
        library_generate as generate, JoinPattern, PartialPattern, TerminalPartialPattern,
    };
}
