use structopt::StructOpt;

mod utils;
use utils::*;

#[test]
fn invisible_group_issue_439() {
    macro_rules! m {
        ($bool:ty) => {
            #[derive(Debug, StructOpt)]
            struct Opts {
                #[structopt(long = "x")]
                x: $bool
            }
        };
    }

    m!(bool);

    let help = get_long_help::<Opts>();

    assert!(help.contains("--x"));
    assert!(!help.contains("--x <x>"));
    Opts::from_iter_safe(&["test", "--x"]).unwrap();
}
