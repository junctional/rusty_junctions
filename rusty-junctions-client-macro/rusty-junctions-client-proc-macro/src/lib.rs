use crate::single_junction::Junction;
use proc_macro::{self, TokenStream};
use syn::{__private::TokenStream2, parse_macro_input};

mod single_junction;

#[proc_macro]
pub fn junction(input: TokenStream) -> TokenStream {
    let junction: Junction = parse_macro_input!(input);
    let output: TokenStream2 = junction.into();
    output.into()
}
