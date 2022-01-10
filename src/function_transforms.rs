//! Function transformers used to hide actual type signatures of functions stored
//! with a Join Pattern and instead expose a generic interface that is easily stored.

// Function transformers for functions stored with unary Join Patterns.
rusty_junctions_macro::function_transform!(unary; T);

// Function transformers for functions stored with binary Join Patterns.
rusty_junctions_macro::function_transform!(binary; T, U);

// Function transformers for functions stored with ternary Join Patterns.
rusty_junctions_macro::function_transform!(ternary; T, U, V);
