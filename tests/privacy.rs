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

#[macro_use]
extern crate structopt;

mod options {
    #[derive(Debug, StructOpt)]
    pub struct Options {
	#[structopt(subcommand)]
	pub subcommand: ::subcommands::SubCommand,
    }
}

mod subcommands {
    #[derive(Debug, StructOpt)]
    pub enum SubCommand {
	#[structopt(name = "foo", about = "foo")]
	Foo {
	    #[structopt(help = "foo")]
	    bars: Vec<String>,
	},
    }
}
