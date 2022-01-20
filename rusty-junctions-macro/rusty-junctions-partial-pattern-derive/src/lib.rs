use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericParam,
    Generics, TypeParam, __private::TokenStream2,
};

mod channels;

#[proc_macro_derive(TerminalPartialPattern)]
pub fn terminal_partial_pattern(input: TokenStream) -> TokenStream {
    derive_pattern(input, true)
}

#[proc_macro_derive(PartialPattern)]
pub fn partial_pattern(input: TokenStream) -> TokenStream {
    derive_pattern(input, false)
}

fn derive_pattern(input: TokenStream, is_terminal_pattern: bool) -> TokenStream {
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
            #( #generic_type_parameters: std::any::Any + Send ,)*
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

struct Module {
    pub number: usize,
}

impl Module {
    pub fn from_usize(number: usize) -> Self {
        Self { number }
    }

    pub fn name(&self) -> String {
        let name = match self.number {
            0 => panic!("Invalid number of fields"),
            1 => "unary".to_string(),
            2 => "binary".to_string(),
            3 => "ternary".to_string(),
            n => format!("n{}ary", n),
        };

        name
    }

    pub fn ident(&self) -> Ident {
        let name = self.name();
        Ident::new(&name, Span::call_site())
    }
}

impl std::iter::Iterator for Module {
    type Item = Self;
    fn next(&mut self) -> Option<<Self as std::iter::Iterator>::Item> {
        let number = self.number + 1;
        Some(Self { number })
    }
}
