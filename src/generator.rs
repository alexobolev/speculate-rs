use crate::block::{Block, Describe, It};
use proc_macro2::{Ident, TokenStream};
use quote::{quote_spanned, ToTokens};

pub trait Generate {
    fn generate(self, outer: Option<&Describe>) -> TokenStream;
}

impl Generate for Block {
    fn generate(self, outer: Option<&Describe>) -> TokenStream {
        match self {
            Block::Describe(describe) => describe.generate(outer),
            Block::It(it) => it.generate(outer),
            Block::Item(item) => item.into_token_stream(),
        }
    }
}

impl Generate for Describe {
    fn generate(mut self, outer: Option<&Describe>) -> TokenStream {
        if let Some(outer) = outer {
            self.before = outer.before.iter()
                .chain(self.before.iter())
                .cloned().collect();

            self.after = self.after.iter()
                .chain(outer.after.iter())
                .cloned().collect();
        }

        let name = &self.name;
        let items = self.blocks.iter()
            .map(|block| block.clone().generate(Some(&self)))
            .collect::<Vec<_>>();

        quote_spanned!(name.span() =>
            mod #name {
                #[allow(unused_imports)]
                use super::*;

                #(#items)*
            }
        )
    }
}

impl Generate for It {
    fn generate(self, outer: Option<&Describe>) -> TokenStream {
        let blocks = match outer {
            Some(outer) => {
                outer.before.iter()
                    .chain(Some(self.block).iter())
                    .chain(outer.after.iter())
                    .cloned().collect()
            },
            None => vec![self.block],
        };

        let name = Ident::new(&format!("test_{}", self.name), self.name.span());
        let stmts = blocks.into_iter().flat_map(|block| block.stmts);
        let attributes = self.attributes;

        quote_spanned!(name.span() =>
            #[test]
            #(#attributes)*
            fn #name() {
                #(#stmts)*
            }
        )
    }
}
