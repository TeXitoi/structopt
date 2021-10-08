//! How to parse "key=value" pairs with structopt.

use std::error::Error;
use structopt::StructOpt;

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
    U: std::str::FromStr,
    U::Err: Error + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(StructOpt, Debug)]
struct Opt {
    // We can use -D option in multiple cases :
    
    // The most basic:
    // my_program -D a=1
    
    // With multiple values
    // my_program -D a=1 b=2
    
    // With multiple calls to the -D option
    // my_program -D a=1 -D b=2
    
    // All of these are valid. Note that this is just examples of what is correct.
    #[structopt(short = "D", parse(try_from_str = parse_key_val))]
    defines: Vec<(String, i32)>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
