// Copyright 2019-present structopt developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate structopt;

use structopt::clap::Shell;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// A flag, true if used in the command line.
    #[structopt(short = "d", long = "debug", help = "Activate debug mode")]
    debug: bool,
}

fn main() {
    // generate `bash` completions in "target" directory
    Opt::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");

    let opt = Opt::from_args();
    println!("{:?}", opt);
}
