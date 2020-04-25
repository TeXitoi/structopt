use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, default_value = ())]
    b: String,

    #[structopt(short, env = ())]
    a: String,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
