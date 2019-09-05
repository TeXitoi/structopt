// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::spanned::Sp;

use std::env;

use heck::{CamelCase, KebabCase, MixedCase, ShoutySnakeCase, SnakeCase};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::span_error;
use quote::quote;
use syn::{
    self, spanned::Spanned, AngleBracketedGenericArguments, Attribute, GenericArgument, Ident,
    LitStr, MetaNameValue, PathArguments, PathSegment, Type::Path, TypePath,
};

use crate::parse::*;

#[derive(Clone, Debug)]
pub enum Kind {
    Arg(Sp<Ty>),
    Subcommand(Sp<Ty>),
    FlattenStruct,
    Skip,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Ty {
    Bool,
    Vec,
    Option,
    OptionOption,
    OptionVec,
    Other,
}

#[derive(Clone)]
pub struct Method {
    name: Ident,
    args: TokenStream,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Parser {
    FromStr,
    TryFromStr,
    FromOsStr,
    TryFromOsStr,
    FromOccurrences,
}

/// Defines the casing for the attributes long representation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CasingStyle {
    /// Indicate word boundaries with uppercase letter, excluding the first word.
    Camel,
    /// Keep all letters lowercase and indicate word boundaries with hyphens.
    Kebab,
    /// Indicate word boundaries with uppercase letter, including the first word.
    Pascal,
    /// Keep all letters uppercase and indicate word boundaries with underscores.
    ScreamingSnake,
    /// Keep all letters lowercase and indicate word boundaries with underscores.
    Snake,
    /// Use the original attribute name defined in the code.
    Verbatim,
}

#[derive(Clone)]
pub struct Attrs {
    name: Sp<String>,
    cased_name: String,
    casing: Sp<CasingStyle>,
    methods: Vec<Method>,
    parser: Sp<(Sp<Parser>, TokenStream)>,
    author: Option<(Ident, LitStr)>,
    about: Option<(Ident, LitStr)>,
    version: Option<(Ident, LitStr)>,
    no_version: Option<Ident>,
    has_custom_parser: bool,
    kind: Sp<Kind>,
}

impl Parser {
    fn from_ident(ident: Ident) -> Sp<Self> {
        use Parser::*;

        let p = |kind| Sp::new(kind, ident.span());
        match &*ident.to_string() {
            "from_str" => p(FromStr),
            "try_from_str" => p(TryFromStr),
            "from_os_str" => p(FromOsStr),
            "try_from_os_str" => p(TryFromOsStr),
            "from_occurrences" => p(FromOccurrences),
            s => span_error!(ident.span(), "unsupported parser `{}`", s),
        }
    }
}

impl CasingStyle {
    fn translate(&self, input: &str) -> String {
        use CasingStyle::*;

        match self {
            Pascal => input.to_camel_case(),
            Kebab => input.to_kebab_case(),
            Camel => input.to_mixed_case(),
            ScreamingSnake => input.to_shouty_snake_case(),
            Snake => input.to_snake_case(),
            Verbatim => String::from(input),
        }
    }

    fn from_lit(name: LitStr) -> Sp<Self> {
        use CasingStyle::*;

        let normalized = name.value().to_camel_case().to_lowercase();
        let cs = |kind| Sp::new(kind, name.span());

        match normalized.as_ref() {
            "camel" | "camelcase" => cs(Camel),
            "kebab" | "kebabcase" => cs(Kebab),
            "pascal" | "pascalcase" => cs(Pascal),
            "screamingsnake" | "screamingsnakecase" => cs(ScreamingSnake),
            "snake" | "snakecase" => cs(Snake),
            "verbatim" | "verbatimcase" => cs(Verbatim),
            s => span_error!(name.span(), "unsupported casing: `{}`", s),
        }
    }
}

impl Attrs {
    fn new(name: Sp<String>, casing: Sp<CasingStyle>) -> Self {
        let cased_name = casing.translate(&name);

        Self {
            name,
            cased_name,
            casing,
            methods: vec![],
            parser: Sp::call_site((
                Sp::call_site(Parser::TryFromStr),
                quote!(::std::str::FromStr::from_str),
            )),
            about: None,
            author: None,
            version: None,
            no_version: None,

            has_custom_parser: false,
            kind: Sp::call_site(Kind::Arg(Sp::call_site(Ty::Other))),
        }
    }

    /// push `.method("str literal")`
    fn push_str_method(&mut self, name: Sp<String>, arg: Sp<String>) {
        match (&**name, &**arg) {
            ("name", _) => {
                self.cased_name = self.casing.translate(&arg);
                self.name = arg;
            }
            _ => self.methods.push(Method {
                name: name.as_ident(),
                args: quote!(#arg),
            }),
        }
    }

    fn push_attrs(&mut self, attrs: &[Attribute]) {
        use crate::parse::StructOptAttr::*;

        fn from_lit_or_env(
            ident: Ident,
            lit: Option<LitStr>,
            env_var: &str,
        ) -> Option<(Ident, LitStr)> {
            let lit = lit.unwrap_or_else(|| {
                let gen = env::var(env_var)
                    .unwrap_or_else(|_|
                     span_error!(ident.span(), "`{}` environment variable is not defined, use `{} = \"{}\"` to set it manually", env_var, env_var, env_var));
                LitStr::new(&gen, Span::call_site())
            });

            Some((ident, lit))
        }

        for attr in parse_structopt_attributes(attrs) {
            match attr {
                Short(ident) => {
                    let cased_name = Sp::call_site(self.cased_name.clone());
                    self.push_str_method(ident.into(), cased_name);
                }

                Long(ident) => {
                    let cased_name = Sp::call_site(self.cased_name.clone());
                    self.push_str_method(ident.into(), cased_name);
                }

                Subcommand(ident) => {
                    let ty = Sp::call_site(Ty::Other);
                    let kind = Sp::new(Kind::Subcommand(ty), ident.span());
                    self.set_kind(kind);
                }

                Flatten(ident) => {
                    let kind = Sp::new(Kind::FlattenStruct, ident.span());
                    self.set_kind(kind);
                }

                Skip(ident) => {
                    let kind = Sp::new(Kind::Skip, ident.span());
                    self.set_kind(kind);
                }

                NoVersion(ident) => self.no_version = Some(ident),

                About(ident, about) => {
                    self.about = from_lit_or_env(ident, about, "CARGO_PKG_DESCRIPTION")
                }

                Author(ident, author) => {
                    self.author =
                        from_lit_or_env(ident, author, "CARGO_PKG_AUTHORS").map(|(ident, lit)| {
                            let value = lit.value().replace(":", ", ");
                            (ident.clone(), LitStr::new(&value, ident.span()))
                        })
                }

                Version(ident, version) => self.version = Some((ident, version)),

                NameLitStr(name, lit) => {
                    self.push_str_method(name.into(), lit.into());
                }

                NameExpr(name, expr) => self.methods.push(Method {
                    name: name.into(),
                    args: quote!(#expr),
                }),

                MethodCall(name, args) => self.methods.push(Method {
                    name: name.into(),
                    args: quote!(#(#args),*),
                }),

                RenameAll(_, casing_lit) => {
                    self.casing = CasingStyle::from_lit(casing_lit);
                    self.cased_name = self.casing.translate(&self.name);
                }

                Parse(ident, spec) => {
                    self.has_custom_parser = true;

                    self.parser = match spec.parse_func {
                        None => {
                            use crate::attrs::Parser::*;

                            let parser: Sp<_> = Parser::from_ident(spec.kind).into();
                            let function = match *parser {
                                FromStr | FromOsStr => quote!(::std::convert::From::from),
                                TryFromStr => quote!(::std::str::FromStr::from_str),
                                TryFromOsStr => span_error!(
                                    parser.span(),
                                    "cannot omit parser function name with `try_from_os_str`"
                                ),
                                FromOccurrences => quote!({ |v| v as _ }),
                            };
                            Sp::new((parser, function), ident.span())
                        }

                        Some(func) => {
                            let parser: Sp<_> = Parser::from_ident(spec.kind).into();
                            match func {
                                syn::Expr::Path(_) => {
                                    Sp::new((parser, quote!(#func)), ident.span())
                                }
                                _ => span_error!(
                                    func.span(),
                                    "`parse` argument must be a function path"
                                ),
                            }
                        }
                    }
                }
            }
        }
    }

    fn push_doc_comment(&mut self, attrs: &[Attribute], name: &str) {
        let doc_comments = attrs
            .iter()
            .filter_map(|attr| {
                if attr.path.is_ident("doc") {
                    attr.parse_meta().ok()
                } else {
                    None
                }
            })
            .filter_map(|attr| {
                use crate::Lit::*;
                use crate::Meta::*;
                if let NameValue(MetaNameValue {
                    path, lit: Str(s), ..
                }) = attr
                {
                    if !path.is_ident("doc") {
                        return None;
                    }
                    let value = s.value();

                    let text = value
                        .trim_start_matches("//!")
                        .trim_start_matches("///")
                        .trim_start_matches("/*!")
                        .trim_start_matches("/**")
                        .trim_end_matches("*/")
                        .trim();
                    if text.is_empty() {
                        Some("\n\n".to_string())
                    } else {
                        Some(text.to_string())
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if doc_comments.is_empty() {
            return;
        }

        let merged_lines = doc_comments
            .join(" ")
            .split('\n')
            .map(str::trim)
            .map(str::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        let expected_doc_comment_split = if let Some(content) = doc_comments.get(1) {
            (doc_comments.len() > 2) && (content == "\n\n")
        } else {
            false
        };

        if expected_doc_comment_split {
            let long_name = Sp::call_site(format!("long_{}", name));

            self.methods.push(Method {
                name: long_name.as_ident(),
                args: quote!(#merged_lines),
            });

            // Remove trailing whitespace and period from short help, as rustdoc
            // best practice is to use complete sentences, but command-line help
            // typically omits the trailing period.
            let short_arg = doc_comments
                .first()
                .map(|s| s.trim())
                .map_or("", |s| s.trim_end_matches('.'));

            self.methods.push(Method {
                name: Ident::new(name, Span::call_site()),
                args: quote!(#short_arg),
            });
        } else {
            self.methods.push(Method {
                name: Ident::new(name, Span::call_site()),
                args: quote!(#merged_lines),
            });
        }
    }

    pub fn from_struct(
        attrs: &[Attribute],
        name: Sp<String>,
        argument_casing: Sp<CasingStyle>,
    ) -> Self {
        let mut res = Self::new(name, argument_casing);
        res.push_attrs(attrs);
        res.push_doc_comment(attrs, "about");

        if res.has_custom_parser {
            span_error!(
                res.parser.span(),
                "parse attribute is only allowed on fields"
            );
        }
        match &*res.kind {
            Kind::Subcommand(_) => {
                span_error!(res.kind.span(), "subcommand is only allowed on fields")
            }
            Kind::FlattenStruct => {
                span_error!(res.kind.span(), "flatten is only allowed on fields")
            }
            Kind::Skip => span_error!(res.kind.span(), "skip is only allowed on fields"),
            Kind::Arg(_) => res,
        }
    }

    fn ty_from_field(ty: &syn::Type) -> Sp<Ty> {
        let t = |kind| Sp::new(kind, ty.span());
        if let Path(TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) = *ty
        {
            match segments.iter().last().unwrap().ident.to_string().as_str() {
                "bool" => t(Ty::Bool),
                "Option" => sub_type(ty)
                    .map(Attrs::ty_from_field)
                    .map(|ty| match *ty {
                        Ty::Option => t(Ty::OptionOption),
                        Ty::Vec => t(Ty::OptionVec),
                        _ => t(Ty::Option),
                    })
                    .unwrap_or(t(Ty::Option)),

                "Vec" => t(Ty::Vec),
                _ => t(Ty::Other),
            }
        } else {
            t(Ty::Other)
        }
    }

    pub fn from_field(field: &syn::Field, struct_casing: Sp<CasingStyle>) -> Self {
        let name = field.ident.clone().unwrap();
        let mut res = Self::new(name.into(), struct_casing);
        res.push_doc_comment(&field.attrs, "help");
        res.push_attrs(&field.attrs);

        match &*res.kind {
            Kind::FlattenStruct => {
                if res.has_custom_parser {
                    span_error!(
                        res.parser.span(),
                        "parse attribute is not allowed for flattened entry"
                    );
                }
                if !res.methods.is_empty() {
                    span_error!(
                        res.kind.span(),
                        "methods and doc comments are not allowed for flattened entry"
                    );
                }
            }
            Kind::Subcommand(_) => {
                if res.has_custom_parser {
                    span_error!(
                        res.parser.span(),
                        "parse attribute is not allowed for subcommand"
                    );
                }
                if let Some(m) = res.methods.iter().find(|m| m.name != "help") {
                    span_error!(
                        m.name.span(),
                        "methods in attributes are not allowed for subcommand"
                    );
                }

                let ty = Self::ty_from_field(&field.ty);
                match *ty {
                    Ty::OptionOption => {
                        span_error!(
                            ty.span(),
                            "Option<Option<T>> type is not allowed for subcommand"
                        );
                    }
                    Ty::OptionVec => {
                        span_error!(
                            ty.span(),
                            "Option<Vec<T>> type is not allowed for subcommand"
                        );
                    }
                    _ => (),
                }

                res.kind = Sp::new(Kind::Subcommand(ty), res.kind.span());
            }
            Kind::Skip => {
                if let Some(m) = res
                    .methods
                    .iter()
                    .find(|m| m.name != "help" && m.name != "long_help")
                {
                    span_error!(m.name.span(), "methods are not allowed for skipped fields");
                }
            }
            Kind::Arg(orig_ty) => {
                let mut ty = Self::ty_from_field(&field.ty);
                if res.has_custom_parser {
                    match *ty {
                        Ty::Option | Ty::Vec => (),
                        _ => ty = Sp::new(Ty::Other, ty.span()),
                    }
                }

                match *ty {
                    Ty::Bool => {
                        if let Some(m) = res.find_method("default_value") {
                            span_error!(m.name.span(), "default_value is meaningless for bool")
                        }
                        if let Some(m) = res.find_method("required") {
                            span_error!(m.name.span(), "required is meaningless for bool")
                        }
                    }
                    Ty::Option => {
                        if let Some(m) = res.find_method("default_value") {
                            span_error!(m.name.span(), "default_value is meaningless for Option")
                        }
                        if let Some(m) = res.find_method("required") {
                            span_error!(m.name.span(), "required is meaningless for Option")
                        }
                    }
                    Ty::OptionOption => {
                        // If it's a positional argument.
                        if !(res.has_method("long") || res.has_method("short")) {
                            span_error!(
                                ty.span(),
                                "Option<Option<T>> type is meaningless for positional argument"
                            )
                        }
                    }
                    Ty::OptionVec => {
                        // If it's a positional argument.
                        if !(res.has_method("long") || res.has_method("short")) {
                            span_error!(
                                ty.span(),
                                "Option<Vec<T>> type is meaningless for positional argument"
                            )
                        }
                    }

                    _ => (),
                }
                res.kind = Sp::new(Kind::Arg(ty), orig_ty.span());
            }
        }

        res
    }

    fn set_kind(&mut self, kind: Sp<Kind>) {
        if let Kind::Arg(_) = *self.kind {
            self.kind = kind;
        } else {
            span_error!(
                kind.span(),
                "subcommand, flatten and skip cannot be used together"
            );
        }
    }

    pub fn has_method(&self, name: &str) -> bool {
        self.find_method(name).is_some()
    }

    pub fn find_method(&self, name: &str) -> Option<&Method> {
        self.methods.iter().find(|m| m.name == name)
    }

    /// generate methods from attributes on top of struct or enum
    pub fn top_level_methods(&self) -> TokenStream {
        let version = match (&self.no_version, &self.version) {
            (Some(no_version), Some(_)) => span_error!(
                no_version.span(),
                "`no_version` and `version = \"version\"` can't be used together"
            ),

            (None, Some((_, version))) => quote!(.version(#version)),

            (None, None) => {
                if let Ok(version) = std::env::var("CARGO_PKG_VERSION") {
                    quote!(.version(#version))
                } else {
                    TokenStream::new()
                }
            }

            (Some(_), None) => TokenStream::new(),
        };

        let version = Some(version);
        let author = self
            .author
            .as_ref()
            .map(|(_, version)| quote!(.author(#version)));
        let about = self
            .about
            .as_ref()
            .map(|(_, version)| quote!(.about(#version)));

        let methods = self
            .methods
            .iter()
            .map(|&Method { ref name, ref args }| quote!( .#name(#args) ))
            .chain(version)
            .chain(author)
            .chain(about);

        quote!( #(#methods)* )
    }

    /// generate methods on top of a field
    pub fn field_methods(&self) -> TokenStream {
        let methods = self
            .methods
            .iter()
            .map(|&Method { ref name, ref args }| quote!( .#name(#args) ));

        quote!( #(#methods)* )
    }

    pub fn cased_name(&self) -> String {
        self.cased_name.to_string()
    }

    pub fn parser(&self) -> &(Sp<Parser>, TokenStream) {
        &self.parser
    }

    pub fn kind(&self) -> Sp<Kind> {
        self.kind.clone()
    }

    pub fn casing(&self) -> Sp<CasingStyle> {
        self.casing.clone()
    }
}

pub fn sub_type(t: &syn::Type) -> Option<&syn::Type> {
    let segs = match *t {
        Path(TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) => segments,
        _ => return None,
    };
    match *segs.iter().last().unwrap() {
        PathSegment {
            arguments:
                PathArguments::AngleBracketed(AngleBracketedGenericArguments { ref args, .. }),
            ..
        } if args.len() == 1 => {
            if let GenericArgument::Type(ref ty) = args[0] {
                Some(ty)
            } else {
                None
            }
        }
        _ => None,
    }
}
