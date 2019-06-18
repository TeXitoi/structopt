use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{self, Attribute, Expr, Ident, LitStr};

pub struct StructOptAttributes {
    pub paren_token: syn::token::Paren,
    pub attrs: Punctuated<StructOptAttr, Token![,]>,
}

impl Parse for StructOptAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(StructOptAttributes {
            paren_token: parenthesized!(content in input),
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
    MethodCall(Ident, Punctuated<Expr, Token![,]>),
}

impl Parse for StructOptAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use self::StructOptAttr::*;

        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        if input.peek(Token![=]) {
            // `name = value` attributes.
            input.parse::<Token![=]>()?; // skip '='

            match name_str.as_ref() {
                "rename_all" => {
                    let casing_lit: LitStr = input.parse()?;
                    Ok(RenameAll(casing_lit))
                }

                _ => {
                    if input.peek(LitStr) {
                        let lit: LitStr = input.parse()?;
                        Ok(NameLitStr(name, lit))
                    } else {
                        let expr: Expr = input.parse()?;
                        Ok(NameExpr(name, expr))
                    }
                }
            }
        } else if input.peek(syn::token::Paren) {
            // `name(...)` attributes.
            let nested;
            parenthesized!(nested in input);

            match name_str.as_ref() {
                "parse" => {
                    let parser_specs: Punctuated<ParserSpec, Token![,]> =
                        nested.parse_terminated(ParserSpec::parse)?;

                    if parser_specs.len() == 1 {
                        Ok(Parse(parser_specs[0].clone()))
                    } else {
                        Err(input.error("parse must have one argument"))
                    }
                }

                "raw" => {
                    let raw_entries = nested.parse_terminated(RawEntry::parse)?;
                    Ok(Raw(raw_entries))
                }

                _ => {
                    let method_args = nested.parse_terminated(Expr::parse)?;
                    Ok(MethodCall(name, method_args))
                }
            }
        } else {
            // Attributes represented with a sole identifier.
            match name_str.as_ref() {
                "long" => Ok(Long),
                "short" => Ok(Short),
                "flatten" => Ok(Flatten),
                "subcommand" => Ok(Subcommand),
                _ => {
                    let msg = format!("unexpected attribute: {}", name_str);
                    Err(input.error(&msg))
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ParserSpec {
    pub kind: Ident,
    pub eq_token: Option<Token![=]>,
    pub parse_func: Option<Expr>,
}

impl Parse for ParserSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kind = input.parse()?;
        let eq_token = input.parse()?;
        let parse_func = match eq_token {
            None => None,
            Some(_) => Some(input.parse()?),
        };
        Ok(ParserSpec {
            kind,
            eq_token,
            parse_func,
        })
    }
}

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
    let mut s_opt_attrs: Vec<StructOptAttr> = vec![];
    for attr in all_attrs {
        let path = &attr.path;
        match quote!(#path).to_string().as_ref() {
            "structopt" => {
                let tokens = attr.tts.clone();
                let error_msg = format!("cannot parse structopt attributes: {}", tokens);
                let so_attrs: StructOptAttributes = syn::parse2(tokens).expect(&error_msg);
                s_opt_attrs.extend(so_attrs.attrs);
            }
            _ => {}
        }
    }
    s_opt_attrs
}
