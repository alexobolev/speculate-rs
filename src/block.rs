use std::marker::PhantomData;
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use crate::extension::{ParseExt, ToIdentExt};


/// Keywords used in test specs.
mod kw {
    syn::custom_keyword!(describe);
    syn::custom_keyword!(context);
    syn::custom_keyword!(before);
    syn::custom_keyword!(after);
    syn::custom_keyword!(it);
    syn::custom_keyword!(test);
}

/// Parse one of several possible keywords.
#[allow(unused_macros)]
macro_rules! parse_keyword_alt {
    ( $input:ident, $( $kw:path ),+ ) => {
        {
            let __ahead__ = $input.lookahead1();
            if false {
                Err(__ahead__.error())
            }
            $(
                else if __ahead__.peek($kw) {
                    match $input.parse::<$kw>() {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e)
                    }
                }
            )+
            else {
                Err(__ahead__.error())
            }
        }
    };
}


#[derive(Clone)]
pub struct Root(pub(crate) Describe);

impl Parse for Root {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut befores = vec![];
        let mut afters = vec![];
        let mut regulars = vec![];

        while !input.cursor().eof() {
            match input.parse::<DescribeBlock>()? {
                DescribeBlock::Regular(block) => regulars.push(block),
                DescribeBlock::Before(before) => befores.push(before),
                DescribeBlock::After(after) => afters.push(after),
            }
        }

        Ok(Root(Describe {
            name: syn::Ident::new("speculate", Span::call_site()),
            before: befores, after: afters, blocks: regulars
        }))
    }
}

#[derive(Clone)]
pub enum Block {
    Describe(Describe),
    It(It),
    Item(syn::Item),
}

impl Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(describe) = input.parse_speculative::<Describe>() {
            return Ok(Block::Describe(describe));
        }

        if let Ok(it) = input.parse_speculative::<It>() {
            return Ok(Block::It(it));
        }

        if let Ok(item) = input.parse_speculative::<syn::Item>() {
            return Ok(Block::Item(item));
        }

        Err(input.error("expected describe, it, or item block"))
    }
}

#[derive(Clone)]
enum DescribeBlock {
    Regular(Block),
    Before(syn::Block),
    After(syn::Block),
}

impl Parse for DescribeBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(before) = input.parse_speculative::<PrefixedBlock<kw::before>>() {
            return Ok(DescribeBlock::Before(before.0));
        }

        if let Ok(after) = input.parse_speculative::<PrefixedBlock<kw::after>>() {
            return Ok(DescribeBlock::After(after.0));
        }

        if let Ok(block) = input.parse_speculative::<Block>() {
            return Ok(DescribeBlock::Regular(block));
        }

        Err(input.error("expected before, after, or regular block"))
    }
}

#[derive(Clone)]
struct PrefixedBlock<T : Parse>(syn::Block, PhantomData<T>);
impl<T> Parse for PrefixedBlock<T> where T : Parse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<T>()?;
        let block = input.parse::<syn::Block>()?;
        Ok(Self(block, PhantomData))
    }
}

#[derive(Clone)]
pub struct Describe {
    pub name: syn::Ident,
    pub before: Vec<syn::Block>,
    pub after: Vec<syn::Block>,
    pub blocks: Vec<Block>,
}

impl Parse for Describe {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        parse_keyword_alt!(input, kw::describe, kw::context)?;
        let name = input.parse::<syn::LitStr>()?;
        let mut root = input.parse_braced::<Root>()?;
        root.0.name = name.to_ident();
        Ok(root.0)
    }
}

#[derive(Clone)]
pub struct It {
    pub name: syn::Ident,
    pub attributes: Vec<syn::Attribute>,
    pub block: syn::Block,
}

impl Parse for It {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = syn::Attribute::parse_outer(input)?;
        parse_keyword_alt!(input, kw::it, kw::test)?;
        let name = input.parse::<syn::LitStr>()?;
        let block = input.parse::<syn::Block>()?;
        Ok(It { name: name.to_ident(), attributes, block })
    }
}
