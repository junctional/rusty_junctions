use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use quote::quote;
use rusty_junctions_utils::Module;
use syn::{
    parse_macro_input,
    Ident,
};

#[proc_macro]
pub fn function_transform(input: TokenStream) -> TokenStream {
    use syn::__private::TokenStream2;

    let module: Module = parse_macro_input!(input);
    let module_name = module.ident();
    let type_parameters = module.type_parameters();

    let mut send_types: Vec<Ident> = Vec::new();
    let mut send_function_args: Vec<Ident> = Vec::new();
    let mut send_stmts: Vec<TokenStream2> = Vec::new();

    let mut recv_types: Vec<Ident> = Vec::new();
    let mut recv_function_args: Vec<Ident> = Vec::new();
    let mut recv_stmts: Vec<TokenStream2> = Vec::new();

    type_parameters.iter().enumerate().for_each(|(i, t)| {
        let ty_ident = Ident::new(&t.to_string(), Span::call_site());

        let send_arg_ident = Ident::new(&format!("arg_{}", i), Span::call_site());
        let send_arg_stmt = quote! {*#send_arg_ident.downcast::<#t>().unwrap()};
        send_types.push(ty_ident.clone());
        send_function_args.push(send_arg_ident);
        send_stmts.push(send_arg_stmt);

        let recv_arg_ident = Ident::new(&format!("arg_{}", i), proc_macro2::Span::call_site());
        let recv_arg_stmt = quote! {*#recv_arg_ident.downcast::<#t>().unwrap()};
        recv_types.push(ty_ident.clone());
        recv_function_args.push(recv_arg_ident);
        recv_stmts.push(recv_arg_stmt);
    });

    let last_type = recv_types.pop();
    recv_function_args.pop();
    recv_stmts.pop();

    let output = quote! {
        pub(crate) mod #module_name {
            use crate::types::{functions, Message};
            use std::{any::Any, sync::mpsc::Sender};

            /// Transform function of `SendJoinPattern` to use `Message` arguments.
            pub(crate) fn transform_send<F, #(#send_types ,)* >(f: F) -> Box<impl functions::#module_name::FnBoxClone>
            where
                F: Fn( #(#send_types ,)* ) -> () + Send + Clone + 'static,
                #(#send_types: Any + Send + 'static ,)*
            {
                Box::new(move | #(#send_function_args: Message ,)* | {
                    f( #(#send_stmts ,)* );
                })
            }


            /// Transform function of `RecvJoinPattern` to use `Message` arguments.
            pub(crate) fn transform_recv<F, #(#recv_types ,)* R>(f: F) -> Box<impl functions::#module_name::FnBoxClone>
            where
                F: Fn( #(#recv_types ,)* ) -> R + Send + Clone + 'static,
                #(#recv_types: Any + Send + 'static ,)*
                R: Any + Send + 'static,
            {
                Box::new(
                    move | #(#recv_function_args: Message ,)* return_sender: Message| {
                        let return_sender = *return_sender.downcast::<Sender<R>>().unwrap();
                        return_sender.send(f( #(#recv_stmts ,)* )).unwrap();
                    },
                )
            }

            /// Transform function of `BidirJoinPattern` to use `Message` arguments.
            pub(crate) fn transform_bidir<F, #(#type_parameters ,)* R>(f: F) -> Box<impl functions::#module_name::FnBoxClone>
            where
                F: Fn( #(#send_types ,)* ) -> R + Send + Clone + 'static,
                #(#send_types: Any + Send + 'static ,)*
                R: Any + Send + 'static,
            {
                Box::new(
                    move | #(#recv_function_args: Message ,)* arg_3_and_sender: Message| {
                        let (arg_3, return_sender) =
                            *arg_3_and_sender.downcast::<(#last_type, Sender<R>)>().unwrap();

                        return_sender.send(f( #(#recv_stmts ,)* arg_3)).unwrap();
                    },
                )
            }
        }
    };

    output.into()
}
