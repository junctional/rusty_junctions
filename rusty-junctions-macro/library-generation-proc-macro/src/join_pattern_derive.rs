use crate::Module;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::__private::TokenStream2;
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Path, PathSegment, Type, TypePath,
};

pub fn join_pattern_from_derive(input: DeriveInput) -> TokenStream2 {
    let DeriveInput { ident, data, .. } = input;

    let join_pattern_name = ident.to_string().replace("Partial", "Join");
    let join_pattern_name = Ident::new(&join_pattern_name, Span::call_site());

    let channel_fields: Vec<Ident> = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named.into_iter().collect::<Vec<Field>>(),
        _ => panic!("A JoinPattern should only be created from a struct with named fields."),
    }
    .into_iter()
    .filter_map(|f| channel_field(f))
    .collect();

    let arity = channel_fields.len();
    let module_name = Module::from_usize(arity).ident();

    let function_args: Vec<TokenStream2> = std::iter::repeat(quote!(messages.remove(0)))
        .take(arity)
        .collect();

    let output = quote! {
        pub struct #join_pattern_name {
            #( #channel_fields: crate::types::ids::ChannelId ,)*
            f: crate::functions::#module_name::FnBox,
        }

        impl crate::join_pattern::JoinPattern for #join_pattern_name {
            fn channels(&self) -> Vec<crate::types::ids::ChannelId> {
                vec![ #( self.#channel_fields ,)* ]
            }

            /// Fire Join Pattern by running associated function in separate thread.
            fn fire(&self, mut messages: Vec<crate::types::Message>) -> std::thread::JoinHandle<()> {
                let f_clone = self.f.clone();

                std::thread::spawn(move || {
                    (*f_clone)( #( #function_args ,)* );
                })
            }
        }
    };

    output.into()
}

fn channel_field(field: Field) -> Option<Ident> {
    let Field { ident, ty, .. } = field;

    let segments = match ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => Some(segments),
        _ => None,
    }?;

    let last_segment = segments
        .into_iter()
        .last()
        .map(|PathSegment { ident, .. }| ident.to_string())?;

    match last_segment.as_str() {
        "StrippedSendChannel" | "StrippedRecvChannel" | "StrippedBidirChannel" => ident,
        _ => None,
    }
}
