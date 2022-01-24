//! Crate implementing Join Patterns from the [Join Calculus](https://www.microsoft.com/en-us/research/wp-content/uploads/2017/01/join-tutorial.pdf) developed by
//! Cédric Fournet and Georges Gonthier.
//!
//! Join Patterns are based on message-passing concurrency and offer a declarative
//! way of expression concurrent computation that is entirely thread-safe and
//! requires no manual coordination efforts.
//!
//! See the example of a storage cell written using this crate:
//!
//! ```
//! use rusty_junctions::Junction;
//!
//! fn main() {
//!     /* Start of the Join Pattern setup. */
//!
//!     // Declare a new Junction to create new channels and construct new
//!     // Join Patterns based on them.
//!     let cell = Junction::new();
//!
//!     // New channel to retrieve the value of the storage cell.
//!     let get = cell.recv_channel::<i32>();
//!
//!     // New channel to update the value of the storage cell.
//!     let put = cell.send_channel::<i32>();
//!
//!     // New channel to swap the value of the storage cell for a new one and
//!     // retrieve the value that was just replaced.
//!     let swap = cell.bidir_channel::<i32, i32>();
//!
//!     // New channel that will actually carry the value so that at no point
//!     // any given thread will have possession over it so concurrency issues
//!     // are avoided by design.
//!     let val = cell.send_channel::<i32>();
//!
//!     // Set up some clones of the above channels we can move over to the
//!     // thread in which the function body of the Join Pattern will run.
//!     //
//!     // Clones of channels work like clones of the std::sync::mpsc::Sender
//!     // clones - any message sent from the clone will be received as if
//!     // sent from the original.
//!     let get_val = val.clone();
//!     let put_val = val.clone();
//!     let swap_val = val.clone();
//!     let val_val = val.clone();
//!
//!     // Declare a new Join Pattern to update the storage cell value. If
//!     // both the put and val channel have sent a message, meaning someone
//!     // requested a value update and there is a value to be updated, send
//!     // a new val message through one of val's clones that carries the
//!     // updated value.
//!     cell.when(&put).and(&val).then_do(move |new, _old| {
//!         println!(">> put-val pattern fired with new={}!", new);
//!         put_val.send(new).unwrap();
//!     });
//!
//!     // Declare a new Join Pattern to retrieve the storage cell value. If
//!     // both the get and val channel have sent a message, meaning someone
//!     // requested the value and there is a value to be given, return that
//!     // value and resend it through one of val's clones so that the value
//!     // is still available in future and not just consumed once.
//!     cell.when(&val).and_recv(&get).then_do(move |v| {
//!         println!(">> val-get pattern fired with v={}!", v);
//!
//!         get_val.send(v.clone()).unwrap();
//!
//!         v
//!     });
//!
//!     // Declare a new Join Pattern to swap the storage cell value with a
//!     // new one and retrieve the old. Essentially works like a combination
//!     // of the previous two Join Patterns, with one crucial distinction:
//!     // with this Join Pattern, the update of the value and the retrieval
//!     // of the old are atomic, meaning that it is guaranteed that even in
//!     // a multithreaded environment with many users accessing the storage
//!     // cell, the value retrieved is exactly the value that has been
//!     // updated.
//!     cell.when(&val).and_bidir(&swap).then_do(move |old, new| {
//!         println!(
//!             ">> val-swap pattern fired with old={} and new={}!",
//!             old, new
//!         );
//!         swap_val.send(new).unwrap();
//!
//!         old
//!     });
//!
//!     // Declare a new Join Pattern that mentions the same channel multiple
//!     // times, so if the val channel has sent two messages they will be
//!     // combined into a single messages sent by a clone of val. This ensures
//!     // that eventually, the storage cell will only keep a single value
//!     // around.
//!     cell.when(&val).and(&val).then_do(move |a, b| {
//!         println!(">> val-val pattern fired with a={} and b={}!", a, b);
//!         val_val.send(a + b).unwrap();
//!     });
//!
//!     /* End of the Join Pattern setup. */
//!
//!     // Initialise the storage cell by sending an initial value that
//!     // can be picked up in future executions of the above Join Patterns.
//!     val.send(1729).unwrap();
//!
//!     // Request a value update if one is available.
//!     put.send(42).unwrap();
//!
//!     // Send another value that will eventually get combined with the
//!     // existing one.
//!     val.send(1).unwrap();
//!
//!     // Request another value update.
//!     put.send(22).unwrap();
//!
//!     // Request the current value of the storage cell and print it.
//!     println!("get.recv()={}", get.recv().unwrap());
//!
//!     // Request a swap of the current value of the storage cell with a new
//!     // one and print the old value that is retrieved as a result.
//!     println!("swap.send_recv()={}", swap.send_recv(16).unwrap());
//!
//!     // Request the current value of the storage cell again and print it.
//!     println!("get.recv()={}", get.recv().unwrap());
//! }
//! ```
//!
//! The above example is reasonably complex to show off most of the
//! capabilities of this crate, as well as the general workflow of setting
//! up channels, then Join Patterns synchronising the channels and running
//! arbitrary code, followed by actually sending messages on the declared
//! channels to satisfy the conditions declared in the Join Patterns that will
//! trigger an execution of their function body.
//!
//! For more examples, visit the [`examples`](https://github.com/smueksch/rusty_junctions/tree/master/examples) folder in the [Rusty Junctions GitHub
//! repository](https://github.com/smueksch/rusty_junctions).

pub mod channels;
mod controller;
mod join_pattern;
mod junction;
mod types;

pub use junction::Junction;

mod function_transforms {
    //! Function transformers used to hide actual type signatures of functions stored
    //! with a Join Pattern and instead expose a generic interface that is easily stored.

    // Function transformers for functions stored with unary Join Patterns.
    rusty_junctions_macro::function_transform!(unary; T);

    // Function transformers for functions stored with binary Join Patterns.
    rusty_junctions_macro::function_transform!(binary; T, U);

    // Function transformers for functions stored with ternary Join Patterns.
    rusty_junctions_macro::function_transform!(ternary; T, U, V);
}

mod patterns {
    // Create all of the Send, Recv, and Bidir for the unary module
    rusty_junctions_macro::pattern!(1, false);

    // Create all of the Send, Recv, and Bidir for the unary module
    rusty_junctions_macro::pattern!(2, false);

    // Create all of the Send, Recv, and Bidir for the unary module
    rusty_junctions_macro::pattern!(3, true);
}
