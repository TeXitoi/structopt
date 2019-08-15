use proc_macro_error::{span_error, ResultExt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{self, parenthesized, parse2, Attribute, Expr, Ident, LitStr, Token};

pub struct StructOptAttributes {
    pub paren_token: syn::token::Paren,
    pub attrs: Punctuated<StructOptAttr, Token![,]>,
}

impl Parse for StructOptAttributes {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;

        Ok(StructOptAttributes {
            paren_token: parenthesized!(content in input),
            attrs: content.parse_terminated(StructOptAttr::parse)?,
        })
    }
}

pub enum StructOptAttr {
    Short(Ident),
    Long(Ident),
    Flatten(Ident),
    Subcommand(Ident),
    Skip(Ident),
    Parse(Ident, ParserSpec),
    RenameAll(Ident, LitStr),
    NameLitStr(Ident, LitStr),
    NameExpr(Ident, Expr),
    MethodCall(Ident, Punctuated<Expr, Token![,]>),
}

impl Parse for StructOptAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        use self::StructOptAttr::*;

        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        if input.peek(Token![=]) {
            // `name = value` attributes.
            input.parse::<Token![=]>()?; // skip '='

            match name_str.as_ref() {
                "rename_all" => {
                    let casing_lit: LitStr = input.parse()?;
                    Ok(RenameAll(name, casing_lit))
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
                        Ok(Parse(name, parser_specs[0].clone()))
                    } else {
                        span_error!(name.span(), "parse must have exactly one argument")
                    }
                }

                _ => {
                    let method_args = nested.parse_terminated(Expr::parse)?;
                    Ok(MethodCall(name, method_args))
                }
            }
        } else {
            // Attributes represented with a sole identifier.
            match name_str.as_ref() {
                "long" => Ok(Long(name)),
                "short" => Ok(Short(name)),
                "flatten" => Ok(Flatten(name)),
                "subcommand" => Ok(Subcommand(name)),
                "skip" => Ok(Skip(name)),
                _ => span_error!(name.span(), "unexpected attribute: {}", name_str),
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
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let kind = input
            .parse()
            .map_err(|_| input.error("parser specification must start with identifier"))?;
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

pub fn parse_structopt_attributes(all_attrs: &[Attribute]) -> Vec<StructOptAttr> {
    all_attrs
        .iter()
        .filter(|attr| attr.path.is_ident("structopt"))
        .flat_map(|attr| {
            let attrs: StructOptAttributes = parse2(attr.tokens.clone())
                .map_err(|e| match &*e.to_string() {
                    // this error message is misleading and points to Span::call_site()
                    // so we patch it with something meaningful
                    "unexpected end of input, expected parentheses" => {
                        let span = attr.path.span();
                        let patch_msg = "expected parentheses after `structopt`";
                        syn::Error::new(span, patch_msg)
                    }
                    _ => e,
                })
                .unwrap_or_exit();
            attrs.attrs
        })
        .collect()
}
