// Copyright 2019-present structopt developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::clap::Shell;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
/// An example of how to generate bash completions with structopt.
struct Opt {
    #[structopt(short, long)]
    /// Activate debug mode
    debug: bool,
}

fn main() {
    // generate `bash` completions in "target" directory
    Opt::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");

    let opt = Opt::from_args();
    println!("{:?}", opt);
}
