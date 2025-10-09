use base64::prelude::*;
use proc_macro::{Span, TokenStream, TokenTree};

mod data;
use data::get_translations;

mod output;
use output::generate_output_token_stream;

mod util;

pub(crate) enum Occurance {
    Identifier(Span),
    Literal(Span),
}

impl From<&TokenTree> for Occurance {
    fn from(token_tree: &TokenTree) -> Self {
        match token_tree {
            proc_macro::TokenTree::Ident(ident) => Self::Identifier(ident.span()),
            proc_macro::TokenTree::Literal(literal) => Self::Literal(literal.span()),
            _ => panic!("Unsupported input! Must be identifier or literal."),
        }
    }
}

impl Occurance {
    #[cfg(feature = "update-toml")]
    fn source_location(&self) -> String {
        let span = match self {
            Occurance::Identifier(span) => span,
            Occurance::Literal(span) => span,
        };

        format!("{}:{}:{}", span.file(), span.line(), span.column())
    }
}

fn generate_toml_entry_hash(text: &str, occurance: &Occurance) -> u128 {
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

    match occurance {
        Occurance::Identifier(span) => {
            // This assumes that the identifier is only used once / stable, i.e., the same accross the file
            for byte in span.file().as_bytes() {
                hasher.write_u8(*byte);
            }
        }
        Occurance::Literal(_span) => {}
    }

    hasher.finish::<HashU128>().0
}

pub(crate) struct Context {
    pub hash_base64: String,
    pub text: String,
    pub occurance: crate::Occurance,
}

impl Context {
    pub fn new(token_tree: &TokenTree) -> Self {
        let text = format!("{}", token_tree);
        let occurance = Occurance::from(token_tree);
        let hash = generate_toml_entry_hash(&text, &occurance);
        let hash_base64 = BASE64_STANDARD.encode(hash.to_le_bytes());

        Self {
            hash_base64,
            text,
            occurance,
        }
    }
}

#[proc_macro]
pub fn t(input: TokenStream) -> TokenStream {
    let input = input.into_iter().collect::<Vec<_>>();
    assert_eq!(
        input.len(),
        1,
        "Only a single argument to the macro is supported!"
    );

    let token_tree = input.first().unwrap();
    let ctx = Context::new(token_tree);

    let translations = get_translations(&ctx);

    generate_output_token_stream(translations, &ctx)
}
