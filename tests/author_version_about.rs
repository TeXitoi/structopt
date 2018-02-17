// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use] extern crate structopt;

use structopt::StructOpt;

#[test]
fn no_author_version_about() {
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(name = "foo", about = "", author = "", version = "")]
    struct Opt {}

    let mut output = Vec::new();
    Opt::clap().write_long_help(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.starts_with("foo \n\nUSAGE:"));
}

#[test]
fn use_env() {
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt()]
    struct Opt {}

    let mut output = Vec::new();
    Opt::clap().write_long_help(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.starts_with("structopt 0.2."));
    assert!(output.contains("Guillaume Pinot <texitoi@texitoi.eu>"));
    assert!(output.contains("Parse command line argument by defining a struct."));
}
