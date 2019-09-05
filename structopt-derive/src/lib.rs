// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate is custom derive for `StructOpt`. It should not be used
//! directly. See [structopt documentation](https://docs.rs/structopt)
//! for the usage of `#[derive(StructOpt)]`.

extern crate proc_macro;

mod attrs;
mod parse;
mod spanned;

use crate::{
    attrs::{sub_type, Attrs, CasingStyle, Kind, Parser, Ty},
    spanned::Sp,
};

use proc_macro2::{Span, TokenStream};
use proc_macro_error::{call_site_error, filter_macro_errors, set_dummy, span_error};
use quote::{quote, quote_spanned};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, *};

/// Default casing style for generated arguments.
const DEFAULT_CASING: CasingStyle = CasingStyle::Kebab;

/// Output for the `gen_xxx()` methods were we need more than a simple stream of tokens.
///
/// The output of a generation method is not only the stream of new tokens but also the attribute
/// information of the current element. These attribute information may contain valuable information
/// for any kind of child arguments.
struct GenOutput {
    tokens: TokenStream,
    attrs: Attrs,
}

/// Generates the `StructOpt` impl.
#[proc_macro_derive(StructOpt, attributes(structopt))]
pub fn structopt(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    filter_macro_errors! {
        let input: DeriveInput = syn::parse(input).unwrap();
        let gen = impl_structopt(&input);
        gen.into()
    }
}

/// Generate a block of code to add arguments/subcommands corresponding to
/// the `fields` to an app.
fn gen_augmentation(
    fields: &Punctuated<Field, Comma>,
    app_var: &Ident,
    parent_attribute: &Attrs,
) -> TokenStream {
    let mut subcmds = fields.iter().filter_map(|field| {
        let attrs = Attrs::from_field(field, parent_attribute.casing());
        if let Kind::Subcommand(ty) = &*attrs.kind() {
            let subcmd_type = match (**ty, sub_type(&field.ty)) {
                (Ty::Option, Some(sub_type)) => sub_type,
                _ => &field.ty,
            };
            let required = if **ty == Ty::Option {
                quote!()
            } else {
                quote! {
                    let #app_var = #app_var.setting(
                        ::structopt::clap::AppSettings::SubcommandRequiredElseHelp
                    );
                }
            };

            let span = field.span();
            let ts = quote! {
                let #app_var = <#subcmd_type>::augment_clap( #app_var );
                #required
            };
            Some((span, ts))
        } else {
            None
        }
    });

    let subcmd = subcmds.next().map(|(_, ts)| ts);
    if let Some((span, _)) = subcmds.next() {
        span_error!(
            span,
            "multiple subcommand sets are not allowed, that's the second"
        );
    }

    let args = fields.iter().filter_map(|field| {
        let attrs = Attrs::from_field(field, parent_attribute.casing());
        match &*attrs.kind() {
            Kind::Subcommand(_) | Kind::Skip => None,
            Kind::FlattenStruct => {
                let ty = &field.ty;
                Some(quote! {
                    let #app_var = <#ty>::augment_clap(#app_var);
                    let #app_var = if <#ty>::is_subcommand() {
                        #app_var.setting(::structopt::clap::AppSettings::SubcommandRequiredElseHelp)
                    } else {
                        #app_var
                    };
                })
            }
            Kind::Arg(ty) => {
                let convert_type = match **ty {
                    Ty::Vec | Ty::Option => sub_type(&field.ty).unwrap_or(&field.ty),
                    Ty::OptionOption | Ty::OptionVec => sub_type(&field.ty).and_then(sub_type).unwrap_or(&field.ty),
                    _ => &field.ty,
                };

                let occurrences = *attrs.parser().0 == Parser::FromOccurrences;

                let (parser, f) = attrs.parser();
                let validator = match **parser {
                    Parser::TryFromStr => quote! {
                        .validator(|s| {
                            #f(&s)
                            .map(|_: #convert_type| ())
                            .map_err(|e| e.to_string())
                        })
                    },
                    Parser::TryFromOsStr => quote! {
                        .validator_os(|s| #f(&s).map(|_: #convert_type| ()))
                    },
                    _ => quote!(),
                };

                let modifier = match **ty {
                    Ty::Bool => quote!( .takes_value(false).multiple(false) ),
                    Ty::Option => quote!( .takes_value(true).multiple(false) #validator ),
                    Ty::OptionOption => {
                        quote! ( .takes_value(true).multiple(false).min_values(0).max_values(1) #validator )
                    }
                    Ty::OptionVec => {
                        quote! ( .takes_value(true).multiple(true).min_values(0) #validator )
                    }
                    Ty::Vec => quote!( .takes_value(true).multiple(true) #validator ),
                    Ty::Other if occurrences => quote!( .takes_value(false).multiple(true) ),
                    Ty::Other => {
                        let required = !attrs.has_method("default_value");
                        quote!( .takes_value(true).multiple(false).required(#required) #validator )
                    }
                };

                let name = attrs.cased_name();
                let methods = attrs.field_methods();

                Some(quote! {
                    let #app_var = #app_var.arg(
                        ::structopt::clap::Arg::with_name(#name)
                            #modifier
                            #methods
                    );
                })
            }
        }
    });

    quote! {{
        #( #args )*
        #subcmd
        #app_var
    }}
}

fn gen_constructor(fields: &Punctuated<Field, Comma>, parent_attribute: &Attrs) -> TokenStream {
    let fields = fields.iter().map(|field| {
        let attrs = Attrs::from_field(field, parent_attribute.casing());
        let field_name = field.ident.as_ref().unwrap();
        let kind = attrs.kind();
        match &*kind {
            Kind::Subcommand(ty) => {
                let subcmd_type = match (**ty, sub_type(&field.ty)) {
                    (Ty::Option, Some(sub_type)) => sub_type,
                    _ => &field.ty,
                };
                let unwrapper = match **ty {
                    Ty::Option => quote!(),
                    _ => quote!( .unwrap() ),
                };
                quote!(#field_name: <#subcmd_type>::from_subcommand(matches.subcommand())#unwrapper)
            }
            Kind::FlattenStruct => quote!(#field_name: ::structopt::StructOpt::from_clap(matches)),
            Kind::Skip => quote_spanned!(kind.span()=> #field_name: Default::default()),
            Kind::Arg(ty) => {
                use crate::attrs::Parser::*;
                let (parser, f) = attrs.parser();
                let (value_of, values_of, parse) = match **parser {
                    FromStr => (quote!(value_of), quote!(values_of), f.clone()),
                    TryFromStr => (
                        quote!(value_of),
                        quote!(values_of),
                        quote!(|s| #f(s).unwrap()),
                    ),
                    FromOsStr => (quote!(value_of_os), quote!(values_of_os), f.clone()),
                    TryFromOsStr => (
                        quote!(value_of_os),
                        quote!(values_of_os),
                        quote!(|s| #f(s).unwrap()),
                    ),
                    FromOccurrences => (quote!(occurrences_of), quote!(), f.clone()),
                };

                let occurrences = *attrs.parser().0 == Parser::FromOccurrences;
                let name = attrs.cased_name();
                let field_value = match **ty {
                    Ty::Bool => quote!(matches.is_present(#name)),
                    Ty::Option => quote! {
                        matches.#value_of(#name)
                            .map(#parse)
                    },
                    Ty::OptionOption => quote! {
                        if matches.is_present(#name) {
                            Some(matches.#value_of(#name).map(#parse))
                        } else {
                            None
                        }
                    },
                    Ty::OptionVec => quote! {
                        if matches.is_present(#name) {
                            Some(matches.#values_of(#name)
                                 .map(|v| v.map(#parse).collect())
                                 .unwrap_or_else(Vec::new))
                        } else {
                            None
                        }
                    },
                    Ty::Vec => quote! {
                        matches.#values_of(#name)
                            .map(|v| v.map(#parse).collect())
                            .unwrap_or_else(Vec::new)
                    },
                    Ty::Other if occurrences => quote! {
                        #parse(matches.#value_of(#name))
                    },
                    Ty::Other => quote! {
                        matches.#value_of(#name)
                            .map(#parse)
                            .unwrap()
                    },
                };

                quote!( #field_name: #field_value )
            }
        }
    });

    quote! {{
        #( #fields ),*
    }}
}

fn gen_from_clap(
    struct_name: &Ident,
    fields: &Punctuated<Field, Comma>,
    parent_attribute: &Attrs,
) -> TokenStream {
    let field_block = gen_constructor(fields, parent_attribute);

    quote! {
        fn from_clap(matches: &::structopt::clap::ArgMatches) -> Self {
            #struct_name #field_block
        }
    }
}

fn gen_clap(attrs: &[Attribute]) -> GenOutput {
    let name = std::env::var("CARGO_PKG_NAME").ok().unwrap_or_default();

    let attrs = Attrs::from_struct(attrs, Sp::call_site(name), Sp::call_site(DEFAULT_CASING));
    let tokens = {
        let name = attrs.cased_name();
        let methods = attrs.top_level_methods();

        quote!(::structopt::clap::App::new(#name)#methods)
    };

    GenOutput { tokens, attrs }
}

fn gen_clap_struct(struct_attrs: &[Attribute]) -> GenOutput {
    let initial_clap_app_gen = gen_clap(struct_attrs);
    let clap_tokens = initial_clap_app_gen.tokens;

    let augmented_tokens = quote! {
        fn clap<'a, 'b>() -> ::structopt::clap::App<'a, 'b> {
            let app = #clap_tokens;
            Self::augment_clap(app)
        }
    };

    GenOutput {
        tokens: augmented_tokens,
        attrs: initial_clap_app_gen.attrs,
    }
}

fn gen_augment_clap(fields: &Punctuated<Field, Comma>, parent_attribute: &Attrs) -> TokenStream {
    let app_var = Ident::new("app", Span::call_site());
    let augmentation = gen_augmentation(fields, &app_var, parent_attribute);
    quote! {
        pub fn augment_clap<'a, 'b>(
            #app_var: ::structopt::clap::App<'a, 'b>
        ) -> ::structopt::clap::App<'a, 'b> {
            #augmentation
        }
    }
}

fn gen_clap_enum(enum_attrs: &[Attribute]) -> GenOutput {
    let initial_clap_app_gen = gen_clap(enum_attrs);
    let clap_tokens = initial_clap_app_gen.tokens;

    let tokens = quote! {
        fn clap<'a, 'b>() -> ::structopt::clap::App<'a, 'b> {
            let app = #clap_tokens
                .setting(::structopt::clap::AppSettings::SubcommandRequiredElseHelp);
            Self::augment_clap(app)
        }
    };

    GenOutput {
        tokens,
        attrs: initial_clap_app_gen.attrs,
    }
}

fn gen_augment_clap_enum(
    variants: &Punctuated<Variant, Comma>,
    parent_attribute: &Attrs,
) -> TokenStream {
    use syn::Fields::*;

    let subcommands = variants.iter().map(|variant| {
        let attrs = Attrs::from_struct(
            &variant.attrs,
            variant.ident.clone().into(),
            parent_attribute.casing(),
        );
        let app_var = Ident::new("subcommand", Span::call_site());
        let arg_block = match variant.fields {
            Named(ref fields) => gen_augmentation(&fields.named, &app_var, &attrs),
            Unit => quote!( #app_var ),
            Unnamed(FieldsUnnamed { ref unnamed, .. }) if unnamed.len() == 1 => {
                let ty = &unnamed[0];
                quote! {
                    {
                        let #app_var = <#ty>::augment_clap(#app_var);
                        if <#ty>::is_subcommand() {
                            #app_var.setting(
                                ::structopt::clap::AppSettings::SubcommandRequiredElseHelp
                            )
                        } else {
                            #app_var
                        }
                    }
                }
            }
            Unnamed(..) => call_site_error!("{}: tuple enums are not supported", variant.ident),
        };

        let name = attrs.cased_name();
        let from_attrs = attrs.top_level_methods();

        quote! {
            .subcommand({
                let #app_var = ::structopt::clap::SubCommand::with_name(#name);
                let #app_var = #arg_block;
                #app_var#from_attrs
            })
        }
    });

    quote! {
        pub fn augment_clap<'a, 'b>(
            app: ::structopt::clap::App<'a, 'b>
        ) -> ::structopt::clap::App<'a, 'b> {
            app #( #subcommands )*
        }
    }
}

fn gen_from_clap_enum(name: &Ident) -> TokenStream {
    quote! {
        fn from_clap(matches: &::structopt::clap::ArgMatches) -> Self {
            <#name>::from_subcommand(matches.subcommand())
                .unwrap()
        }
    }
}

fn gen_from_subcommand(
    name: &Ident,
    variants: &Punctuated<Variant, Comma>,
    parent_attribute: &Attrs,
) -> TokenStream {
    use syn::Fields::*;

    let match_arms = variants.iter().map(|variant| {
        let attrs = Attrs::from_struct(
            &variant.attrs,
            variant.ident.clone().into(),
            parent_attribute.casing(),
        );
        let sub_name = attrs.cased_name();
        let variant_name = &variant.ident;
        let constructor_block = match variant.fields {
            Named(ref fields) => gen_constructor(&fields.named, &attrs),
            Unit => quote!(),
            Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0];
                quote!( ( <#ty as ::structopt::StructOpt>::from_clap(matches) ) )
            }
            Unnamed(..) => call_site_error!("{}: tuple enums are not supported", variant.ident),
        };

        quote! {
            (#sub_name, Some(matches)) =>
                Some(#name :: #variant_name #constructor_block)
        }
    });

    quote! {
        pub fn from_subcommand<'a, 'b>(
            sub: (&'b str, Option<&'b ::structopt::clap::ArgMatches<'a>>)
        ) -> Option<Self> {
            match sub {
                #( #match_arms ),*,
                _ => None
            }
        }
    }
}

#[cfg(feature = "paw")]
fn gen_paw_impl(name: &Ident) -> TokenStream {
    quote! {
        impl paw::ParseArgs for #name {
            type Error = std::io::Error;

            fn parse_args() -> std::result::Result<Self, Self::Error> {
                Ok(<#name as ::structopt::StructOpt>::from_args())
            }
        }
    }
}
#[cfg(not(feature = "paw"))]
fn gen_paw_impl(_: &Ident) -> TokenStream {
    TokenStream::new()
}

fn impl_structopt_for_struct(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
    attrs: &[Attribute],
) -> TokenStream {
    let basic_clap_app_gen = gen_clap_struct(attrs);
    let augment_clap = gen_augment_clap(fields, &basic_clap_app_gen.attrs);
    let from_clap = gen_from_clap(name, fields, &basic_clap_app_gen.attrs);
    let paw_impl = gen_paw_impl(name);

    let clap_tokens = basic_clap_app_gen.tokens;
    quote! {
        #[allow(unused_variables)]
        impl ::structopt::StructOpt for #name {
            #clap_tokens
            #from_clap
        }

        #[allow(dead_code, unreachable_code)]
        #[doc(hidden)]
        impl #name {
            #augment_clap
            pub fn is_subcommand() -> bool { false }
        }

        #paw_impl
    }
}

fn impl_structopt_for_enum(
    name: &Ident,
    variants: &Punctuated<Variant, Comma>,
    attrs: &[Attribute],
) -> TokenStream {
    let basic_clap_app_gen = gen_clap_enum(attrs);

    let augment_clap = gen_augment_clap_enum(variants, &basic_clap_app_gen.attrs);
    let from_clap = gen_from_clap_enum(name);
    let from_subcommand = gen_from_subcommand(name, variants, &basic_clap_app_gen.attrs);
    let paw_impl = gen_paw_impl(name);

    let clap_tokens = basic_clap_app_gen.tokens;
    quote! {
        impl ::structopt::StructOpt for #name {
            #clap_tokens
            #from_clap
        }

        #[allow(unused_variables, dead_code, unreachable_code)]
        #[doc(hidden)]
        impl #name {
            #augment_clap
            #from_subcommand
            pub fn is_subcommand() -> bool { true }
        }

        #paw_impl
    }
}

fn impl_structopt(input: &DeriveInput) -> TokenStream {
    use syn::Data::*;

    let struct_name = &input.ident;

    set_dummy(Some(quote! {
        impl ::structopt::StructOpt for #struct_name {
            fn clap<'a, 'b>() -> ::structopt::clap::App<'a, 'b> {
                unimplemented!()
            }
            fn from_clap(_matches: &::structopt::clap::ArgMatches) -> Self {
                unimplemented!()
            }
        }
    }));

    match input.data {
        Struct(DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => impl_structopt_for_struct(struct_name, &fields.named, &input.attrs),
        Enum(ref e) => impl_structopt_for_enum(struct_name, &e.variants, &input.attrs),
        _ => call_site_error!("structopt only supports non-tuple structs and enums"),
    }
}
