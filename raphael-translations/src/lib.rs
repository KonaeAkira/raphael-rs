mod data;
use data::get_translations;

use proc_macro2::Literal;
use quote::quote;
use syn::{Expr, LitStr, Token, parse::Parse, parse_macro_input, punctuated::Punctuated};

mod translation;
mod util;

pub(crate) struct StringLiteralDetails {
    #[allow(dead_code)] // Used in `StringLiteralDetails::source_location_string`
    literal: Literal,
    intro: String,
    body: String,
    outro: String,
}

impl StringLiteralDetails {
    #[cfg(feature = "update-toml")]
    pub fn source_location_string(&self) -> String {
        let span = self.literal.span();
        let start = span.start();
        format!("{}:{}:{}", span.file(), start.line, start.column)
    }
}

impl From<&LitStr> for StringLiteralDetails {
    fn from(literal: &LitStr) -> Self {
        let literal = literal.token();
        let literal_text = literal.to_string();
        let (intro, rest) = literal_text.split_once('"').unwrap();
        let (body, outro) = rest.rsplit_once('"').unwrap();

        Self {
            literal,
            intro: format!("{intro}\""),
            body: body.to_string(),
            outro: format!("\"{outro}"),
        }
    }
}

struct MainArguments {
    locale: Expr,
    string_literal: LitStr,
}

impl Parse for MainArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let locale = input.parse()?;
        input.parse::<Token![,]>()?;
        let string_literal = input.parse()?;
        Ok(Self {
            locale,
            string_literal,
        })
    }
}

#[proc_macro]
pub fn t(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let MainArguments {
        locale,
        string_literal,
    } = parse_macro_input!(input as MainArguments);

    let translations = get_translations(StringLiteralDetails::from(&string_literal));

    let output = quote! {
        match #locale {
            #(#translations,)*
            _ => #string_literal,
        }
    };

    proc_macro::TokenStream::from(output)
}

struct FormatArguments {
    locale: Expr,
    string_literal: LitStr,
    format_args: Punctuated<Expr, Token![,]>,
}

impl Parse for FormatArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let locale = input.parse()?;
        input.parse::<Token![,]>()?;
        let string_literal = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        let format_args = input.parse_terminated(Expr::parse, Token![,])?;
        Ok(Self {
            locale,
            string_literal,
            format_args,
        })
    }
}

#[proc_macro]
pub fn t_format(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let FormatArguments {
        locale,
        string_literal,
        format_args,
    } = parse_macro_input!(input as FormatArguments);

    let (translation_locales, translation_format_strings): (Vec<_>, Vec<_>) =
        get_translations(StringLiteralDetails::from(&string_literal))
            .into_iter()
            .map(translation::Translation::split)
            .unzip();

    let output = quote! {
        match #locale {
            #(#translation_locales => format!(#translation_format_strings, #format_args),)*
            _ => format!(#string_literal, #format_args)
        }
    };

    proc_macro::TokenStream::from(output)
}
