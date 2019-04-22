//! current `clap::arg_enum!` uses "non-ident macro path" feature, which was stabilized in
//! Rust 1.31.0.

extern crate clap;
extern crate structopt;

use clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    enum Baz {
        Foo,
        Bar,
        FooBar
    }
}

#[derive(StructOpt, Debug)]
struct Opt {
    /// Important argument.
    #[structopt(raw(possible_values = "&Baz::variants()", case_insensitive = "true"))]
    i: Baz,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
