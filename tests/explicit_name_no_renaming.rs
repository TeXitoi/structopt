use structopt::StructOpt;

#[test]
fn explicit_short_long_no_rename() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short = ".", long = ".foo")]
        foo: Vec<String>,
    }

    assert_eq!(
        Opt {
            foo: vec!["short".into(), "long".into()]
        },
        Opt::from_iter(&["test", "-.", "short", "--.foo", "long"])
    );
}

#[test]
fn explicit_name_no_rename() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(name = ".options")]
        foo: Vec<String>,
    }

    let mut output = Vec::new();
    Opt::clap().write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("[.options]..."))
}
