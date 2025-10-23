use std::str::FromStr;

use proc_macro2::Literal;
use quote::quote;
use raphael_data::Locale;

use crate::StringLiteralDetails;

#[derive(Debug)]
pub struct Translation {
    locale: Locale,
    literal: Literal,
}

impl Translation {
    pub fn new(
        language_key: &str,
        translation: &str,
        string_literal_details: &StringLiteralDetails,
    ) -> Self {
        let locale = match language_key {
            "en" => unimplemented!(
                "The entry in the translation TOML is only for reference, the EN string literal argument passed to the macro is used directly."
            ),
            "de" => Locale::DE,
            "fr" => Locale::FR,
            "ja" => Locale::JP,
            "chs" => Locale::CN,
            "ko" => Locale::KR,
            _ => panic!("Unsupported language key!"),
        };

        let StringLiteralDetails { intro, outro, .. } = string_literal_details;
        let literal = Literal::from_str(&format!("{intro}{translation}{outro}")).unwrap();

        Self { locale, literal }
    }

    pub fn split(self) -> (LocaleTokens, Literal) {
        (LocaleTokens::from(self.locale), self.literal)
    }
}

pub struct LocaleTokens(Locale);

impl From<Locale> for LocaleTokens {
    fn from(value: Locale) -> Self {
        Self(value)
    }
}

impl quote::ToTokens for LocaleTokens {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.0 {
            Locale::EN => unimplemented!(
                "The entry in the translation TOML is only for reference, the EN string literal argument passed to the macro is used directly."
            ),
            Locale::DE => quote! { ::raphael_data::Locale::DE },
            Locale::FR => quote! { ::raphael_data::Locale::FR },
            Locale::JP => quote! { ::raphael_data::Locale::JP },
            Locale::CN => quote! { ::raphael_data::Locale::CN },
            Locale::KR => quote! { ::raphael_data::Locale::KR },
        }
        .to_tokens(tokens);
    }
}

impl quote::ToTokens for Translation {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let locale_tokens = LocaleTokens::from(self.locale);
        let literal = &self.literal;
        quote! { #locale_tokens => #literal }.to_tokens(tokens);
    }
}
