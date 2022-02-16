use proc_macro2::Span;
use quote::quote;
use crate::Module;
use syn::{Ident, __private::TokenStream2};

pub fn pattern_from_module(module: Module, final_pattern: bool) -> TokenStream2 {
    let module_name = module.ident();
    let number_of_send_channels = module.number() - 1;

    // TODO: Use the Module iterator to do this
    // Not creating a new module every time
    let generics = (1..=number_of_send_channels)
        .into_iter()
        .map(|n| Module::from_usize(n).ident())
        .collect::<Vec<Ident>>();

    let channel_names = (0..number_of_send_channels)
        .map(|n| Ident::new(&format!("channel_{}", n), Span::call_site()))
        .collect::<Vec<Ident>>();

    let patterns_macro = if final_pattern {
        quote!(TerminalPartialPattern)
    } else {
        quote!(PartialPattern)
    };

    let output = quote! {
        pub mod #module_name {
            use crate::join_pattern::JoinPattern;

            #[derive(rusty_junctions_library_macro::#patterns_macro, rusty_junctions_library_macro::JoinPattern)]
            /// `SendChannel` partial Join Pattern.
            pub struct SendPartialPattern< #( #generics , )* S> {
                junction_id: crate::types::ids::JunctionId,
                #( #channel_names: crate::channels::StrippedSendChannel< #generics > , )*
                specialist_channel: crate::channels::StrippedSendChannel<S>,
                sender: std::sync::mpsc::Sender<crate::types::Packet>,
            }

            #[derive(rusty_junctions_library_macro::TerminalPartialPattern, rusty_junctions_library_macro::JoinPattern)]
            /// `RecvChannel` partial Join Pattern.
            pub struct RecvPartialPattern< #( #generics , )* S> {
                // TODO: We need all send channels then one recv
                #( #channel_names: crate::channels::StrippedSendChannel< #generics > , )*
                specialist_channel: crate::channels::StrippedRecvChannel<S>,
                sender: std::sync::mpsc::Sender<crate::types::Packet>,
            }

            #[derive(rusty_junctions_library_macro::TerminalPartialPattern, rusty_junctions_library_macro::JoinPattern)]
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
