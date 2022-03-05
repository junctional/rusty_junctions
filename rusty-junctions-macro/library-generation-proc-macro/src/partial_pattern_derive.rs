use crate::Module;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::str::FromStr;
use syn::{
    AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    GenericArgument, GenericParam, Generics, Path, PathArguments, PathSegment, Type, TypeParam,
    TypePath, __private::TokenStream2,
};

struct JoinPattern {
    pub join_pattern_ident: Ident,
    pub type_param: Vec<Ident>,
    pub return_type: TokenStream2,
    pub fn_param: Vec<Ident>,
    pub transform_function: TokenStream2,
    pub requires_junction_id: bool,
    pub field_names: Vec<TokenStream2>,
    pub field_types: Vec<TokenStream2>,
}

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    Send,
    Recv,
    Bidir,
}

impl FromStr for Mode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SendPartialPattern" => Ok(Mode::Send),
            "RecvPartialPattern" => Ok(Mode::Recv),
            "BidirPartialPattern" => Ok(Mode::Bidir),
            _unsupported_mode => Err("Unsupported Channel Type".to_string()),
        }
    }
}

impl JoinPattern {
    pub fn new(pattern_name: &Ident, generics: Generics, data: Data) -> Self {
        let mode = Mode::from_str(&pattern_name.to_string()).unwrap();

        let type_param = generics
            .params
            .into_iter()
            .filter_map(|p| match p {
                GenericParam::Type(TypeParam { ident, .. }) => Some(ident),
                _ => None,
            })
            .collect::<Vec<Ident>>();

        let mut fn_param = type_param.to_vec();

        let return_type = match mode {
            Mode::Send => quote!(()),
            Mode::Recv | Mode::Bidir => {
                let last_param = fn_param.pop();
                quote!(#last_param)
            }
        };

        let join_pattern_name = pattern_name
            .to_string()
            .replace("PartialPattern", "JoinPattern");
        let join_pattern_ident = Ident::new(&join_pattern_name, Span::call_site());

        let transform_function = match mode {
            Mode::Send => quote!(transform_send(f)),
            Mode::Recv => quote!(transform_recv(f)),
            Mode::Bidir => quote!(transform_bidir(f)),
        };

        let requires_junction_id = mode == Mode::Send;

        let (field_names, field_types) = Self::parse_data(data);

        Self {
            join_pattern_ident,
            type_param,
            return_type,
            fn_param,
            transform_function,
            requires_junction_id,
            field_names,
            field_types,
        }
    }

    pub fn parse_data(data: Data) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
        let mut field_names: Vec<TokenStream2> = Vec::new();
        let mut field_ty_with_generics: Vec<TokenStream2> = Vec::new();

        let fields = match data {
            Data::Struct(DataStruct {
                fields: Fields::Named(FieldsNamed { named, .. }),
                ..
            }) => named.into_iter().collect::<Vec<Field>>(),
            _ => panic!("A PartialPattern should only be created from a struct with named fields."),
        };

        fields
            .into_iter()
            .filter(|f| is_channel(f))
            .for_each(|Field { ident, ty, .. }| {
                let field_name = ident.expect("Fields should always be named");
                field_names.push(quote!(#field_name));

                let (field_type, field_generics) = match get_last_type_path_segment(ty) {
                    Some(PathSegment {
                        ident,
                        arguments:
                            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                args, ..
                            }),
                    }) => {
                        let generics: Vec<Ident> = args
                            .into_iter()
                            .filter_map(|a| match a {
                                GenericArgument::Type(t) => get_last_type_path_segment(t),
                                _ => None,
                            })
                            .map(|ps| ps.ident)
                            .collect();

                        Some((ident, generics))
                    }
                    _ => None,
                }
                .expect("Invalid PartialPattern Fields");

                field_ty_with_generics.push(quote!(#field_type< #( #field_generics ,)* >));
            });

        (field_names, field_ty_with_generics)
    }
}

pub fn partial_pattern_from_derive(input: DeriveInput, is_terminal_pattern: bool) -> TokenStream2 {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = input;

    let JoinPattern {
        join_pattern_ident,
        type_param,
        return_type,
        fn_param,
        transform_function,
        requires_junction_id,
        field_names,
        field_types,
    } = JoinPattern::new(&ident, generics, data);

    let partial_pattern_name = ident;

    let mut module = Module::from_usize(field_names.len());
    let module_name = module.ident();
    let next_module_name = module.next().expect("Will always be higher module").name();

    let new_method = new_method(
        &partial_pattern_name,
        &type_param,
        &field_names,
        &field_types,
        requires_junction_id,
    );

    // TODO: Fix this
    let then_do_method = then_do_method(
        &join_pattern_ident,
        &field_names,
        &module_name,
        &fn_param,
        return_type,
        transform_function,
    );

    let and_method_fn = (!is_terminal_pattern).then(|| {
        and_method(
            "and",
            &next_module_name,
            "send_channel",
            "Send",
            &type_param,
            &field_names,
        )
    });
    let and_recv_method_fn = (!is_terminal_pattern).then(|| {
        and_method(
            "and_recv",
            &next_module_name,
            "recv_channel",
            "Recv",
            &type_param,
            &field_names,
        )
    });
    let and_bidir_method_fn = (!is_terminal_pattern).then(|| {
        and_method(
            "and_bidir",
            &next_module_name,
            "bidir_channel",
            "Bidir",
            &type_param,
            &field_names,
        )
    });

    let output = quote! {
        impl< #( #type_param ,)* > #partial_pattern_name < #( #type_param,)* >
        where
            #( #type_param : std::any::Any + std::marker::Send ,)*
        {
            #new_method
            #then_do_method
            #and_method_fn
            #and_recv_method_fn
            #and_bidir_method_fn
        }
    };

    output.into()
}

fn then_do_method(
    join_pattern_name: &Ident,
    channel_names: &Vec<TokenStream2>,
    module_name: &Ident,
    function_args: &Vec<Ident>,
    return_type: TokenStream2,
    transform_function: TokenStream2,
) -> TokenStream2 {
    quote! {
        pub fn then_do<F>(self, f: F)
        where
            F: Fn( #( #function_args ,)* ) -> #return_type + std::marker::Send + std::clone::Clone + 'static,
        {
            let join_pattern = #join_pattern_name {
                #( #channel_names: self.#channel_names.id() ,)*
                f: crate::function_transforms::#module_name::#transform_function,
            };

            join_pattern.add(self.sender);
        }
    }
}

fn and_method(
    specific_method: &str,
    next_module: &str,
    channel_name: &str,
    pattern_type: &str,
    generic_type_parameters: &Vec<Ident>,
    channel_names: &Vec<TokenStream2>,
) -> TokenStream2 {
    let method_name = Ident::new(specific_method, Span::call_site());
    let next_module = Ident::new(next_module, Span::call_site());
    let channel_name = Ident::new(channel_name, Span::call_site());
    let channel_type = Ident::new(&format!("{}Channel", pattern_type), Span::call_site());
    let created_partial_pattern = Ident::new(
        &format!("{}PartialPattern", pattern_type),
        Span::call_site(),
    );
    let junction_id = match pattern_type {
        "Send" => Some(quote!(self.junction_id,)),
        _not_send_channel => None,
    };

    // TODO: Fix this hack for the generic parameter of the other method
    let method_generic_param = match pattern_type {
        "Bidir" => vec![
            Ident::new("A", Span::call_site()),
            Ident::new("AA", Span::call_site()),
        ],
        _not_bidir => vec![Ident::new("A", Span::call_site())],
    };

    quote! {
        pub fn #method_name< #( #method_generic_param ,)* >(
            self,
            #channel_name: &crate::channels::#channel_type<#( #method_generic_param ,)* >,
        ) -> crate::patterns::#next_module::#created_partial_pattern< #( #generic_type_parameters ,)* #( #method_generic_param ,)* >
        where
            #( #method_generic_param: std::any::Any + std::marker::Send, )*
        {
            if #channel_name.junction_id() != self.junction_id {
                panic!("A Join Pattern only supports channels from the same Junction");
            }
            super::#next_module::#created_partial_pattern::new(
                #junction_id
                #( self.#channel_names ,)*
                #channel_name.strip(),
                self.sender,
            )
        }
    }
}

fn new_method(
    partial_pattern_name: &Ident,
    generic_type_parameters: &Vec<Ident>,
    channel_names: &Vec<TokenStream2>,
    channel_types: &Vec<TokenStream2>,
    include_junction_id: bool,
) -> TokenStream2 {
    let junction_id_arg =
        include_junction_id.then(|| quote!(junction_id: crate::types::ids::JunctionId,));
    let junction_id_field = include_junction_id.then(|| quote!(junction_id,));

    quote! {
        pub(crate) fn new(
            #junction_id_arg
            #( #channel_names: crate::channels::#channel_types ,)*
            sender: std::sync::mpsc::Sender<crate::types::Packet>,
        ) -> #partial_pattern_name< #( #generic_type_parameters ,)* > {
            #partial_pattern_name {
                #junction_id_field
                #( #channel_names , )*
                sender,
            }
        }
    }
}

fn is_channel(field: &Field) -> bool {
    let Field { ty, .. } = field;
    match ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => segments,
        _ => return false,
    }
    .into_iter()
    .last()
    .map(|f| match f.ident.to_string().as_str() {
        "StrippedSendChannel" | "StrippedRecvChannel" | "StrippedBidirChannel" => true,
        _ => false,
    })
    .unwrap_or(false)
}

fn get_last_type_path_segment(ty: Type) -> Option<PathSegment> {
    match ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => Some(segments),
        _ => None,
    }
    .map(|s| s.into_iter().last())
    .flatten()
}
