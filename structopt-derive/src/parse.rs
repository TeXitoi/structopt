use std::iter::FromIterator;

use proc_macro_error::{span_error, ResultExt};
use syn::{
    self, parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Expr, ExprLit, Ident, Lit, LitBool, LitStr, Token,
};

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
    // single-identifier attributes
    Short(Ident),
    Long(Ident),
    Flatten(Ident),
    Subcommand(Ident),
    Skip(Ident),
    NoVersion(Ident),

    // ident [= "string literal"]
    About(Ident, Option<LitStr>),
    Author(Ident, Option<LitStr>),

    // ident = "string literal"
    Version(Ident, LitStr),
    RenameAll(Ident, LitStr),
    NameLitStr(Ident, LitStr),

    // parse(parser_kind [= parser_func])
    Parse(Ident, ParserSpec),

    // ident = arbitrary_expr
    NameExpr(Ident, Expr),

    // ident(arbitrary_expr,*)
    MethodCall(Ident, Vec<Expr>),
}

impl Parse for StructOptAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        use self::StructOptAttr::*;

        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        if input.peek(Token![=]) {
            // `name = value` attributes.
            let assign_token = input.parse::<Token![=]>()?; // skip '='

            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                let lit_str = lit.value();

                let check_empty_lit = |s| {
                    if lit_str.is_empty() {
                        span_error!(lit.span(), "`#[structopt({} = \"\") is deprecated in structopt 0.3, now it's default behavior", s);
                    }
                };

                match &*name_str.to_string() {
                    "rename_all" => Ok(RenameAll(name, lit)),

                    "version" => {
                        check_empty_lit("version");
                        Ok(Version(name, lit))
                    }

                    "author" => {
                        check_empty_lit("author");
                        Ok(Author(name, Some(lit)))
                    }

                    "about" => {
                        check_empty_lit("about");
                        Ok(About(name, Some(lit)))
                    }

                    _ => Ok(NameLitStr(name, lit)),
                }
            } else {
                match input.parse::<Expr>() {
                    Ok(expr) => Ok(NameExpr(name, expr)),
                    Err(_) => span_error! {
                        assign_token.span(),
                        "expected `string literal` or `expression` after `=`"
                    },
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

                "raw" => {
                    match nested.parse::<LitBool>() {
                        Ok(bool_token) => {
                            let expr = ExprLit { attrs: vec![], lit: Lit::Bool(bool_token) };
                            let expr = Expr::Lit(expr);
                            Ok(MethodCall(name, vec![expr]))
                        }

                        Err(_) => span_error!(name.span(),
                            "`#[structopt(raw(...))` attributes are deprecated in structopt 0.3, only `raw(true)` and `raw(false)` are allowed")
                    }
                }

                _ => {
                    let method_args: Punctuated<_, Token![,]> = nested.parse_terminated(Expr::parse)?;
                    Ok(MethodCall(name, Vec::from_iter(method_args)))
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
                "no_version" => Ok(NoVersion(name)),

                "about" => (Ok(About(name, None))),
                "author" => (Ok(Author(name, None))),

                "version" => {
                    span_error!(name.span(),
                    "#[structopt(version)] is invalid attribute, structopt 0.3 inherits version from Cargo.toml by default, no attribute needed")
                },

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
