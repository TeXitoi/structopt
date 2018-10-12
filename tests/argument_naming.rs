#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[test]
fn test_single_word_enum_variant_is_default_renamed_into_kebab_case() {
    #[derive(StructOpt, Debug, PartialEq)]
    enum Opt {
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
    #[allow(non_snake_case)]
    struct Opt {
        #[structopt(long)]
        FOO_OPTION: bool,
    }

    assert_eq!(
        Opt { FOO_OPTION: true },
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

#[test]
fn test_standalone_short_generates_kebab_case() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[allow(non_snake_case)]
    struct Opt {
        #[structopt(short)]
        FOO_OPTION: bool,
    }

    assert_eq!(
        Opt { FOO_OPTION: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-f"]))
    );
}

#[test]
fn test_custom_short_overwrites_default_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(short = "o")]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-o"]))
    );
}

#[test]
fn test_standalone_short_uses_previous_defined_custom_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(name = "option", short)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-o"]))
    );
}

#[test]
fn test_standalone_short_ignores_afterwards_defined_custom_name() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(short, name = "option")]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-f"]))
    );
}

#[test]
fn test_standalone_long_uses_previous_defined_casing() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(rename_all = "screaming_snake", long)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--FOO_OPTION"]))
    );
}

#[test]
fn test_standalone_short_uses_previous_defined_casing() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(rename_all = "screaming_snake", short)]
        foo_option: bool,
    }

    assert_eq!(
        Opt { foo_option: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-F"]))
    );
}

#[test]
fn test_standalone_long_works_with_verbatim_casing() {
    #[derive(StructOpt, Debug, PartialEq)]
    #[allow(non_snake_case)]
    struct Opt {
        #[structopt(rename_all = "verbatim", long)]
        _fOO_oPtiON: bool,
    }

    assert_eq!(
        Opt { _fOO_oPtiON: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--_fOO_oPtiON"]))
    );
}

#[test]
fn test_standalone_short_works_with_verbatim_casing() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(rename_all = "verbatim", short)]
        _foo: bool,
    }

    assert_eq!(
        Opt { _foo: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-_"]))
    );
}
