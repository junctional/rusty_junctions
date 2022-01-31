use proc_macro2::Span;
use quote::quote;
use rusty_junctions_utils::Module;
use syn::{Ident, __private::TokenStream2};

pub fn function_transform_from_module(module: Module) -> TokenStream2 {
    let module_name = module.ident();
    let type_parameters = module.type_parameters();

    // TODO: Do this without the mutable vec definition
    let mut send_types: Vec<Ident> = Vec::new();
    let mut send_function_args: Vec<Ident> = Vec::new();
    let mut send_stmts: Vec<TokenStream2> = Vec::new();

    module.type_parameters().enumerate().for_each(|(i, t)| {
        let send_arg_ident = Ident::new(&format!("arg_{}", i), Span::call_site());
        let send_arg_stmt = quote! {*#send_arg_ident.downcast::<#t>().unwrap()};
        send_types.push(t.clone());
        send_function_args.push(send_arg_ident);
        send_stmts.push(send_arg_stmt);
    });

    let mut recv_types: Vec<Ident> = send_types.to_vec();
    let mut recv_function_args: Vec<Ident> = send_function_args.to_vec();
    let mut recv_stmts: Vec<TokenStream2> = send_stmts.to_vec();
    let last_type = recv_types.pop();
    recv_function_args.pop();
    recv_stmts.pop();

    let output = quote! {
        pub(crate) mod #module_name {
            /// Transform function of `SendJoinPattern` to use `Message` arguments.
            pub(crate) fn transform_send<F, #(#send_types ,)* >(f: F) -> Box<impl crate::functions::#module_name::FnBoxClone>
            where
                F: Fn( #(#send_types ,)* ) -> () + std::marker::Send + Clone + 'static,
                #(#send_types: std::any::Any + std::marker::Send + 'static ,)*
            {
                Box::new(move | #(#send_function_args: crate::types::Message ,)* | {
                    f( #(#send_stmts ,)* );
                })
            }


            /// Transform function of `RecvJoinPattern` to use `Message` arguments.
            pub(crate) fn transform_recv<F, #(#recv_types ,)* R>(f: F) -> Box<impl crate::functions::#module_name::FnBoxClone>
            where
                F: Fn( #(#recv_types ,)* ) -> R + std::marker::Send + Clone + 'static,
                #(#recv_types: std::any::Any + std::marker::Send + 'static ,)*
                R: std::any::Any + std::marker::Send + 'static,
            {
                Box::new(
                    move | #(#recv_function_args: crate::types::Message ,)* return_sender: crate::types::Message| {
                        let return_sender = *return_sender.downcast::<std::sync::mpsc::Sender<R>>().unwrap();
                        return_sender.send(f( #(#recv_stmts ,)* )).unwrap();
                    },
                )
            }

            /// Transform function of `BidirJoinPattern` to use `Message` arguments.
            pub(crate) fn transform_bidir<F, #(#type_parameters ,)* R>(f: F) -> Box<impl crate::functions::#module_name::FnBoxClone>
            where
                F: Fn( #(#send_types ,)* ) -> R + std::marker::Send + Clone + 'static,
                #(#send_types: std::any::Any + std::marker::Send + 'static ,)*
                R: std::any::Any + std::marker::Send + 'static,
            {
                Box::new(
                    move | #(#recv_function_args: crate::types::Message ,)* arg_bi_and_sender: crate::types::Message| {
                        let (arg_bi, return_sender) =
                            *arg_bi_and_sender.downcast::<(#last_type, std::sync::mpsc::Sender<R>)>().unwrap();

                        return_sender.send(f( #(#recv_stmts ,)* arg_bi)).unwrap();
                    },
                )
            }
        }
    };

    output.into()
}
