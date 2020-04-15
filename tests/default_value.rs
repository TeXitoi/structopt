use structopt::StructOpt;

mod utils;

use utils::*;

#[test]
fn auto_default_value() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(default_value)]
        arg: i32,
    }
    assert_eq!(Opt { arg: 0 }, Opt::from_iter(&["test"]));
    assert_eq!(Opt { arg: 1 }, Opt::from_iter(&["test", "1"]));

    let help = get_long_help::<Opt>();
    assert!(help.contains("[default: 0]"));
}

#[test]
fn option_with_default_value() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(long, default_value)]
        arg1: Option<i32>,
        #[structopt(long, default_value = "17", env)]
        arg2: Option<i32>,
    }
    assert_eq!(
        Opt {
            arg1: None,
            arg2: None
        },
        Opt::from_iter(&["test"])
    );
    assert_eq!(
        Opt {
            arg1: Some(0),
            arg2: Some(17)
        },
        Opt::from_iter(&["test", "--arg1", "--arg2"])
    );
    std::env::set_var("ARG2", "34");
    assert_eq!(
        Opt {
            arg1: Some(2),
            arg2: Some(34)
        },
        Opt::from_iter(&["test", "--arg1=2"])
    );
    assert_eq!(
        Opt {
            arg1: None,
            arg2: Some(34)
        },
        Opt::from_iter(&["test", "--arg2"])
    );
    assert_eq!(
        Opt {
            arg1: Some(5),
            arg2: Some(42)
        },
        Opt::from_iter(&["test", "--arg1=5", "--arg2=42"])
    );

    let help = get_long_help::<Opt>();
    assert!(help.contains("[default: 0]"));
    assert!(help.contains("[default: 17]"));
}
