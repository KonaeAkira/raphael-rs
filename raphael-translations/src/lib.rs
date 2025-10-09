use base64::prelude::*;
use proc_macro::{TokenStream, TokenTree};

mod data;
use data::get_translations;

mod output;
use output::*;

mod util;

fn generate_toml_entry_hash(text: &str, main_arg_token_tree: &TokenTree) -> u128 {
    use rustc_stable_hash::FromStableHash;
    use rustc_stable_hash::hashers::{SipHasher128Hash, StableSipHasher128};
    use std::hash::Hasher;
    struct HashU128(u128);
    impl FromStableHash for HashU128 {
        type Hash = SipHasher128Hash;

        fn from(SipHasher128Hash(hash): SipHasher128Hash) -> Self {
            Self(u128::from(hash[0]) | u128::from(hash[1]).unbounded_shl(64))
        }
    }

    let mut hasher = StableSipHasher128::new();
    for byte in text.as_bytes() {
        hasher.write_u8(*byte);
    }

    if let TokenTree::Ident(_) = main_arg_token_tree {
        // This assumes that the identifier is only used once / stable, i.e., the same accross the file
        for byte in main_arg_token_tree.span().file().as_bytes() {
            hasher.write_u8(*byte);
        }
    }

    hasher.finish::<HashU128>().0
}

#[derive(Debug)]
pub(crate) struct Context {
    pub hash_base64: String,
    pub main_arg_token_tree: TokenTree,
    pub text: String,
}

impl Context {
    pub fn new(main_arg_token_tree: TokenTree) -> Self {
        match &main_arg_token_tree {
            TokenTree::Ident(_) | TokenTree::Literal(_) => {}
            _ => panic!("Unsuported input! Must be either an identifier or a literal."),
        }
        let text = format!("{}", main_arg_token_tree);
        let hash = generate_toml_entry_hash(&text, &main_arg_token_tree);
        let hash_base64 = BASE64_STANDARD.encode(hash.to_le_bytes());

        Self {
            hash_base64,
            main_arg_token_tree,
            text,
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
