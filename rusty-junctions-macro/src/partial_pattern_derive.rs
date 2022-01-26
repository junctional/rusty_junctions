use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericParam,
    Generics, TypeParam, __private::TokenStream2,
    AngleBracketedGenericArguments, GenericArgument, Path, PathArguments, PathSegment, Type,
    TypePath,
};
use rusty_junctions_utils::Module;

pub fn partial_pattern_from_derive(input: DeriveInput, is_terminal_pattern: bool) -> TokenStream2 {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = input;

    let partial_pattern_name = ident;
    let join_pattern = JoinPattern::from_partial_pattern_name(&partial_pattern_name);
    let join_pattern_name = join_pattern.clone().ident;
    let transform_function = join_pattern.transform_function();
    let generic_type_parameters = generic_type_parameters(generics);

    let fields = get_fields(data);
    let ParsedChannels {
        field_names,
        field_ty_with_generics,
        function_args,
        return_type,
    } = ParsedChannels::from_fields(fields);

    let channel_number = field_names.len();
    let mut module = Module::from_usize(channel_number);
    let module_name = module.ident();
    let next_module_name = module.next().expect("Will always be higher module").name();

    let new_method = new_method(
        &partial_pattern_name,
        &generic_type_parameters,
        &field_names,
        &field_ty_with_generics,
        join_pattern.requires_junction_id(),
    );

    // TODO: Fix this
    let then_do_method = then_do_method(
        &join_pattern_name,
        &field_names,
        &module_name,
        &function_args,
        return_type,
        transform_function,
    );

    let and_method_fn = (!is_terminal_pattern).then(|| {
        and_method(
            "and",
            &next_module_name,
            "send_channel",
            "Send",
            &generic_type_parameters,
            &field_names,
        )
    });
    let and_recv_method_fn = (!is_terminal_pattern).then(|| {
        and_method(
            "recv",
            &next_module_name,
            "recv_channel",
            "Recv",
            &generic_type_parameters,
            &field_names,
        )
    });
    let and_bidir_method_fn = (!is_terminal_pattern).then(|| {
        and_method(
            "bidir",
            &next_module_name,
            "bidir_channel",
            "Bidir",
            &generic_type_parameters,
            &field_names,
        )
    });

    let output = quote! {
        impl< #( #generic_type_parameters ,)* > #partial_pattern_name < #( #generic_type_parameters ,)* >
        where
            #( #generic_type_parameters: std::any::Any + std::marker::Send ,)*
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

fn get_fields(data: Data) -> Vec<Field> {
    match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named.into_iter().collect(),
        _ => panic!("A PartialPattern should only be created from a struct with named fields."),
    }
}

fn generic_type_parameters(generics: Generics) -> Vec<Ident> {
    generics
        .params
        .into_iter()
        .filter_map(|p| match p {
            GenericParam::Type(TypeParam { ident, .. }) => Some(ident),
            _ => None,
        })
        .collect::<Vec<Ident>>()
}

#[derive(Debug, Clone)]
struct JoinPattern {
    pub name: String,
    pub ident: Ident,
}

impl JoinPattern {
    pub fn from_partial_pattern_name(partial_pattern_name: &Ident) -> Self {
        let name = partial_pattern_name
            .to_string()
            .replace("PartialPattern", "JoinPattern");
        let ident = Ident::new(&name, Span::call_site());

        Self { name, ident }
    }

    pub fn transform_function(&self) -> TokenStream2 {
        match self.name.as_str() {
            "SendJoinPattern" => quote!(transform_send(f)),
            "RecvJoinPattern" => quote!(transform_recv(f)),
            "BidirJoinPattern" => quote!(transform_bidir(f)),
            _ => panic!("Unsuppoted Module"),
        }
    }

    pub fn requires_junction_id(&self) -> bool {
        match self.name.as_str() {
            "SendJoinPattern" => true,
            _not_send_pattern => false,
        }
    }
}

pub struct ParsedChannels {
    pub field_names: Vec<TokenStream2>,
    pub field_ty_with_generics: Vec<TokenStream2>,
    pub function_args: Vec<Ident>,
    pub return_type: TokenStream2,
}

impl ParsedChannels {
    /// Create a `ParsedChannels` struct from a `Vec<Field>`
    pub fn from_fields(fields: Vec<Field>) -> Self {
        let mut field_names: Vec<TokenStream2> = Vec::new();
        let mut field_ty_with_generics: Vec<TokenStream2> = Vec::new();
        let mut function_args: Vec<Ident> = Vec::new();
        let mut return_type: Option<Ident> = None;

        let channels: Vec<Field> = fields.into_iter().filter(|f| is_channel(f)).collect();

        for Field { ident, ty, .. } in channels {
            // Extract the field name
            let field_name = match ident {
                Some(name) => name,
                _field_not_named => continue,
            };

            // Extract the field type and its generics
            let (field_type, mut field_generics) = if let Some(PathSegment {
                ident,
                arguments:
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
            }) = get_last_type_path_segment(ty)
            {
                let generics: Vec<Ident> = args
                    .into_iter()
                    .filter_map(|a| match a {
                        GenericArgument::Type(t) => get_last_type_path_segment(t),
                        _ => None,
                    })
                    .map(|ps| ps.ident)
                    .collect();
                (ident, generics)
            } else {
                continue;
            };

            // Add the field name to the Vec
            field_names.push(quote!(#field_name));

            // Create the combined type and generics, and add to the Vec
            let combined_ty_with_generics = quote!(#field_type< #( #field_generics ,)* >);
            field_ty_with_generics.push(combined_ty_with_generics);

            // Depending on the type of the field perform a different action
            match field_type.to_string().as_str() {
                "StrippedSendChannel" => {
                    let generic_type = field_generics
                        .pop()
                        .expect("Send channel should have a generic type parameter");
                    function_args.push(generic_type);
                }
                "StrippedRecvChannel" => {
                    let generic_type = field_generics
                        .pop()
                        .expect("Send channel should have a generic type parameter");
                    match return_type {
                        Some(_) => panic!("There can only be a single return type"),
                        None => return_type = Some(generic_type),
                    }
                }
                "StrippedBidirChannel" => {
                    let generic_type = field_generics
                        .pop()
                        .expect("Send channel should have a generic type parameter");
                    let generic_return_type = field_generics
                        .pop()
                        .expect("Send channel should have a generic type parameter");
                    function_args.push(generic_type);
                    match return_type {
                        Some(_) => panic!("There can only be a single return type"),
                        None => return_type = Some(generic_return_type),
                    }
                }
                _other_field_type => continue,
            }
        }

        let return_type = return_type.map_or(quote!(()), |rt| quote!(#rt));

        ParsedChannels {
            field_names,
            field_ty_with_generics,
            function_args,
            return_type,
        }
    }
}

fn is_channel(field: &Field) -> bool {
    let Field { ty, .. } = field;
    let last_segment = match ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => segments,
        _ => return false,
    }
    .into_iter()
    .last();

    let channel_type = match last_segment {
        Some(s) => s.ident.to_string(),
        None => return false,
    };

    match channel_type.as_str() {
        "StrippedSendChannel" | "StrippedRecvChannel" | "StrippedBidirChannel" => true,
        _ => false,
    }
}

fn get_last_type_path_segment(ty: Type) -> Option<PathSegment> {
    let last_segment = match ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => Some(segments),
        _ => None,
    }
    .map(|s| s.into_iter().last());

    match last_segment {
        Some(Some(s)) => Some(s),
        _ => None,
    }
}
