use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{Attribute, Expr, Ident, LitStr, Token};

pub struct StructOptAttributes {
    #[allow(dead_code)]
    pub paren_token: Paren,
    pub attrs: Punctuated<StructOptAttr, Token![,]>,
}

impl Parse for StructOptAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(StructOptAttributes {
            paren_token: syn::parenthesized!(content in input),
            attrs: content.parse_terminated(StructOptAttr::parse)?,
        })
    }
}

pub enum StructOptAttr {
    Short,
    Long,
    Flatten,
    Subcommand,
    Parse(ParserSpec),
    RenameAll(LitStr),
    NameLitStr(Ident, LitStr),
    NameExpr(Ident, Expr),
    Raw(Punctuated<RawEntry, Token![,]>),
}

impl Parse for StructOptAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use self::StructOptAttr::*;

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
                        return Ok(RenameAll(casing_lit));
                    }

                    _ => {
                        if input.peek(LitStr) {
                            let lit: LitStr = input.parse()?;
                            return Ok(NameLitStr(name, lit));
                        } else {
                            let expr: Expr = input.parse()?;
                            return Ok(NameExpr(name, expr));
                        }
                    }
                }
            } else if input.peek(Paren) {
                match name_str.as_ref() {
                    "parse" => {
                        let nested;
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
                        return Ok(Raw(raw_entries));
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

pub fn parse_structopt_attributes(all_attrs: &[Attribute]) -> Vec<StructOptAttr> {
    let mut v: Vec<StructOptAttr> = vec![];
    for attr in all_attrs {
        let path = &attr.path;
        match quote!(#path).to_string().as_ref() {
            "structopt" => {
                let tokens = attr.tts.clone();
                // FIXME: propagate errors further?
                let pa: StructOptAttributes = syn::parse2(tokens).expect("hoho");
                v.extend(pa.attrs);
            }
            _ => {}
        }
    }
    v
}
