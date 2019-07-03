// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// A flag, true if used in the command line.
    #[structopt(short = "d", long = "debug", help = "Activate debug mode")]
    debug: bool,

    /// An argument of type float, with a default value.
    #[structopt(short = "s", long = "speed", help = "Set speed", default_value = "42")]
    speed: f64,

    /// Needed parameter, the first on the command line.
    #[structopt(help = "Input file")]
    input: String,

    /// An optional parameter, will be `None` if not present on the
    /// command line.
    #[structopt(help = "Output file, stdout if not present")]
    output: Option<String>,

    /// An optional parameter with optional value, will be `None` if
    /// not present on the command line, will be `Some(None)` if no
    /// argument is provided (i.e. `--log`) and will be
    /// `Some(Some(String))` if argument is provided (e.g. `--log
    /// log.txt`).
    #[structopt(
        long = "log",
        help = "Log file, stdout if no file, no logging if not present"
    )]
    #[allow(clippy::option_option)]
    log: Option<Option<String>>,

    /// An optional list of values, will be `None` if not present on
    /// the command line, will be `Some(vec![])` if no argument is
    /// provided (i.e. `--optv`) and will be `Some(Some(String))` if
    /// argument list is provided (e.g. `--optv a b c`).
    #[structopt(long = "optv")]
    optv: Option<Vec<String>>,

    /// Skipped option: it won't be parsed and will be filled with the
    /// default value for its type (in this case '').
    #[structopt(skip)]
    skipped: String,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
