use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericParam,
    Generics, TypeParam, __private::TokenStream2,
};

mod channels;

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

        Self {
            name,
            ident,
        }
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

struct Module {
    // pub name: String,
    pub ident: Ident,
}

impl Module {
    pub fn from_usize(number: usize) -> Self {
        let name = match number {
            0 => panic!("Invalid number of fields"),
            1 => "unary".to_string(),
            2 => "binary".to_string(),
            3 => "ternary".to_string(),
            n => format!("n{}ary", n),
        };
        let ident = Ident::new(&name, Span::call_site());

        Self {
            // name,
            ident,
        }
    }
}

#[proc_macro_derive(TerminalPartialPattern)]
pub fn terminal_partial_pattern(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input);

    let partial_pattern_name = ident;
    let join_pattern = JoinPattern::from_partial_pattern_name(&partial_pattern_name);
    let join_pattern_name = join_pattern.clone().ident;
    let transform_function = join_pattern.transform_function();
    let generic_type_parameters = generic_type_parameters(generics);

    let fields = get_fields(data);
    let channels::ParsedChannels {
        field_names,
        field_ty_with_generics,
        function_args,
        return_type,
    } = channels::ParsedChannels::from_fields(fields);

    let channel_number = field_names.len();
    let module_name = Module::from_usize(channel_number).ident;

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

    let output = quote! {
        impl< #( #generic_type_parameters ,)* > #partial_pattern_name < #( #generic_type_parameters ,)* >
        where
            #( #generic_type_parameters: std::any::Any + Send ,)*
        {
            #new_method
            #then_do_method
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
            F: Fn( #( #function_args ,)* ) -> #return_type + Send + Clone + 'static,
        {
            let join_pattern = #join_pattern_name {
                #( #channel_names: self.#channel_names.id() ,)*
                f: crate::function_transforms::#module_name::#transform_function,
            };

            join_pattern.add(self.sender);
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
            #( #channel_names: #channel_types ,)*
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
