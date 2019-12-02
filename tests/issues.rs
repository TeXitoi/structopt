// https://github.com/TeXitoi/structopt/issues/151
// https://github.com/TeXitoi/structopt/issues/289

#[test]
fn issue_151() {
    use structopt::{clap::ArgGroup, StructOpt};

    #[derive(StructOpt, Debug)]
    #[structopt(group = ArgGroup::with_name("verb").required(true).multiple(true))]
    struct Opt {
        #[structopt(long, group = "verb")]
        foo: bool,
        #[structopt(long, group = "verb")]
        bar: bool,
    }

    #[derive(Debug, StructOpt)]
    struct Cli {
        #[structopt(flatten)]
        a: Opt,
    }

    assert!(Cli::clap().get_matches_from_safe(&["test"]).is_err());
    assert!(Cli::clap()
        .get_matches_from_safe(&["test", "--foo"])
        .is_ok());
    assert!(Cli::clap()
        .get_matches_from_safe(&["test", "--bar"])
        .is_ok());
    assert!(Cli::clap()
        .get_matches_from_safe(&["test", "--zebra"])
        .is_err());
}

#[test]
fn issue_289() {
    use structopt::{clap::AppSettings, StructOpt};

    #[derive(StructOpt)]
    #[structopt(setting = AppSettings::InferSubcommands)]
    enum Args {
        SomeCommand(SubSubCommand),
        AnotherCommand,
    }

    #[derive(StructOpt)]
    #[structopt(setting = AppSettings::InferSubcommands)]
    enum SubSubCommand {
        TestCommand,
    }

    assert!(Args::clap()
        .get_matches_from_safe(&["test", "some-command", "test-command"])
        .is_ok());
    assert!(Args::clap()
        .get_matches_from_safe(&["test", "some", "test-command"])
        .is_ok());
    assert!(Args::clap()
        .get_matches_from_safe(&["test", "some-command", "test"])
        .is_ok());
    assert!(Args::clap()
        .get_matches_from_safe(&["test", "some", "test"])
        .is_ok());
}
