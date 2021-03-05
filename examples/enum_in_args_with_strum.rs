use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames, VariantNames};

const DEFAULT: &str = "txt";

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(
        long,
        possible_values = Format::VARIANTS,
        case_insensitive = true,
        default_value = DEFAULT,
    )]
    format: Format,
}

#[derive(EnumString, EnumVariantNames, Debug)]
#[strum(serialize_all = "kebab_case")]
enum Format {
    Txt,
    Md,
    Html,
}

fn main() {
    println!("{:?}", Opt::from_args());
}
