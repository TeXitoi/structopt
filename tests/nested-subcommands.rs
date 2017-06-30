// Copyright (c) 2017 Guillaume Pinot <texitoi(a)texitoi.eu>
//
// This work is free. You can redistribute it and/or modify it under
// the terms of the Do What The Fuck You Want To Public License,
// Version 2, as published by Sam Hocevar. See the COPYING file for
// more details.

extern crate structopt;
#[macro_use] extern crate structopt_derive;

use structopt::StructOpt;

#[derive(StructOpt, PartialEq, Debug)]
struct Opt {
    #[structopt(short = "f", long = "force")]
    force: bool,
    #[structopt(short = "v", long = "verbose")]
    verbose: u64,
    #[structopt(subcommand)]
    cmd: Sub
}

#[derive(StructOpt, PartialEq, Debug)]
enum Sub {
    #[structopt(name = "fetch")]
    Fetch {},
    #[structopt(name = "add")]
    Add {}
}

#[derive(StructOpt, PartialEq, Debug)]
struct Opt2 {
    #[structopt(short = "f", long = "force")]
    force: bool,
    #[structopt(short = "v", long = "verbose")]
    verbose: u64,
    #[structopt(subcommand)]
    cmd: Option<Sub>
}

#[test]
fn test_no_cmd() {
    let result = Opt::clap().get_matches_from_safe(&["test"]);
    assert!(result.is_err());

    assert_eq!(Opt2 { force: false, verbose: 0, cmd: None },
               Opt2::from_clap(Opt2::clap().get_matches_from(&["test"])));
}

#[test]
fn test_fetch() {
    assert_eq!(Opt { force: false, verbose: 3, cmd: Sub::Fetch {} },
               Opt::from_clap(Opt::clap().get_matches_from(&["test", "-vvv", "fetch"])));
    assert_eq!(Opt { force: true, verbose: 0, cmd: Sub::Fetch {} },
               Opt::from_clap(Opt::clap().get_matches_from(&["test", "--force", "fetch"])));
}

#[test]
fn test_add() {
    assert_eq!(Opt { force: false, verbose: 0, cmd: Sub::Add {} },
               Opt::from_clap(Opt::clap().get_matches_from(&["test", "add"])));
    assert_eq!(Opt { force: false, verbose: 2, cmd: Sub::Add {} },
               Opt::from_clap(Opt::clap().get_matches_from(&["test", "-vv", "add"])));
}

#[test]
fn test_badinput() {
    let result = Opt::clap().get_matches_from_safe(&["test", "badcmd"]);
    assert!(result.is_err());
    let result = Opt::clap().get_matches_from_safe(&["test", "add", "--verbose"]);
    assert!(result.is_err());
    let result = Opt::clap().get_matches_from_safe(&["test", "--badopt", "add"]);
    assert!(result.is_err());
    let result = Opt::clap().get_matches_from_safe(&["test", "add", "--badopt"]);
    assert!(result.is_err());
}

#[derive(StructOpt, PartialEq, Debug)]
struct Opt3 {
    #[structopt(short = "a", long = "all")]
    all: bool,
    #[structopt(subcommand)]
    cmd: Sub2
}

#[derive(StructOpt, PartialEq, Debug)]
enum Sub2 {
    #[structopt(name = "foo")]
    Foo {
        file: String,
        #[structopt(subcommand)]
        cmd: Sub3
    },
    #[structopt(name = "bar")]
    Bar {
    }
}

#[derive(StructOpt, PartialEq, Debug)]
enum Sub3 {
    #[structopt(name = "baz")]
    Baz {},
    #[structopt(name = "quux")]
    Quux {}
}

#[test]
fn test_subsubcommand() {
    assert_eq!(
        Opt3 {
            all: true,
            cmd: Sub2::Foo { file: "lib.rs".to_string(), cmd: Sub3::Quux {} }
        },
        Opt3::from_clap(Opt3::clap().get_matches_from(&["test", "--all", "foo", "lib.rs", "quux"]))
    );
}
