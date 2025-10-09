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
macro_rules! str_lit {
    ($value:expr) => {
        TokenTree::Literal(Literal::from_str($value).unwrap())
    };
}
macro_rules! single_punct {
    (,) => {
        TokenTree::Punct(Punct::new(',', Spacing::Alone))
    };
    (!) => {
        TokenTree::Punct(Punct::new('!', Spacing::Alone))
    };
}
macro_rules! parentheses {
    ($stream:expr) => {
        TokenTree::Group(Group::new(Delimiter::Parenthesis, $stream))
    };
}
macro_rules! braces {
    ($stream:expr) => {
        TokenTree::Group(Group::new(Delimiter::Brace, $stream))
    };
}
macro_rules! translation_match_rule {
	( $(::$ident:ident)* => $($value:expr)* ) => {
		[
			$(
				TokenTree::Punct(Punct::new(':', Spacing::Joint)),
				TokenTree::Punct(Punct::new(':', Spacing::Alone)),
				ident!($ident),
			)*
			TokenTree::Punct(Punct::new('=', Spacing::Joint)),
			TokenTree::Punct(Punct::new('>', Spacing::Alone)),
			$($value,)*
			TokenTree::Punct(Punct::new(',', Spacing::Alone)),
		]
	};
}
macro_rules! source_match_rule {
    ( $($value:expr)* ) => {
        [
            ident!(_),
            TokenTree::Punct(Punct::new('=', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            $($value,)*
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
                "de" => {
                    translation_match_rule!(::raphael_data::Locale::DE => str_lit!(string_literal))
                }
                "fr" => {
                    translation_match_rule!(::raphael_data::Locale::FR => str_lit!(string_literal))
                }
                "ja" => {
                    translation_match_rule!(::raphael_data::Locale::JP => str_lit!(string_literal))
                }
                "ko" => {
                    translation_match_rule!(::raphael_data::Locale::KR => str_lit!(string_literal))
                }
                _ => unreachable!(),
            })
        })
        .chain([TokenStream::from_iter(source_match_rule!(
            main_arg_token_tree
        ))]);
    let match_body = TokenStream::from_iter(translation_token_streams);
    TokenStream::from_iter([ident!(match), ident!(locale), braces!(match_body)])
}

pub fn generate_format_macro_output_token_stream(
    translations: Vec<Translation>,
    format_arguments: proc_macro::token_stream::IntoIter,
    ctx: Context,
) -> TokenStream {
    let Context {
        main_arg_token_tree,
        ..
    } = ctx;

    let match_arms_token_streams = translations
        .iter()
        .map(|Translation(language_key, string)| {
            let format_string = &format!("\"{}\"", string);
            let format_body = TokenStream::from_iter(
                TokenStream::from(str_lit!(format_string))
                    .into_iter()
                    .chain(format_arguments.clone()),
            );
            TokenStream::from_iter(match *language_key {
                "de" => translation_match_rule!(::raphael_data::Locale::DE => ident!(format) single_punct!(!) parentheses!(format_body)),
                "fr" => translation_match_rule!(::raphael_data::Locale::FR => ident!(format) single_punct!(!) parentheses!(format_body)),
                "ja" => translation_match_rule!(::raphael_data::Locale::JP => ident!(format) single_punct!(!) parentheses!(format_body)),
                "ko" => translation_match_rule!(::raphael_data::Locale::KR => ident!(format) single_punct!(!) parentheses!(format_body)),
                _ => unreachable!(),
            })
        })
        .chain([TokenStream::from_iter({
            let format_body = TokenStream::from_iter(
                TokenStream::from(main_arg_token_tree)
                    .into_iter()
                    .chain(format_arguments.clone()),
            );
            source_match_rule!(ident!(format) single_punct!(!) parentheses!(format_body))
        })]);
    let match_body = TokenStream::from_iter(match_arms_token_streams);
    TokenStream::from_iter([ident!(match), ident!(locale), braces!(match_body)])
}
