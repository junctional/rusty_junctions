use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitInt,
};

pub struct Module {
    number: usize,
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

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn ident(&self) -> Ident {
        let name = self.name();
        Ident::new(&name, Span::call_site())
    }

    pub fn type_parameters(&self) -> Vec<Ident> {
        (1..=self.number())
            .map(|n| Ident::new(&"A".repeat(n), Span::call_site()))
            .collect::<Vec<Ident>>()
    }
}

impl std::iter::Iterator for Module {
    type Item = Self;
    fn next(&mut self) -> Option<<Self as std::iter::Iterator>::Item> {
        let number = self.number + 1;
        Some(Self { number })
    }
}

impl Parse for Module {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        if input.is_empty() {
            panic!("Invalid input for Module");
        }

        let number = input
            .parse::<LitInt>()?
            .to_string()
            .parse::<usize>()
            .map_err(|_| syn::Error::new(Span::call_site(), "ParseStream was not a usize"))?;

        Ok(Self { number })
    }
}
