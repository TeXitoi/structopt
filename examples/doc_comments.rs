// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example for the usage of doc comments as replacement
/// of the arguments `help`, `long_help`, `about` and `long_about`.
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Just use doc comments to replace `help`, `long_help`,
    /// `about` or `long_about` input.
    #[structopt(short = "f", long = "first-flag")]
    first_flag: bool,

    /// Split between `help` and `long_help`.
    ///
    /// In the previous case structopt is going to present
    /// the whole comment both as text for the `help` and the
    /// `long_help` argument.
    ///
    /// But if the doc comment is formatted like this example
    /// -- with an empty second line splitting the heading and
    /// the rest of the comment -- only the first line is used
    /// as `help` argument. The `long_help` argument will still
    /// contain the whole comment.
    ///
    /// ## Attention
    ///
    /// Any formatting next to empty lines that could be used
    /// inside a doc comment is currently not preserved. If
    /// lists or other well formatted content is required it is
    /// necessary to use the related structopt argument with a
    /// raw string as shown on the `third_flag` description.
    #[structopt(short = "s", long = "second-flag")]
    second_flag: bool,

    #[structopt(
        short = "t",
        long = "third-flag",
        long_help = r"This is a raw string.

It can be used to pass well formatted content (e.g. lists or source
code) in the description:

 - first example list entry
 - second example list entry
 "
    )]
    third_flag: bool,

    #[structopt(subcommand)]
    sub_command: SubCommand,
}

#[derive(StructOpt, Debug)]
#[structopt()]
enum SubCommand {
    /// The same rules described previously for flags. Are
    /// also true for in regards of sub-commands.
    #[structopt(name = "first")]
    First,

    /// Applicable for both `about` an `help`.
    ///
    /// The formatting rules described in the comment of the
    /// `second_flag` also apply to the description of
    /// sub-commands which is normally given through the `about`
    /// and `long_about` arguments.
    #[structopt(name = "second")]
    Second,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
