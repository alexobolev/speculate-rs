use proc_macro2::Span;
use syn::{braced, parse::{Parse, ParseStream, discouraged::Speculative}};
use unicode_xid::UnicodeXID;


mod kw {

    syn::custom_keyword!(describe);
    syn::custom_keyword!(context);

    syn::custom_keyword!(before);
    syn::custom_keyword!(after);

    syn::custom_keyword!(it);
    syn::custom_keyword!(test);

}

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
        let ahead_desc = input.fork();
        if let Ok(describe) = ahead_desc.parse::<Describe>() {
            input.advance_to(&ahead_desc);
            return Ok(Block::Describe(describe));
        }

        let ahead_it = input.fork();
        if let Ok(it) = ahead_it.parse::<It>() {
            input.advance_to(&ahead_it);
            return Ok(Block::It(it));
        }

        let ahead_item = input.fork();
        if let Ok(item) = ahead_item.parse::<syn::Item>() {
            input.advance_to(&ahead_item);
            return Ok(Block::Item(item));
        }

        Err(input.error("expected describe, it, or item block"))
    }
}

enum DescribeBlock {
    Regular(Block),
    Before(syn::Block),
    After(syn::Block),
}

impl Parse for DescribeBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ahead_before = input.fork();
        if let Ok(before) = ahead_before.parse::<DescribeBeforeParseBlock>() {
            input.advance_to(&ahead_before);
            return Ok(DescribeBlock::Before(before.0));
        }

        let ahead_after = input.fork();
        if let Ok(after) = ahead_after.parse::<DescribeAfterParseBlock>() {
            input.advance_to(&ahead_after);
            return Ok(DescribeBlock::After(after.0));
        }

        input.parse::<Block>().map(|block| DescribeBlock::Regular(block))
    }
}

struct DescribeBeforeParseBlock(syn::Block);
struct DescribeAfterParseBlock(syn::Block);

impl Parse for DescribeBeforeParseBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::before>()?;
        let block = input.parse::<syn::Block>()?;
        Ok(Self(block))
    }
}

impl Parse for DescribeAfterParseBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::after>()?;
        let block = input.parse::<syn::Block>()?;
        Ok(Self(block))
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
        let kw_lookahead = input.lookahead1();
        if kw_lookahead.peek(kw::describe) {
            input.parse::<kw::describe>()?;
        } else if kw_lookahead.peek(kw::context) {
            input.parse::<kw::context>()?;
        } else {
            return Err(kw_lookahead.error());
        }

        let name = input.parse::<syn::LitStr>()?;
        let mut root = {
            let root_content;
            braced!(root_content in input);
            root_content
        }.parse::<Root>()?;

        root.0.name = litstr_to_ident(&name);
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
        let attrs = syn::Attribute::parse_outer(input)?;
        
        let kw_lookahead = input.lookahead1();
        if kw_lookahead.peek(kw::it) {
            input.parse::<kw::it>()?;
        } else if kw_lookahead.peek(kw::test) {
            input.parse::<kw::test>()?;
        } else {
            return Err(kw_lookahead.error());
        }

        let name = input.parse::<syn::LitStr>()?;
        let block = input.parse::<syn::Block>()?;

        Ok(It {
            name: litstr_to_ident(&name),
            attributes: attrs,
            block
        })
    }
}

fn litstr_to_ident(l: &syn::LitStr) -> syn::Ident {
    let string = l.value();
    let mut id = String::with_capacity(string.len());

    if string.is_empty() {
        return syn::Ident::new("_", l.span());
    }

    let mut chars = string.chars();
    let mut added_underscore = false;

    let first_ch = chars.next().unwrap();

    if !UnicodeXID::is_xid_start(first_ch) {
        id.push('_');

        if UnicodeXID::is_xid_continue(first_ch) {
            id.push(first_ch);
        } else {
            added_underscore = true;
        }
    } else {
        id.push(first_ch);
    }

    for ch in chars {
        if UnicodeXID::is_xid_continue(ch) {
            id.push(ch);
            added_underscore = false;
        } else if !added_underscore {
            id.push('_');
            added_underscore = true;
        }
    }

    if id.as_bytes()[id.len() - 1] == b'_' {
        id.pop();
    }

    syn::Ident::new(&id, l.span())
}
