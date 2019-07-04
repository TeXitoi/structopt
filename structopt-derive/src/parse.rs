use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{self, parenthesized, Attribute, Expr, Ident, LitStr, Token};

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
    Short,
    Long,
    Flatten,
    Subcommand,
    Skip,
    Parse(ParserSpec),
    RenameAll(LitStr),
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
                        // Use `Error::new` instead of `input.error(...)`
                        // because when `input.error` tries to locate current span
                        // and sees that there is no tokens left to parse it adds
                        // 'unexpected end of input` to the error message, which is
                        // undesirable and misleading.
                        Err(syn::Error::new(
                            nested.cursor().span(),
                            "parse must have exactly one argument",
                        ))
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
                "long" => Ok(Long),
                "short" => Ok(Short),
                "flatten" => Ok(Flatten),
                "subcommand" => Ok(Subcommand),
                "skip" => Ok(Skip),
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
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let err_msg = "unknown value parser specification";
        let kind = input.parse().map_err(|_| input.error(err_msg))?;
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
    let mut s_opt_attrs: Vec<StructOptAttr> = vec![];
    for attr in all_attrs {
        let path = &attr.path;
        if let "structopt" = quote!(#path).to_string().as_ref() {
            let tokens = attr.tts.clone();
            let is_empty = tokens.is_empty();
            let so_attrs: StructOptAttributes = syn::parse2(tokens).unwrap_or_else(|err| {
                let tokens_str = if is_empty {
                    String::new()
                } else {
                    format!("problematic tokens: {}", &attr.tts)
                };
                panic!("{}, {}", err.to_string(), tokens_str)
            });
            s_opt_attrs.extend(so_attrs.attrs);
        }
    }
    s_opt_attrs
}
