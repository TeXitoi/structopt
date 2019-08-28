use structopt::StructOpt;

#[derive(StructOpt, Debug, PartialEq)]
#[structopt(name = "a")]
pub struct Opt {
    #[structopt(long, short)]
    number: u32,
    #[structopt(skip)]
    k: Kind,
    #[structopt(skip)]
    v: Vec<u32>,
}

#[derive(Debug, PartialEq)]
#[allow(unused)]
enum Kind {
    A,
    B,
}

impl Default for Kind {
    fn default() -> Self {
        return Kind::B;
    }
}

fn main() {
    assert_eq!(
        Opt::from_iter(&["test", "-n", "10"]),
        Opt {
            number: 10,
            k: Kind::B,
            v: vec![],
        }
    );
}
