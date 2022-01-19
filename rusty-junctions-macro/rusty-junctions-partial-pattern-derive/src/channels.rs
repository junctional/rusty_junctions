use proc_macro2::Ident;
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, Field, GenericArgument, Path, PathArguments, PathSegment, Type,
    TypePath, __private::TokenStream2,
};

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
