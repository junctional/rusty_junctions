use proc_macro2::Span;
use quote::quote;
use std::str::FromStr;
use syn::{
    __private::TokenStream2,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Block, Ident, Token, Type, TypeTuple,
};

#[derive(Eq, PartialEq)]
pub enum Mode {
    Send,
    Recv,
    Bidir,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Send" => Ok(Mode::Send),
            "Bidir" => Ok(Mode::Bidir),
            "Recv" => Ok(Mode::Recv),
            _ => Err(format!(
                "Invalid channel mode {s}, must be one of [Send|Recv|Bidir]"
            )),
        }
    }
}

pub struct ControlDefinition {
    name: Ident,
}

impl Parse for ControlDefinition {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if input.is_empty() {
            panic!("Invalid input for ControlDefinition");
        }

        let name = input.parse::<Ident>()?;
        let _as_token = input.parse::<Token![as]>()?;
        let _ty = input.parse::<Ident>()?;
        let _comma = input.parse::<Token![,]>()?;

        Ok(Self { name })
    }
}

pub struct ChannelDefinition {
    name: Ident,
    mode: Mode,
    ty: TokenStream2,
}

impl Parse for ChannelDefinition {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if input.is_empty() {
            panic!("Invalid input for ChannelDefinition");
        }

        let name = input.parse::<Ident>()?;
        let _as_token = input.parse::<Token![as]>()?;
        let mode = Mode::from_str(&input.parse::<Ident>()?.to_string())
            .map_err(|e| syn::Error::new(Span::call_site(), e))?;
        let _namespace_token = input.parse::<Token![::]>()?;

        let ty = match mode {
            Mode::Bidir => match input.parse::<Type>() {
                Ok(Type::Tuple(TypeTuple { elems, .. })) => Some(elems),
                _ => None,
            }
            .map(|es| es.into_iter().collect::<Vec<Type>>())
            .map(|es| {
                let elem1 = es.get(0);
                let elem2 = es.get(1);
                match (elem1, elem2) {
                    (Some(e1), Some(e2)) => Some(quote!(#e1, #e2)),
                    _ => None,
                }
            })
            .flatten(),
            Mode::Send | Mode::Recv => input.parse::<Type>().map(|t| quote!(#t)).ok(),
        }
        .expect("Invalid Bidirectional Channel Types");

        Ok(Self { name, mode, ty })
    }
}

impl ChannelDefinition {
    fn to_tokens(&self, junction_name: &Ident) -> TokenStream2 {
        let ChannelDefinition { name, mode, ty } = self;
        match mode {
            Mode::Send => quote! {
                #[allow(unused_variables)]
                let #name = #junction_name.send_channel::<#ty>();
            },
            Mode::Bidir => quote!( let #name = #junction_name.bidir_channel::<#ty>(); ),
            Mode::Recv => quote!( let #name = #junction_name.recv_channel::<#ty>(); ),
        }
    }
}

pub struct JoinPattern {
    pub channels: Vec<Ident>,
    pub function_block: Block,
}

impl JoinPattern {
    pub fn to_tokens(
        &self,
        channel_definitions: &Vec<ChannelDefinition>,
        junction_name: &Ident,
    ) -> TokenStream2 {
        // Create a map of all of the channel names to the channels If the map
        // already contains an element we panic, there should be no duplicates
        let mut channels_map = std::collections::HashMap::new();
        channel_definitions.into_iter().for_each(|chan| {
            if let Some(_) = channels_map.insert(&chan.name, chan) {
                panic!("Channel Definitions contains a duplicate channel.")
            }
        });

        // Get all of the channels and ensure that they have been declared
        let channels = self
            .channels
            .iter()
            .map(|chan| {
                channels_map
                    .get(chan)
                    .expect("Join Pattern contains a channel that was not declared.")
            })
            .collect::<Vec<&&ChannelDefinition>>();

        // Get the first channel for the pattern
        if channels.len() == 0 {
            panic!("Join Pattern has no input channels");
        }

        // Split the channels into their mode: Send, Recv, and Bidir
        let mut send_channels = Vec::new();
        let mut bidir_channels = Vec::new();
        let mut recv_channels = Vec::new();
        channels.into_iter().for_each(|chan| {
            match chan.mode {
                Mode::Send => send_channels.push(chan),
                Mode::Recv => recv_channels.push(chan),
                Mode::Bidir => bidir_channels.push(chan),
            };
        });

        // Ensure we have at most one Recv or Bidir channel
        if bidir_channels.len() > 0 && recv_channels.len() > 0
            || bidir_channels.len() > 1
            || recv_channels.len() > 1
        {
            panic!("There can be a maximum of one Recv or Bidir Channel");
        }

        let when_token = match (
            send_channels.get(0),
            recv_channels.get(0),
            bidir_channels.get(0),
        ) {
            (None, None, Some(chan)) => {
                let name = &chan.name;
                bidir_channels.remove(0);
                quote!(when_bidir(&#name))
            }
            (None, Some(chan), None) => {
                let name = &chan.name;
                recv_channels.remove(0);
                quote!(when_recv(&#name))
            }
            (Some(chan), _, _) => {
                let name = &chan.name;
                send_channels.remove(0);
                quote!(when(&#name))
            }
            _ => panic!("Invalid input channels to the Join Pattern"),
        };

        let and_tokens = [
            send_channels.to_vec(),
            recv_channels,
            bidir_channels.to_vec(),
        ]
        .concat()
        .into_iter()
        .map(|chan| match chan {
            ChannelDefinition {
                name,
                mode: Mode::Send,
                ..
            } => quote!(and(&#name)),
            ChannelDefinition {
                name,
                mode: Mode::Recv,
                ..
            } => quote!(and_recv(&#name)),
            ChannelDefinition {
                name,
                mode: Mode::Bidir,
                ..
            } => quote!(and_bidir(&#name)),
        });

        // Get all of the Send and Bidir channels for the join pattern, these
        // need to be included in the closure.
        // let channels = &self.channels;
        let channels = &self
            .channels
            .iter()
            .filter_map(|chan| match channels_map.get(&chan) {
                Some(ChannelDefinition { name, mode, .. }) if mode != &Mode::Recv => Some(name),
                _not_send_bidir_channel => None,
            })
            .collect::<Vec<&Ident>>();
        let function = &self.function_block;

        // TODO: The move here could introduce issues
        quote! {
            #junction_name . #when_token #(. #and_tokens)* .then_do( move | #(#channels ,)* | #function );
        }
    }
}

impl Parse for JoinPattern {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if input.is_empty() {
            panic!("Invalid input for JoinPattern");
        }

        let _left_pipe = input.parse::<Token![|]>()?;

        let mut channels = Vec::new();
        loop {
            channels.push(input.parse::<Ident>()?);
            if input.peek(Token![|]) {
                let _right_pipe = input.parse::<Token![|]>()?;
                break;
            }
            let _comma = input.parse::<Token![,]>()?;
        }

        let function_block = input.parse::<Block>()?;

        Ok(Self {
            channels,
            function_block,
        })
    }
}

pub struct Junction {
    junction: Option<ControlDefinition>,
    channels: Vec<ChannelDefinition>,
    super_statements: Vec<TokenStream2>,
    join_patterns: Vec<JoinPattern>,
}

impl Parse for Junction {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if input.is_empty() {
            panic!("Invalid input for Junction");
        }

        let junction = match input.fork().parse::<ControlDefinition>() {
            Ok(_) => input.parse::<ControlDefinition>().ok(),
            _ => None,
        };

        // Parse all of the channels from the list
        let mut channels: Punctuated<ChannelDefinition, Token![,]> = Punctuated::new();
        loop {
            channels.push(input.parse::<ChannelDefinition>()?);
            let _comma = input.parse::<Token![,]>()?;
            if input.peek(Token![|]) {
                break;
            }
        }

        // The remaining tokens in the input, are just the JoinPatterns
        let join_patterns = Punctuated::<JoinPattern, Token![,]>::parse_terminated(&input)?
            .into_iter()
            .collect::<Vec<JoinPattern>>();

        let channels = channels.into_iter().collect::<Vec<ChannelDefinition>>();

        let super_statements = channels
            .iter()
            .map(|ChannelDefinition { name, .. }| {
                let super_name =
                    Ident::new(&format!("{}_super", name.to_string()), Span::call_site());
                quote! {
                    #[allow(unused_variables)]
                    let #super_name = #name.clone();
                }
            })
            .collect::<Vec<TokenStream2>>();

        Ok(Self {
            junction,
            channels,
            super_statements,
            join_patterns,
        })
    }
}

impl From<Junction> for TokenStream2 {
    fn from(junction: Junction) -> Self {
        // Create a temporary junction named with a UUID
        let mut uuid_buffer = uuid::Uuid::encode_buffer();
        let uuid = uuid::Uuid::new_v4()
            .to_simple()
            .encode_lower(&mut uuid_buffer);
        let junction_name = Ident::new(&format!("junction_{}", uuid), Span::call_site());

        let return_junction = junction.junction.map(|j| {
            let name = j.name;
            quote!(let mut #name = #junction_name;)
        });

        let super_stmts = junction.super_statements;
        let join_pattern_definitions = junction
            .join_patterns
            .iter()
            .map(|pat| {
                let pattern = pat.to_tokens(&junction.channels, &junction_name);
                quote! {
                    #( #super_stmts )*
                    #pattern
                }
            })
            .collect::<Vec<TokenStream2>>();

        let channel_definitions = junction
            .channels
            .iter()
            .map(|chan| chan.to_tokens(&junction_name))
            .collect::<Vec<TokenStream2>>();


        let output = quote! {
            let #junction_name = rusty_junctions::Junction::new();
            #( #channel_definitions )*
            #( #join_pattern_definitions )*
            #return_junction
        };

        output.into()
    }
}
