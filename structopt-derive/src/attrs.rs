// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use heck::{CamelCase, KebabCase, MixedCase, ShoutySnakeCase, SnakeCase};
use proc_macro2::{Span, TokenStream};
use std::{env, mem};
use syn::Type::Path;
use syn::{
    self, AngleBracketedGenericArguments, Attribute, GenericArgument, Ident, LitStr, MetaList,
    MetaNameValue, PathArguments, PathSegment, TypePath,
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Kind {
    Arg(Ty),
    Subcommand(Ty),
    FlattenStruct,
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Ty {
    Bool,
    Vec,
    Option,
    OptionOption,
    Other,
}
#[derive(Debug)]
pub struct Attrs {
    name: String,
    cased_name: String,
    casing: CasingStyle,
    methods: Vec<Method>,
    parser: (Parser, TokenStream),
    has_custom_parser: bool,
    kind: Kind,
}
#[derive(Debug)]
struct Method {
    name: String,
    args: TokenStream,
}
#[derive(Debug, PartialEq)]
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

impl ::std::str::FromStr for Parser {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "from_str" => Ok(Parser::FromStr),
            "try_from_str" => Ok(Parser::TryFromStr),
            "from_os_str" => Ok(Parser::FromOsStr),
            "try_from_os_str" => Ok(Parser::TryFromOsStr),
            "from_occurrences" => Ok(Parser::FromOccurrences),
            _ => Err(format!("unsupported parser {}", s)),
        }
    }
}

impl CasingStyle {
    fn translate(self, input: &str) -> String {
        match self {
            CasingStyle::Pascal => input.to_camel_case(),
            CasingStyle::Kebab => input.to_kebab_case(),
            CasingStyle::Camel => input.to_mixed_case(),
            CasingStyle::ScreamingSnake => input.to_shouty_snake_case(),
            CasingStyle::Snake => input.to_snake_case(),
            CasingStyle::Verbatim => String::from(input),
        }
    }
}

impl ::std::str::FromStr for CasingStyle {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let name = name.to_camel_case().to_lowercase();

        let case = match name.as_ref() {
            "camel" | "camelcase" => CasingStyle::Camel,
            "kebab" | "kebabcase" => CasingStyle::Kebab,
            "pascal" | "pascalcase" => CasingStyle::Pascal,
            "screamingsnake" | "screamingsnakecase" => CasingStyle::ScreamingSnake,
            "snake" | "snakecase" => CasingStyle::Snake,
            "verbatim" | "verbatimcase" => CasingStyle::Verbatim,
            _ => return Err(format!("unsupported casing: {}", name)),
        };

        Ok(case)
    }
}

impl Attrs {
    fn new(name: String, casing: CasingStyle) -> Attrs {
        let cased_name = casing.translate(&name);

        Attrs {
            name,
            cased_name,
            casing,
            methods: vec![],
            parser: (Parser::TryFromStr, quote!(::std::str::FromStr::from_str)),
            has_custom_parser: false,
            kind: Kind::Arg(Ty::Other),
        }
    }
    fn push_str_method(&mut self, name: &str, arg: &str) {
        match (name, arg) {
            ("about", "") | ("version", "") | ("author", "") => {
                let methods = mem::replace(&mut self.methods, vec![]);
                self.methods = methods.into_iter().filter(|m| m.name != name).collect();
            }
            ("name", new_name) => {
                self.name = new_name.into();
                self.cased_name = self.casing.translate(new_name);
            }
            (name, arg) => self.methods.push(Method {
                name: name.to_string(),
                args: quote!(#arg),
            }),
        }
    }
    fn push_attrs(&mut self, attrs: &[Attribute]) {
        use Lit::*;
        use Meta::*;
        use NestedMeta::*;

        let structopt_attrs =
            attrs
                .iter()
                .filter_map(|attr| {
                    let path = &attr.path;
                    match quote!(#path).to_string().as_ref() {
                        "structopt" => Some(attr.interpret_meta().unwrap_or_else(|| {
                            panic!("invalid structopt syntax: {}", quote!(#attr))
                        })),
                        _ => None,
                    }
                })
                .flat_map(|m| match m {
                    List(l) => l.nested,
                    tokens => panic!("unsupported syntax: {}", quote!(#tokens).to_string()),
                })
                .map(|m| match m {
                    Meta(m) => m,
                    ref tokens => panic!("unsupported syntax: {}", quote!(#tokens).to_string()),
                });

        for attr in structopt_attrs {
            match attr {
                NameValue(MetaNameValue {
                    ref ident,
                    lit: Str(ref value),
                    ..
                }) if ident == "rename_all" => {
                    self.casing = {
                        let input = value.value();
                        ::std::str::FromStr::from_str(&input)
                            .unwrap_or_else(|error| panic!("{}", error))
                    };
                    self.cased_name = self.casing.translate(&self.name);
                }
                NameValue(MetaNameValue {
                    ident,
                    lit: Str(value),
                    ..
                }) => self.push_str_method(&ident.to_string(), &value.value()),
                NameValue(MetaNameValue { ident, lit, .. }) => self.methods.push(Method {
                    name: ident.to_string(),
                    args: quote!(#lit),
                }),
                List(MetaList {
                    ref ident,
                    ref nested,
                    ..
                }) if ident == "parse" => {
                    if nested.len() != 1 {
                        panic!("parse must have exactly one argument");
                    }
                    self.has_custom_parser = true;
                    self.parser = match nested[0] {
                        Meta(NameValue(MetaNameValue {
                            ref ident,
                            lit: Str(ref v),
                            ..
                        })) => {
                            let function: syn::Path = v.parse().expect("parser function path");
                            let parser = ident.to_string().parse().unwrap();
                            (parser, quote!(#function))
                        }
                        Meta(Word(ref i)) => {
                            use Parser::*;
                            let parser = i.to_string().parse().unwrap();
                            let function = match parser {
                                FromStr => quote!(::std::convert::From::from),
                                TryFromStr => quote!(::std::str::FromStr::from_str),
                                FromOsStr => quote!(::std::convert::From::from),
                                TryFromOsStr => panic!(
                                    "cannot omit parser function name with `try_from_os_str`"
                                ),
                                FromOccurrences => quote!({ |v| v as _ }),
                            };
                            (parser, function)
                        }
                        ref l => panic!("unknown value parser specification: {}", quote!(#l)),
                    };
                }
                List(MetaList {
                    ref ident,
                    ref nested,
                    ..
                }) if ident == "raw" => {
                    for method in nested {
                        match *method {
                            Meta(NameValue(MetaNameValue {
                                ref ident,
                                lit: Str(ref v),
                                ..
                            })) => self.push_raw_method(&ident.to_string(), v),
                            ref mi => panic!("unsupported raw entry: {}", quote!(#mi)),
                        }
                    }
                }
                Word(ref w) if w == "subcommand" => {
                    self.set_kind(Kind::Subcommand(Ty::Other));
                }
                Word(ref w) if w == "flatten" => {
                    self.set_kind(Kind::FlattenStruct);
                }
                Word(ref w) if w == "long" => {
                    let cased_name = &self.cased_name.clone();
                    self.push_str_method("long", cased_name);
                }
                Word(ref w) if w == "short" => {
                    let cased_named = &self.cased_name.clone();
                    self.push_str_method("short", cased_named);
                }
                ref i @ List(..) | ref i @ Word(..) => panic!("unsupported option: {}", quote!(#i)),
            }
        }
    }
    fn push_raw_method(&mut self, name: &str, args: &LitStr) {
        let ts: TokenStream = args.value().parse().unwrap_or_else(|_| {
            panic!(
                "bad parameter {} = {}: the parameter must be valid rust code",
                name,
                quote!(#args)
            )
        });
        self.methods.push(Method {
            name: name.to_string(),
            args: quote!(#(#ts)*),
        })
    }
    fn push_doc_comment(&mut self, attrs: &[Attribute], name: &str) {
        let doc_comments = attrs
            .iter()
            .filter_map(|attr| {
                let path = &attr.path;
                if quote!(#path).to_string() == "doc" {
                    attr.interpret_meta()
                } else {
                    None
                }
            })
            .filter_map(|attr| {
                use Lit::*;
                use Meta::*;
                if let NameValue(MetaNameValue {
                    ident, lit: Str(s), ..
                }) = attr
                {
                    if ident != "doc" {
                        return None;
                    }
                    let value = s.value();
                    let text = value
                        .trim_left_matches("//!")
                        .trim_left_matches("///")
                        .trim_left_matches("/*!")
                        .trim_left_matches("/**")
                        .trim_right_matches("*/")
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
            let long_name = String::from("long_") + name;

            self.methods.push(Method {
                name: long_name,
                args: quote!(#merged_lines),
            });

            // Remove trailing whitespace and period from short help, as rustdoc
            // best practice is to use complete sentences, but command-line help
            // typically omits the trailing period.
            let short_arg = doc_comments
                .first()
                .map(String::as_ref)
                .map(str::trim)
                .map(|s| s.trim_right_matches('.'))
                .unwrap_or("");

            self.methods.push(Method {
                name: name.to_string(),
                args: quote!(#short_arg),
            });
        } else {
            self.methods.push(Method {
                name: name.to_string(),
                args: quote!(#merged_lines),
            });
        }
    }
    pub fn from_struct(attrs: &[Attribute], name: String, argument_casing: CasingStyle) -> Attrs {
        let mut res = Self::new(name, argument_casing);
        let attrs_with_env = [
            ("version", "CARGO_PKG_VERSION"),
            ("about", "CARGO_PKG_DESCRIPTION"),
            ("author", "CARGO_PKG_AUTHORS"),
        ];
        attrs_with_env
            .iter()
            .filter_map(|&(m, v)| env::var(v).ok().and_then(|arg| Some((m, arg))))
            .filter(|&(_, ref arg)| !arg.is_empty())
            .for_each(|(name, arg)| {
                let new_arg = if name == "author" {
                    arg.replace(":", ", ")
                } else {
                    arg
                };
                res.push_str_method(name, &new_arg);
            });
        res.push_doc_comment(attrs, "about");
        res.push_attrs(attrs);
        if res.has_custom_parser {
            panic!("parse attribute is only allowed on fields");
        }
        match res.kind {
            Kind::Subcommand(_) => panic!("subcommand is only allowed on fields"),
            Kind::FlattenStruct => panic!("flatten is only allowed on fields"),
            Kind::Arg(_) => res,
        }
    }
    fn ty_from_field(ty: &syn::Type) -> Ty {
        if let Path(TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) = *ty
        {
            match segments.iter().last().unwrap().ident.to_string().as_str() {
                "bool" => Ty::Bool,
                "Option" => match sub_type(ty).map(Attrs::ty_from_field) {
                    Some(Ty::Option) => Ty::OptionOption,
                    _ => Ty::Option,
                },
                "Vec" => Ty::Vec,
                _ => Ty::Other,
            }
        } else {
            Ty::Other
        }
    }
    pub fn from_field(field: &syn::Field, struct_casing: CasingStyle) -> Attrs {
        let name = field.ident.as_ref().unwrap().to_string();
        let mut res = Self::new(name, struct_casing);
        res.push_doc_comment(&field.attrs, "help");
        res.push_attrs(&field.attrs);

        match res.kind {
            Kind::FlattenStruct => {
                if res.has_custom_parser {
                    panic!("parse attribute is not allowed for flattened entry");
                }
                if !res.methods.is_empty() {
                    panic!("methods and doc comments are not allowed for flattened entry");
                }
            }
            Kind::Subcommand(_) => {
                if res.has_custom_parser {
                    panic!("parse attribute is not allowed for subcommand");
                }
                if !res.methods.iter().all(|m| m.name == "help") {
                    panic!("methods in attributes is not allowed for subcommand");
                }
                let ty = Self::ty_from_field(&field.ty);
                if ty == Ty::OptionOption {
                    panic!("Option<Option<T>> type is not allowed for subcommand")
                }
                res.kind = Kind::Subcommand(ty);
            }
            Kind::Arg(_) => {
                let mut ty = Self::ty_from_field(&field.ty);
                if res.has_custom_parser {
                    match ty {
                        Ty::Option | Ty::Vec => (),
                        _ => ty = Ty::Other,
                    }
                }
                match ty {
                    Ty::Bool => {
                        if res.has_method("default_value") {
                            panic!("default_value is meaningless for bool")
                        }
                        if res.has_method("required") {
                            panic!("required is meaningless for bool")
                        }
                    }
                    Ty::Option => {
                        if res.has_method("default_value") {
                            panic!("default_value is meaningless for Option")
                        }
                        if res.has_method("required") {
                            panic!("required is meaningless for Option")
                        }
                    }
                    Ty::OptionOption => {
                        // If it's a positional argument.
                        if !(res.has_method("long") || res.has_method("short")) {
                            panic!("Option<Option<T>> type is meaningless for positional argument")
                        }
                    }
                    _ => (),
                }
                res.kind = Kind::Arg(ty);
            }
        }

        res
    }
    fn set_kind(&mut self, kind: Kind) {
        if let Kind::Arg(_) = self.kind {
            self.kind = kind;
        } else {
            panic!("subcommands cannot be flattened");
        }
    }
    pub fn has_method(&self, method: &str) -> bool {
        self.methods.iter().any(|m| m.name == method)
    }
    pub fn methods(&self) -> TokenStream {
        let methods = self.methods.iter().map(|&Method { ref name, ref args }| {
            let name = Ident::new(&name, Span::call_site());
            quote!( .#name(#args) )
        });
        quote!( #(#methods)* )
    }
    pub fn cased_name(&self) -> &str {
        &self.cased_name
    }
    pub fn parser(&self) -> &(Parser, TokenStream) {
        &self.parser
    }
    pub fn kind(&self) -> Kind {
        self.kind
    }
    pub fn casing(&self) -> CasingStyle {
        self.casing
    }
}

pub fn sub_type(t: &syn::Type) -> Option<&syn::Type> {
    let segs = match *t {
        syn::Type::Path(TypePath {
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
