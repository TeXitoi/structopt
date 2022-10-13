use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    _debug: bool,
}

fn main() {
    // Running this example will print this message
    // ----------------------------------------------------
    // basic 0.3.26
    // A basic example
    //
    // USAGE:
    //     basic [FLAGS]
    //
    // FLAGS:
    //     -d, --debug    Activate debug mode
    // ----------------------------------------------------
    println!("{}", <Opt as StructOpt>::gen_help().unwrap());
}
