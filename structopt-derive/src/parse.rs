use std::iter::FromIterator;

use quote::ToTokens;
use syn::{
    self, parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, ExprLit, Ident, Lit, LitBool, LitStr, Token,
};

pub enum StructOptAttr {
    // single-identifier attributes
    Short(Ident),
    Long(Ident),
    Env(Ident),
    Flatten(Ident),
    Subcommand(Ident),
    ExternalSubcommand(Ident),
    NoVersion(Ident),
    VerbatimDocComment(Ident),

    // ident [= "string literal"]
    About(Ident, Option<LitStr>),
    Author(Ident, Option<LitStr>),
    DefaultValue(Ident, Option<LitStr>),

    // ident = "string literal"
    Version(Ident, LitStr),
    RenameAllEnv(Ident, LitStr),
    RenameAll(Ident, LitStr),
    NameLitStr(Ident, LitStr),

    // parse(parser_kind [= parser_func])
    Parse(Ident, ParserSpec),

    // ident [= arbitrary_expr]
    Skip(Ident, Option<Expr>),

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
            let _assign_token = input.parse::<Token![=]>()?; // skip '='

            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                let lit_str = lit.value();

                let check_empty_lit = |s| {
                    if lit_str.is_empty() {
                        Err(input.error(format!(
                            "`#[structopt({} = \"\")]` is deprecated in structopt 0.3, \
                             now it's default behavior",
                            s
                        )))
                    } else {
                        Ok(())
                    }
                };

                match &*name_str {
                    "rename_all" => Ok(RenameAll(name, lit)),
                    "rename_all_env" => Ok(RenameAllEnv(name, lit)),
                    "default_value" => Ok(DefaultValue(name, Some(lit))),

                    "version" => {
                        check_empty_lit("version")?;
                        Ok(Version(name, lit))
                    }

                    "author" => {
                        check_empty_lit("author")?;
                        Ok(Author(name, Some(lit)))
                    }

                    "about" => {
                        check_empty_lit("about")?;
                        Ok(About(name, Some(lit)))
                    }

                    "skip" => {
                        let expr = ExprLit {
                            attrs: vec![],
                            lit: Lit::Str(lit),
                        };
                        let expr = Expr::Lit(expr);
                        Ok(Skip(name, Some(expr)))
                    }

                    _ => Ok(NameLitStr(name, lit)),
                }
            } else {
                let expr = input.parse::<Expr>()?;
                if name_str == "skip" {
                    Ok(Skip(name, Some(expr)))
                } else {
                    Ok(NameExpr(name, expr))
                }
            }
        } else if input.peek(syn::token::Paren) {
            // `name(...)` attributes.
            let nested;
            parenthesized!(nested in input);

            match name_str.as_ref() {
                "parse" => {
                    let parser_specs: Punctuated<ParserSpec, Token![,]> =
                        Punctuated::parse_terminated_with(&nested, ParserSpec::parse)?;

                    if parser_specs.len() == 1 {
                        Ok(Parse(name, parser_specs[0].clone()))
                    } else {
                        return Err(syn::Error::new(name.span(), "`parse` must have exactly one argument"));
                    }
                }

                "raw" => {
                    let bool_token = nested.parse::<LitBool>().map_err(|_err| {
                        // NOTE: this previously emitted:
                        //
                        // help = "if you meant to call `clap::Arg::raw()` method \
                        //     you should use bool literal, like `raw(true)` or `raw(false)`";
                        // note = raw_method_suggestion(nested);
                        //
                        // using proc-macro-error, but this is not possible using proc-macro2-diagnostics
                        // because this function returns a syn::Result rather than a Result<_, Diagnostic>.
                        syn::Error::new(name.span(), 
                            "`#[structopt(raw(...))` attributes are removed in structopt 0.3, \
                            they are replaced with raw methods"
                        )
                    })?;
                    let expr = ExprLit {
                        attrs: vec![],
                        lit: Lit::Bool(bool_token),
                    };
                    let expr = Expr::Lit(expr);
                    Ok(MethodCall(name, vec![expr]))
                }

                _ => {
                    let method_args: Punctuated<_, Token![,]> =
                        Punctuated::parse_terminated_with(&nested, Expr::parse)?;
                    Ok(MethodCall(name, Vec::from_iter(method_args)))
                }
            }
        } else {
            // Attributes represented with a sole identifier.
            match name_str.as_ref() {
                "long" => Ok(Long(name)),
                "short" => Ok(Short(name)),
                "env" => Ok(Env(name)),
                "flatten" => Ok(Flatten(name)),
                "subcommand" => Ok(Subcommand(name)),
                "external_subcommand" => Ok(ExternalSubcommand(name)),
                "no_version" => Ok(NoVersion(name)),
                "verbatim_doc_comment" => Ok(VerbatimDocComment(name)),

                "default_value" => Ok(DefaultValue(name, None)),
                "about" => Ok(About(name, None)),
                "author" => Ok(Author(name, None)),

                "skip" => Ok(Skip(name, None)),

                "version" => Err(input.error(
                    "#[structopt(version)] is invalid attribute, \
                     structopt 0.3 inherits version from Cargo.toml by default, \
                     no attribute needed",
                )),

                name_str => Err(input.error(format!("unexpected attribute: {}", name_str))),
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

#[allow(dead_code)]
fn raw_method_suggestion(ts: ParseBuffer) -> String {
    let do_parse = move || -> Result<(Ident, Punctuated<Expr, Token![,]>), syn::Error> {
        let name = ts.parse()?;
        let _eq: Token![=] = ts.parse()?;
        let val: LitStr = ts.parse()?;
        let exprs = val.parse_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
        Ok((name, exprs))
    };

    fn to_string<T: ToTokens>(val: &T) -> String {
        val.to_token_stream()
            .to_string()
            .replace(" ", "")
            .replace(",", ", ")
    }

    if let Ok((name, exprs)) = do_parse() {
        let suggestion = if exprs.len() == 1 {
            let val = to_string(&exprs[0]);
            format!(" = {}", val)
        } else {
            let val = exprs
                .into_iter()
                .map(|expr| to_string(&expr))
                .collect::<Vec<_>>()
                .join(", ");

            format!("({})", val)
        };

        format!(
            "if you need to call `clap::Arg/App::{}` method you \
             can do it like this: #[structopt({}{})]",
            name, name, suggestion
        )
    } else {
        "if you need to call some method from `clap::Arg/App` \
         you should use raw method, see \
         https://docs.rs/structopt/0.3/structopt/#raw-methods"
            .into()
    }
}

pub fn parse_structopt_attributes(all_attrs: &[Attribute]) -> syn::Result<Vec<StructOptAttr>> {
    all_attrs
        .iter()
        .filter(|attr| attr.path().is_ident("structopt"))
        .try_fold(Vec::new(), |mut acc, attr| {
            let attrs =
                attr.parse_args_with(Punctuated::<StructOptAttr, Token![,]>::parse_terminated)?;
            acc.extend(attrs);
            Ok(acc)
        })
}
