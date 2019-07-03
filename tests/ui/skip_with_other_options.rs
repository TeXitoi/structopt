use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "test")]
pub struct Opts {
    #[structopt(long)]
    a: u32,
    #[structopt(skip, long)]
    b: u32,
}

fn main() {
    let opts = Opts::from_args();
    println!("{:?}", opts);
}
