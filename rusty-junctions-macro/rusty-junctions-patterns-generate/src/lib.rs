use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use quote::quote;
use rusty_junctions_utils::Module;
use syn::{parse_macro_input, Ident, Token, Type, LitBool,
          parse::{Parse, ParseStream}};

struct PatternInput {
    module: Module,
    final_pattern: bool
}

impl Parse for PatternInput {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if input.is_empty() {
            panic!("Invalid input for PatternInput");
        }

        let module = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let final_pattern = input
            .parse::<LitBool>()?
            .value();

        Ok(Self { module, final_pattern })
    }
}

#[proc_macro]
pub fn pattern(input: TokenStream) -> TokenStream {
    let PatternInput {module, final_pattern} = parse_macro_input!(input);

    let patterns_macro = if final_pattern {
        quote!(TerminalPartialPattern)
    } else {
        quote!(PartialPattern)
    };

    let module_name = module.ident();
    let number_of_send_channels = module.number() - 1;

    // TODO: Fix this we we support any number, not just the length of the alpabet - 1
    // We have we exclude A due to another hack in the derive partial patterns macro
    // That also needs to be fixed
    let alphabet = ('B'..='Z')
        .map(|c| c as char)
        .filter(|c| c.is_alphabetic())
        .collect::<Vec<char>>();

    let generics = alphabet
        .iter()
        .take(number_of_send_channels)
        .map(|c| Ident::new(&c.to_string(), Span::call_site()))
        .collect::<Vec<Ident>>();

    let channel_names = (0..number_of_send_channels)
        .map(|n| Ident::new(&format!("channel_{}", n), Span::call_site()))
        .collect::<Vec<Ident>>();

    let output = quote! {
        pub mod #module_name {
            use crate::join_pattern::JoinPattern;

            #[derive(rusty_junctions_macro::#patterns_macro, rusty_junctions_macro::JoinPattern)]
            /// `SendChannel` partial Join Pattern.
            pub struct SendPartialPattern< #( #generics , )* S> {
                junction_id: crate::types::ids::JunctionId,
                #( #channel_names: crate::channels::StrippedSendChannel< #generics > , )*
                specialist_channel: crate::channels::StrippedSendChannel<S>,
                sender: std::sync::mpsc::Sender<crate::types::Packet>,
            }

            #[derive(rusty_junctions_macro::TerminalPartialPattern, rusty_junctions_macro::JoinPattern)]
            /// `RecvChannel` partial Join Pattern.
            pub struct RecvPartialPattern< #( #generics , )* S> {
                // TODO: We need all send channels then one recv
                #( #channel_names: crate::channels::StrippedSendChannel< #generics > , )*
                specialist_channel: crate::channels::StrippedRecvChannel<S>,
                sender: std::sync::mpsc::Sender<crate::types::Packet>,
            }

            #[derive(rusty_junctions_macro::TerminalPartialPattern, rusty_junctions_macro::JoinPattern)]
            /// Bidirectional channel partial Join Pattern.
            pub struct BidirPartialPattern< #( #generics , )* S, R> {
                // TODO: We need all send channels then one bidir
                #( #channel_names: crate::channels::StrippedSendChannel< #generics > , )*
                specialist_channel: crate::channels::StrippedBidirChannel<S, R>,
                sender: std::sync::mpsc::Sender<crate::types::Packet>,
            }
        }
    };

    output.into()
}
