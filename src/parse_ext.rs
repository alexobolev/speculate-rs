use syn::{braced, Ident, LitStr, Result};
use syn::parse::{Parse, ParseStream, discouraged::Speculative};
use unicode_xid::UnicodeXID as uni;

pub(crate) trait ParseExt {
    fn parse_braced<T: Parse>(&self) -> Result<T>;
    fn parse_speculative<T: Parse>(&self) -> Result<T>;
}

impl ParseExt for ParseStream<'_> {
    fn parse_braced<T: Parse>(&self) -> Result<T> {
        let content;
        braced!(content in self);
        content.parse::<T>()
    }
    fn parse_speculative<T: Parse>(&self) -> Result<T> {
        let ahead = self.fork();
        let parsed = ahead.parse::<T>();
        if parsed.is_ok() {
            self.advance_to(&ahead);
        }
        parsed
    }
}

pub(crate) trait ToIdentExt {
    fn to_ident(&self) -> Ident;
}

impl ToIdentExt for LitStr {
    fn to_ident(&self) -> Ident {
        let lit_value = self.value();
        let mut ident_value = String::with_capacity(lit_value.len());

        if lit_value.is_empty() {
            return Ident::new("_", self.span());
        }

        let mut lit_chars = lit_value.chars();
        let mut added_underscore = false;

        let first_ch = lit_chars.next().unwrap();
        if !uni::is_xid_start(first_ch) {
            ident_value.push('_');
            match uni::is_xid_continue(first_ch) {
                true => ident_value.push(first_ch),
                false => added_underscore = true
            }
        } else {
            ident_value.push(first_ch);
        }

        for ch in lit_chars {
            if uni::is_xid_continue(ch) {
                ident_value.push(ch);
                added_underscore = false;
                continue;
            }

            if !added_underscore {
                ident_value.push('_');
                added_underscore = true;
            }
        }

        if ident_value.ends_with('_') {
            ident_value.pop();
        }

        Ident::new(&ident_value, self.span())
    }
}


#[allow(dead_code)]
#[deprecated(since="0.2.0", note="please use 'ToIdentExt::to_ident' instead")]
pub(crate) fn litstr_to_ident(l: &syn::LitStr) -> syn::Ident {
    let string = l.value();
    let mut id = String::with_capacity(string.len());

    if string.is_empty() {
        return syn::Ident::new("_", l.span());
    }

    let mut chars = string.chars();
    let mut added_underscore = false;

    let first_ch = chars.next().unwrap();

    if !uni::is_xid_start(first_ch) {
        id.push('_');

        if uni::is_xid_continue(first_ch) {
            id.push(first_ch);
        } else {
            added_underscore = true;
        }
    } else {
        id.push(first_ch);
    }

    for ch in chars {
        if uni::is_xid_continue(ch) {
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
