use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, default_value = ())]
    b: String,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
