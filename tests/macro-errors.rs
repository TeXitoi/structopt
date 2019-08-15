// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed

extern crate version_check;
use version_check::Version;

#[test]
fn ui() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/ui-common/*.rs");

    let version = Version::read().unwrap();
    if version.at_least("1.39.0") {
        t.compile_fail("tests/ui-1.39_post/*.rs");
    } else {
        t.compile_fail("tests/ui-pre_1.39/*.rs");
    }
}
