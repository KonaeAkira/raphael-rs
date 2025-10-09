use std::str::FromStr;

use crate::{Context, Occurance, data::Translation};

use proc_macro::{Span, TokenStream, TokenTree};

macro_rules! ident {
    ($token:ident) => {
        TokenTree::Ident(proc_macro::Ident::new(
            stringify!($token),
            Span::call_site(),
        ))
    };
    (_) => {
        TokenTree::Ident(proc_macro::Ident::new(stringify!(_), Span::call_site()))
    };
}
macro_rules! braces {
    ($stream:ident) => {
        TokenTree::Group(proc_macro::Group::new(
            proc_macro::Delimiter::Brace,
            $stream,
        ))
    };
}
macro_rules! translation_match_rule {
	( $(::$ident:ident)* => $value:expr) => {
		[
			$(
				TokenTree::Punct(proc_macro::Punct::new(':', proc_macro::Spacing::Joint)),
				TokenTree::Punct(proc_macro::Punct::new(':', proc_macro::Spacing::Alone)),
				ident!($ident),
			)*
			TokenTree::Punct(proc_macro::Punct::new('=', proc_macro::Spacing::Joint)),
			TokenTree::Punct(proc_macro::Punct::new('>', proc_macro::Spacing::Alone)),
			TokenTree::Literal(proc_macro::Literal::from_str($value).unwrap()),
			TokenTree::Punct(proc_macro::Punct::new(',', proc_macro::Spacing::Alone)),
		]
	};
}
macro_rules! source_expr_match_rule {
    ($value:expr) => {
        [
            ident!(_),
            TokenTree::Punct(proc_macro::Punct::new('=', proc_macro::Spacing::Joint)),
            TokenTree::Punct(proc_macro::Punct::new('>', proc_macro::Spacing::Alone)),
            TokenTree::Literal(proc_macro::Literal::from_str($value).unwrap()),
            TokenTree::Punct(proc_macro::Punct::new(',', proc_macro::Spacing::Alone)),
        ]
    };
}
macro_rules! source_identifier_match_rule {
    ($ident:expr, $span:expr) => {
        [
            ident!(_),
            TokenTree::Punct(proc_macro::Punct::new('=', proc_macro::Spacing::Joint)),
            TokenTree::Punct(proc_macro::Punct::new('>', proc_macro::Spacing::Alone)),
            TokenTree::Ident(proc_macro::Ident::new($ident, $span)),
            TokenTree::Punct(proc_macro::Punct::new(',', proc_macro::Spacing::Alone)),
        ]
    };
}

pub fn generate_output_token_stream(translations: Vec<Translation>, ctx: &Context) -> TokenStream {
    let Context {
        text, occurance, ..
    } = ctx;
    let translation_token_streams = translations
        .iter()
        .map(|Translation(language_key, string)| {
            let string_literal = &format!("\"{}\"", string);
            TokenStream::from_iter(match *language_key {
                "de" => translation_match_rule!(::raphael_data::Locale::DE => string_literal),
                "fr" => translation_match_rule!(::raphael_data::Locale::FR => string_literal),
                "ja" => translation_match_rule!(::raphael_data::Locale::JP => string_literal),
                "ko" => translation_match_rule!(::raphael_data::Locale::KR => string_literal),
                _ => unreachable!(),
            })
        })
        .chain([TokenStream::from_iter(match occurance {
            Occurance::Identifier(span) => source_identifier_match_rule!(&text, *span),
            Occurance::Literal(_) => source_expr_match_rule!(&text),
        })]);
    let match_body = TokenStream::from_iter(translation_token_streams);
    TokenStream::from_iter([ident!(match), ident!(locale), braces!(match_body)])
}
