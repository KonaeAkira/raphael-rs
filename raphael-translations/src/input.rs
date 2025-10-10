use proc_macro::TokenTree;

#[derive(Debug)]
pub(crate) struct MainArgument {
    pub contents: ParsedMainArgumentContents,
    pub token_tree: TokenTree,
}

impl MainArgument {
    pub fn generate_toml_entry_hash(&self) -> u128 {
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
        for byte in self.contents.characteristic_string().as_bytes() {
            hasher.write_u8(*byte);
        }

        if let ParsedMainArgumentContents::Identifier { .. } = self.contents {
            // This assumes that the identifier is only used once / stable, i.e., the same accross the file
            for byte in self.token_tree.span().file().as_bytes() {
                hasher.write_u8(*byte);
            }
        }

        hasher.finish::<HashU128>().0
    }

    #[cfg(feature = "update-toml")]
    pub fn source_location_string(&self) -> String {
        let span = self.token_tree.span();
        format!("{}:{}:{}", span.file(), span.line(), span.column())
    }
}

#[derive(Debug)]
pub(crate) enum ParsedMainArgumentContents {
    Identifier {
        name: String,
    },
    StringLiteral {
        intro: String,
        body: String,
        outro: String,
    },
}

impl ParsedMainArgumentContents {
    pub fn characteristic_string(&self) -> &str {
        match self {
            Self::Identifier { name } => name,
            Self::StringLiteral { body, .. } => body,
        }
    }
}

impl From<&TokenTree> for ParsedMainArgumentContents {
    fn from(token_tree: &TokenTree) -> Self {
        match token_tree {
            TokenTree::Ident(ident) => {
                return Self::Identifier {
                    name: ident.to_string(),
                };
            }
            TokenTree::Literal(literal) => {
                let literal_text = literal.to_string();
                if let Some((intro, rest)) = literal_text.split_once('"')
                    && let Some((body, outro)) = rest.rsplit_once('"')
                {
                    return Self::StringLiteral {
                        intro: format!("{intro}\""),
                        body: body.to_string(),
                        outro: format!("\"{outro}"),
                    };
                }
            }
            _ => {}
        }
        panic!("Unsuported input! Must be either an identifier or a (raw) string literal.");
    }
}

impl From<TokenTree> for MainArgument {
    fn from(token_tree: TokenTree) -> Self {
        Self {
            contents: ParsedMainArgumentContents::from(&token_tree),
            token_tree,
        }
    }
}
