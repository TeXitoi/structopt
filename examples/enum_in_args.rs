#[macro_use]
extern crate structopt;
#[macro_use]
extern crate clap;

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
    #[structopt(raw(
        possible_values = "&Baz::variants()",
        case_insensitive = "true"
    ))]
    i: Baz,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
