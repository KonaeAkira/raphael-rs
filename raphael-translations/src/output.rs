use std::str::FromStr;

use crate::{Context, data::Translation};

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

macro_rules! ident {
    ($token:ident) => {
        TokenTree::Ident(Ident::new(stringify!($token), Span::call_site()))
    };
    (_) => {
        TokenTree::Ident(Ident::new(stringify!(_), Span::call_site()))
    };
}
macro_rules! braces {
    ($stream:ident) => {
        TokenTree::Group(Group::new(Delimiter::Brace, $stream))
    };
}
macro_rules! translation_match_rule {
	( $(::$ident:ident)* => $value:expr) => {
		[
			$(
				TokenTree::Punct(Punct::new(':', Spacing::Joint)),
				TokenTree::Punct(Punct::new(':', Spacing::Alone)),
				ident!($ident),
			)*
			TokenTree::Punct(Punct::new('=', Spacing::Joint)),
			TokenTree::Punct(Punct::new('>', Spacing::Alone)),
			TokenTree::Literal(Literal::from_str($value).unwrap()),
			TokenTree::Punct(Punct::new(',', Spacing::Alone)),
		]
	};
}
macro_rules! source_match_rule {
    ($value:expr) => {
        [
            ident!(_),
            TokenTree::Punct(Punct::new('=', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            $value,
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
        ]
    };
}

pub fn generate_output_token_stream(translations: Vec<Translation>, ctx: Context) -> TokenStream {
    let Context {
        main_arg_token_tree,
        ..
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
        .chain([TokenStream::from_iter(source_match_rule!(
            main_arg_token_tree
        ))]);
    let match_body = TokenStream::from_iter(translation_token_streams);
    TokenStream::from_iter([ident!(match), ident!(locale), braces!(match_body)])
}
