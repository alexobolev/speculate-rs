use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use unicode_xid::UnicodeXID as uc;

pub(crate) trait ParseExt {
    fn parse_braced<T: Parse>(&self) -> syn::Result<T>;
    fn parse_speculative<T: Parse>(&self) -> syn::Result<T>;
}

impl ParseExt for ParseStream<'_> {
    fn parse_braced<T: Parse>(&self) -> syn::Result<T> {
        let content;
        syn::braced!(content in self);
        content.parse::<T>()
    }
    fn parse_speculative<T: Parse>(&self) -> syn::Result<T> {
        let ahead = self.fork();
        let parsed = ahead.parse::<T>();
        if parsed.is_ok() {
            self.advance_to(&ahead);
        }
        parsed
    }
}

pub(crate) trait ToIdentExt {
    fn to_ident(&self) -> syn::Ident;
}

impl ToIdentExt for syn::LitStr {
    fn to_ident(&self) -> syn::Ident {
        let lit_value = self.value();
        let mut ident_value = String::with_capacity(lit_value.len());

        if lit_value.is_empty() {
            return syn::Ident::new("_", self.span());
        }

        let mut lit_chars = lit_value.chars();
        let mut added_underscore = false;

        let first_ch = lit_chars.next().unwrap();
        if !uc::is_xid_start(first_ch) {
            ident_value.push('_');
            match uc::is_xid_continue(first_ch) {
                true => ident_value.push(first_ch),
                false => added_underscore = true
            }
        } else {
            ident_value.push(first_ch);
        }

        for ch in lit_chars {
            if uc::is_xid_continue(ch) {
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

        syn::Ident::new(&ident_value, self.span())
    }
}
