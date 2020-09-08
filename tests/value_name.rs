// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod utils;

use structopt::StructOpt;
use utils::*;

#[test]
fn test_multiple_identical_value_names() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(long, value_name = "NUM")]
        num1: u32,
        #[structopt(long, value_name = "NUM")]
        num2: u32,
    }

    let help = get_long_help::<Opt>();
    assert!(help.contains("--num1 <NUM>"));
    assert!(help.contains("--num2 <NUM>"));
}

#[test]
fn test_name_and_value_names() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(long, name = "not_num", value_name = "NUM")]
        num: u32,
    }

    let help = get_long_help::<Opt>();
    assert!(help.contains("--num <NUM>"));
}

#[test]
fn test_only_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(long, name = "not_num")]
        num: u32,
    }

    let help = get_long_help::<Opt>();
    assert!(help.contains("--num <not_num>"));
}
