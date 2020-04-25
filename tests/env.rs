mod utils;

use structopt::StructOpt;
use utils::*;

#[test]
fn env_to_string() {
    std::env::set_var("ENV_VAR_1", "2");

    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(env = format!("{}_{}_{}", "ENV", "VAR", 1))]
        arg: i32,
    }
    assert_eq!(Opt { arg: 2 }, Opt::from_iter(&["test"]));
    assert_eq!(Opt { arg: 1 }, Opt::from_iter(&["test", "1"]));

    let help = get_long_help::<Opt>();
    assert!(help.contains("[env: ENV_VAR_1=2]"));
}
