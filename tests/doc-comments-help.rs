// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[test]
fn commets_intead_of_actual_help() {
    /// Lorem ipsum
    #[derive(StructOpt, PartialEq, Debug)]
    struct LoremIpsum {
        /// Fooify a bar
        /// and a baz
        #[structopt(short = "f", long = "foo")]
        foo: bool,
    }

    let mut output = Vec::new();
    LoremIpsum::clap().write_long_help(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("Lorem ipsum"));
    assert!(output.contains("Fooify a bar and a baz"));
}

#[test]
fn help_is_better_than_comments() {
    /// Lorem ipsum
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        /// Fooify a bar
        #[structopt(
            short = "f",
            long = "foo",
            help = "DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES"
        )]
        foo: bool,
    }

    let mut output = Vec::new();
    LoremIpsum::clap().write_long_help(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains("Dolor sit amet"));
    assert!(!output.contains("Lorem ipsum"));
    assert!(output.contains("DO NOT PASS A BAR"));
}

#[test]
fn empty_line_in_doc_comment_is_double_linefeed() {
    /// Foo.
    ///
    /// Bar
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(name = "lorem-ipsum", author = "", version = "")]
    struct LoremIpsum {}

    let mut output = Vec::new();
    LoremIpsum::clap().write_long_help(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    println!("{}", output);
    assert!(output.starts_with("lorem-ipsum \nFoo.\n\nBar\n\nUSAGE:"));
}

#[test]
fn splits_flag_doc_comment_between_short_and_long() {
    /// Lorem ipsumclap
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        /// DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES.
        ///
        /// Or something else
        #[structopt(long = "foo")]
        foo: bool,
    }

    let mut app = LoremIpsum::clap();

    let short_help = {
        let mut buffer = Vec::new();
        app.write_help(&mut buffer).ok();

        String::from_utf8(buffer).unwrap()
    };

    let long_help = {
        let mut buffer = Vec::new();
        app.write_long_help(&mut buffer).ok();

        String::from_utf8(buffer).unwrap()
    };

    assert!(short_help.contains("CIRCUMSTANCES"));
    assert!(!short_help.contains("CIRCUMSTANCES."));
    assert!(!short_help.contains("Or something else"));
    assert!(long_help.contains("DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES"));
    assert!(long_help.contains("Or something else"));
}

#[test]
fn splits_subcommand_doc_comment_between_short_and_long() {
    /// Lorem ipsumclap
    #[derive(StructOpt, Debug)]
    #[structopt(name = "lorem-ipsum", about = "Dolor sit amet")]
    struct LoremIpsum {
        #[structopt(subcommand)]
        foo: SubCommand,
    }

    #[derive(StructOpt, Debug)]
    pub enum SubCommand {
        /// DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES
        ///
        /// Or something else
        #[structopt(name = "foo")]
        Foo {
            #[structopt(help = "foo")]
            bars: Vec<String>,
        },
    }

    let app = LoremIpsum::clap();

    let short_help = {
        let mut buffer = Vec::new();
        app.write_help(&mut buffer).ok();

        String::from_utf8(buffer).unwrap()
    };

    let long_help = {
        app.get_matches_from_safe(vec!["test", "foo", "--help"])
            .expect_err("")
            .message
    };

    assert!(!short_help.contains("Or something else"));
    assert!(long_help.contains("DO NOT PASS A BAR UNDER ANY CIRCUMSTANCES"));
    assert!(long_help.contains("Or something else"));
}
