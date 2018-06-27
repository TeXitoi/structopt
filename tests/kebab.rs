// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[test]
fn no_short_or_long() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(long)]
        arg: i32,
        #[structopt(long)]
        other_arg: i32,
    }
    assert_eq!(
        Opt { arg: 42, other_arg: 37 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--arg", "42",
                                                       "--other-arg", "37"]))
    );
    assert_eq!(
        Opt { arg: 42, other_arg: 37 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--arg=42",
                                                       "--other-arg", "37"]))
    );
    assert!(Opt::clap().get_matches_from_safe(&["test"]).is_err());
    assert!(
        Opt::clap()
            .get_matches_from_safe(&["test", "--arg=42", "--arg=24",
                                     "--other-arg", "37"])
            .is_err()
    );
}
