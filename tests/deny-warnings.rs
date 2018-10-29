// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(warnings)]
#![cfg(feature = "nightly")] // TODO: remove that when never is stable
#![feature(never_type)]

extern crate structopt;

use structopt::StructOpt;

fn try_str(s: &str) -> Result<String, !> {
    Ok(s.into())
}

#[test]
fn warning_never_struct() {
    #[derive(Debug, PartialEq, StructOpt)]
    struct Opt {
        #[structopt(parse(try_from_str = "try_str"))]
        s: String,
    }
    assert_eq!(
        Opt {
            s: "foo".to_string()
        },
        Opt::from_iter(&["test", "foo"])
    );
}

#[test]
fn warning_never_enum() {
    #[derive(Debug, PartialEq, StructOpt)]
    enum Opt {
        Foo {
            #[structopt(parse(try_from_str = "try_str"))]
            s: String,
        },
    }
    assert_eq!(
        Opt::Foo {
            s: "foo".to_string()
        },
        Opt::from_iter(&["test", "Foo", "foo"])
    );
}
