// Copyright (c) 2017 structopt Developers
//
// This work is free. You can redistribute it and/or modify it under
// the terms of the Do What The Fuck You Want To Public License,
// Version 2, as published by Sam Hocevar. See the COPYING file for
// more details.

extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;

use std::path::PathBuf;
use std::num::ParseIntError;
use std::ffi::{OsStr, OsString};

#[derive(StructOpt, PartialEq, Debug)]
struct PathOpt {
    #[structopt(short = "p", long = "path", parse(from_os_str))]
    path: PathBuf,

    #[structopt(short = "d", default_value = "../", parse(from_os_str))]
    default_path: PathBuf,

    #[structopt(short = "v", parse(from_os_str))]
    vector_path: Vec<PathBuf>,

    #[structopt(short = "o", parse(from_os_str))]
    option_path_1: Option<PathBuf>,

    #[structopt(short = "q", parse(from_os_str))]
    option_path_2: Option<PathBuf>,
}

#[test]
fn test_path_opt_simple() {
    assert_eq!(
        PathOpt {
            path: PathBuf::from("/usr/bin"),
            default_path: PathBuf::from("../"),
            vector_path: vec![
                PathBuf::from("/a/b/c"),
                PathBuf::from("/d/e/f"),
                PathBuf::from("/g/h/i"),
            ],
            option_path_1: None,
            option_path_2: Some(PathBuf::from("j.zip"))
        },
        PathOpt::from_clap(PathOpt::clap().get_matches_from(&[
            "test",
            "-p", "/usr/bin",
            "-v", "/a/b/c",
            "-v", "/d/e/f",
            "-v", "/g/h/i",
            "-q", "j.zip",
        ]))
    );
}




fn parse_hex(input: &str) -> Result<u64, ParseIntError> {
    u64::from_str_radix(input, 16)
}

#[derive(StructOpt, PartialEq, Debug)]
struct HexOpt {
    #[structopt(short = "n", parse(try_from_str = "parse_hex"))]
    number: u64,
}

#[test]
fn test_parse_hex() {
    assert_eq!(
        HexOpt { number: 5 },
        HexOpt::from_clap(HexOpt::clap().get_matches_from(&["test", "-n", "5"]))
    );
    assert_eq!(
        HexOpt { number: 0xabcdef },
        HexOpt::from_clap(HexOpt::clap().get_matches_from(&["test", "-n", "abcdef"]))
    );

    let err = HexOpt::clap().get_matches_from_safe(&["test", "-n", "gg"]).unwrap_err();
    assert!(err.message.contains("invalid digit found in string"), err);
}




fn custom_parser_1(_: &str) -> &'static str {
    "A"
}
fn custom_parser_2(_: &str) -> Result<&'static str, u32> {
    Ok("B")
}
fn custom_parser_3(_: &OsStr) -> &'static str {
    "C"
}
fn custom_parser_4(_: &OsStr) -> Result<&'static str, OsString> {
    Ok("D")
}

#[derive(StructOpt, PartialEq, Debug)]
struct NoOpOpt {
    #[structopt(short = "a", parse(from_str = "custom_parser_1"))]
    a: &'static str,
    #[structopt(short = "b", parse(try_from_str = "custom_parser_2"))]
    b: &'static str,
    #[structopt(short = "c", parse(from_os_str = "custom_parser_3"))]
    c: &'static str,
    #[structopt(short = "d", parse(try_from_os_str = "custom_parser_4"))]
    d: &'static str,
}

#[test]
fn test_every_custom_parser() {
    assert_eq!(
        NoOpOpt { a: "A", b: "B", c: "C", d: "D" },
        NoOpOpt::from_clap(NoOpOpt::clap().get_matches_from(&[
            "test", "-a=?", "-b=?", "-c=?", "-d=?",
        ]))
    );
}