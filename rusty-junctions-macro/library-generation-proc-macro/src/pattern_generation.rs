use crate::Module;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, __private::TokenStream2};

pub fn pattern_from_module(module: Module, is_terminal_pattern: bool) -> TokenStream2 {
    let module_name = module.ident();
    let number_non_specialist_channels = module.number() - 1;

    let generics = module
        .type_parameters("B")
        .take(number_non_specialist_channels)
        .collect::<Vec<Ident>>();

    let channel_names = (0..number_non_specialist_channels)
        .map(|n| Ident::new(&format!("channel_{}", n), Span::call_site()))
        .collect::<Vec<Ident>>();

    let patterns_macro = if is_terminal_pattern {
        quote!(TerminalPartialPattern)
    } else {
        quote!(PartialPattern)
    };

    let library_path = quote!(rusty_junctions_macro::library);
    let output = quote! {
        pub mod #module_name {
            use crate::join_pattern::JoinPattern;

            #[derive(#library_path::#patterns_macro, #library_path::JoinPattern)]
            /// `SendChannel` partial Join Pattern.
            pub struct SendPartialPattern< #( #generics , )* S> {
                junction_id: crate::types::ids::JunctionId,
                #( #channel_names: crate::channels::StrippedSendChannel< #generics > , )*
                specialist_channel: crate::channels::StrippedSendChannel<S>,
                sender: std::sync::mpsc::Sender<crate::types::Packet>,
            }

            #[derive(#library_path::TerminalPartialPattern, #library_path::JoinPattern)]
            /// `RecvChannel` partial Join Pattern.
            pub struct RecvPartialPattern< #( #generics , )* S> {
                // TODO: We need all send channels then one recv
                #( #channel_names: crate::channels::StrippedSendChannel< #generics > , )*
                specialist_channel: crate::channels::StrippedRecvChannel<S>,
                sender: std::sync::mpsc::Sender<crate::types::Packet>,
            }

            #[derive(#library_path::TerminalPartialPattern, #library_path::JoinPattern)]
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
