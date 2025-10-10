use base64::prelude::*;
use proc_macro::{TokenStream, TokenTree};

mod input;
use input::*;

mod data;
use data::get_translations;

mod output;
use output::*;

mod util;

#[derive(Debug)]
pub(crate) struct Context {
    pub hash_base64: String,
    pub main_arg: MainArgument,
}

impl Context {
    pub fn new(main_arg_token_tree: TokenTree) -> Self {
        let main_arg = MainArgument::from(main_arg_token_tree);
        let hash = main_arg.generate_toml_entry_hash();
        let hash_base64 = BASE64_STANDARD.encode(hash.to_le_bytes());

        Self {
            hash_base64,
            main_arg,
        }
    }
}

#[proc_macro]
pub fn t(input: TokenStream) -> TokenStream {
    let mut input_iter = input.into_iter();
    assert_eq!(
        // This assumes the size hint is correct
        input_iter.size_hint(),
        (1, Some(1)),
        "Only a single argument to the macro is supported!"
    );

    let token_tree = input_iter.next().unwrap();
    let ctx = Context::new(token_tree);

    let translations = get_translations(&ctx);

    generate_output_token_stream(translations, ctx)
}

#[proc_macro]
pub fn t_format(input: TokenStream) -> TokenStream {
    let mut input_iter = input.into_iter();
    assert_ne!(
        // This assumes the size hint is correct
        input_iter.size_hint().0,
        0,
        "At least a format argument is required!"
    );

    let (format_token_tree, format_arguments) = (input_iter.next().unwrap(), input_iter);
    let ctx = Context::new(format_token_tree);

    let translations = get_translations(&ctx);

    generate_format_macro_output_token_stream(translations, format_arguments, ctx)
}
