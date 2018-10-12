#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[test]
fn test_single_word_enum_variant_is_renamed() {
    #[derive(StructOpt, Debug, PartialEq)]
    enum Opt {
        #[structopt()]
        Command { foo: u32 },
    }

    assert_eq!(
        Opt::Command { foo: 0 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "command", "0"]))
    );
}

#[test]
fn test_multi_word_enum_variant_is_renamed() {
    #[derive(StructOpt, Debug, PartialEq)]
    enum Opt {
        #[structopt()]
        FirstCommand { foo: u32 },
    }

    assert_eq!(
        Opt::FirstCommand { foo: 0 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "first-command", "0"]))
    );
}

#[test]
fn test_standalone_long_generates_kebab_case() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(long)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foo-option"]))
    );
}

#[test]
fn test_custom_long_overwrites_default_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(long = "foo")]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foo"]))
    );
}

#[test]
fn test_standalone_long_uses_previous_defined_custom_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(name = "foo", long)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foo"]))
    );
}

#[test]
fn test_standalone_long_ignores_afterwards_defined_custom_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(long, name = "foo")]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--foo-option"]))
    );
}
