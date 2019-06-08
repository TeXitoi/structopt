// TODO: improve those syn imports, group better?
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{Attribute, Ident, LitStr, Token};

use crate::attrs::*;

// FIXME: use a better name
pub enum ParsedAttr {
    Short,
    Long,
    Flatten,
    Subcommand,
    Parse(ParserSpec),
    RenameAll(CasingStyle), // FIXME: return more low-level stuff, and convert to Casing style later?
    NameLitStr(String, syn::LitStr),
    NameExpr(String, syn::Expr), // use Ident instead of String?
    OldRaw(Punctuated<RawEntry, Token![,]>),
}

// FIXME: use a better name to distinguish from Rust attributes...
pub struct ParsedAttributes {
    #[allow(dead_code)]
    pub paren_token: Paren,
    pub attrs: Punctuated<ParsedAttr, Token![,]>,
}

#[derive(Clone)]
pub struct ParserSpec {
    pub kind: Ident,
    pub eq_token: Option<Token![=]>,
    pub parse_func: Option<syn::LitStr>,
}

impl Parse for ParserSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ParserSpec {
            kind: input.parse()?,
            eq_token: input.parse()?,
            parse_func: input.parse()?,
        })
    }
}

#[derive(Clone)]
pub struct RawEntry {
    pub name: Ident,
    pub eq_token: Token![=],
    pub value: LitStr,
}

impl Parse for RawEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RawEntry {
            name: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for ParsedAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(ParsedAttributes {
            paren_token: syn::parenthesized!(content in input),
            attrs: content.parse_terminated(ParsedAttr::parse)?,
        })
    }
}

impl Parse for ParsedAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use self::ParsedAttr::*;

        // FIXME: do we really need lookahead here?
        // Should just expect identifier.
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let name: Ident = input.parse()?;
            let name_str = name.to_string();

            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?; // skip '='

                match name_str.as_ref() {
                    "rename_all" => {
                        let casing_lit: syn::LitStr = input.parse()?;
                        let casing: CasingStyle = {
                            ::std::str::FromStr::from_str(&casing_lit.value())
                                .unwrap_or_else(|error| panic!("{}", error))
                        };
                        return Ok(RenameAll(casing));
                    }

                    _ => {
                        if input.peek(LitStr) {
                            let lit: LitStr = input.parse()?;
                            return Ok(NameLitStr(name.to_string(), lit));
                        } else {
                            let value: syn::Expr = input.parse()?;
                            return Ok(NameExpr(name.to_string(), value));
                        }
                    }
                }
            } else if input.peek(Paren) {
                match name_str.as_ref() {
                    "parse" => {
                        let nested;
                        // FIXME: just skip parens? will it work?
                        syn::parenthesized!(nested in input);

                        let parser_specs: Punctuated<ParserSpec, Token![,]> =
                            nested.parse_terminated(ParserSpec::parse)?;

                        if parser_specs.len() != 1 {
                            // FIXME: use parsing error, not panic
                            panic!("parse should have one argument");
                        }

                        return Ok(Parse(parser_specs[0].clone()));
                    }

                    "raw" => {
                        let nested;
                        syn::parenthesized!(nested in input);
                        let raw_entries = nested.parse_terminated(RawEntry::parse)?;
                        return Ok(OldRaw(raw_entries));
                    }

                    _ => {
                        // FIXME: throw error
                        return Ok(Long);
                    }
                }
            } else {
                // Treat it as just a word.
                match name_str.as_ref() {
                    "long" => return Ok(Long),
                    "short" => return Ok(Short),
                    "flatten" => return Ok(Flatten),
                    "subcommand" => return Ok(Subcommand),
                    _ => {} // FIXME: throw an error
                }
            }
        } else {
            return Err(lookahead.error());
        }

        // FIXME: this error assumes that it's end of input,
        // use a more appropriate error message
        Err(input.error("expected structopt attribute"))
    }
}

pub fn parse_attributes(all_attrs: &[Attribute]) -> Vec<ParsedAttr> {
    let mut v: Vec<ParsedAttr> = vec![];
    for attr in all_attrs {
        let path = &attr.path;
        match quote!(#path).to_string().as_ref() {
            "structopt" => {
                let tokens = attr.tts.clone();
                // FIXME: propagate errors further?
                let pa: ParsedAttributes = syn::parse2(tokens).expect("hoho");
                v.extend(pa.attrs);
            }
            _ => {}
        }
    }
    v
}
