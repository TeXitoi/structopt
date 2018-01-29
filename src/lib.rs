// Copyright (c) 2017 Guillaume Pinot <texitoi(a)texitoi.eu>
//
// This work is free. You can redistribute it and/or modify it under
// the terms of the Do What The Fuck You Want To Public License,
// Version 2, as published by Sam Hocevar. See the COPYING file for
// more details.

#![deny(missing_docs)]

//! `StructOpt` trait definition
//!
//! This crate defines the `StructOpt` trait.  Alone, this crate is of
//! little interest.  See the 
//! [`structopt-derive`](https://docs.rs/structopt-derive) crate to
//! automatically generate an implementation of this trait.
//!
//! If you want to disable all the `clap` features (colors,
//! suggestions, ..) add `default-features = false` to the `structopt`
//! dependency:
//! ```toml
//! [dependencies]
//! structopt = { version = "0.1.0", default-features = false }
//! structopt-derive = "0.1.0"
//! ```

extern crate clap as _clap;

#[allow(unused_imports)]
#[macro_use]
extern crate structopt_derive;

#[doc(hidden)]
pub use structopt_derive::*;

/// Re-export of clap
pub mod clap {
    pub use _clap::*;
}

/// A struct that is converted from command line arguments.
pub trait StructOpt {
    /// Returns the corresponding `clap::App`.
    fn clap<'a, 'b>() -> clap::App<'a, 'b>;

    /// Creates the struct from `clap::ArgMatches`.  It cannot fail
    /// with a parameter generated by `clap` by construction.
    fn from_clap(&clap::ArgMatches) -> Self;

    /// Gets the struct from the command line arguments.  Print the
    /// error message and quit the program in case of failure.
    fn from_args() -> Self where Self: Sized {
        Self::from_clap(&Self::clap().get_matches())
    }
}
