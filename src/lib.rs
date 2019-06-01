// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(missing_docs)]

//! This crate defines the `StructOpt` trait and its custom derive.
//!
//! ## Features
//!
//! If you want to disable all the `clap` features (colors,
//! suggestions, ..) add `default-features = false` to the `structopt`
//! dependency:
//!
//! ```toml
//! [dependencies]
//! structopt = { version = "0.2", default-features = false }
//! ```
//!
//!
//! Support for [`paw`](https://github.com/rust-cli/paw) (the
//! `Command line argument paw-rser abstraction for main`) is disabled
//! by default, but can be enabled in the `structopt` dependency
//! with the feature `paw`:
//!
//! ```toml
//! [dependencies]
//! structopt = { version = "0.2", features = [ "paw" ] }
//! paw = "1.0"
//! ```
//!
//! ## How to `derive(StructOpt)`
//!
//! First, let's look at an example:
//!
//! ```should_panic
//! #[macro_use]
//! extern crate structopt;
//!
//! use std::path::PathBuf;
//! use structopt::StructOpt;
//!
//! #[derive(Debug, StructOpt)]
//! #[structopt(name = "example", about = "An example of StructOpt usage.")]
//! struct Opt {
//!     /// Activate debug mode
//!     #[structopt(short = "d", long = "debug")]
//!     debug: bool,
//!     /// Set speed
//!     #[structopt(short = "s", long = "speed", default_value = "42")]
//!     speed: f64,
//!     /// Input file
//!     #[structopt(parse(from_os_str))]
//!     input: PathBuf,
//!     /// Output file, stdout if not present
//!     #[structopt(parse(from_os_str))]
//!     output: Option<PathBuf>,
//! }
//!
//! fn main() {
//!     let opt = Opt::from_args();
//!     println!("{:?}", opt);
//! }
//! ```
//!
//! So `derive(StructOpt)` tells Rust to generate a command line parser,
//! and the various `structopt` attributes are simply
//! used for additional parameters.
//!
//! First, define a struct, whatever its name.  This structure will
//! correspond to a `clap::App`.  Every method of `clap::App` in the
//! form of `fn function_name(self, &str)` can be used through attributes
//! placed on the struct. In our example above, the `about` attribute
//! will become an `.about("An example of StructOpt usage.")` call on the
//! generated `clap::App`. There are a few attributes that will default
//! if not specified:
//!
//!   - `name`: The binary name displayed in help messages. Defaults
//!      to the crate name given by Cargo.
//!   - `version`: Defaults to the crate version given by Cargo.
//!   - `author`: Defaults to the crate author name given by Cargo.
//!   - `about`: Defaults to the crate description given by Cargo.
//!
//! Methods from `clap::App` that don't take an `&str` can be called by
//! wrapping them in `raw()`, e.g. to activate colored help text:
//!
//! ```
//! #[macro_use]
//! extern crate structopt;
//!
//! use structopt::StructOpt;
//!
//! #[derive(StructOpt, Debug)]
//! #[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
//! struct Opt {
//!     #[structopt(short = "s")]
//!     speed: bool,
//!     #[structopt(short = "d")]
//!     debug: bool,
//! }
//! # fn main() {}
//! ```
//!
//! Then, each field of the struct not marked as a subcommand corresponds
//! to a `clap::Arg`. As with the struct attributes, every method of
//! `clap::Arg` in the form of `fn function_name(self, &str)` can be used
//! through specifying it as an attribute. The `name` attribute can be used
//! to customize the `Arg::with_name()` call (defaults to the field name in
//! kebab-case).
//! For functions that do not take a `&str` as argument, the attribute can be
//! wrapped in `raw()`, e. g. `raw(aliases = r#"&["alias"]"#, next_line_help = "true")`.
//!
//! The type of the field gives the kind of argument:
//!
//! Type                         | Effect                                            | Added method call to `clap::Arg`
//! -----------------------------|---------------------------------------------------|--------------------------------------
//! `bool`                       | `true` if the flag is present                     | `.takes_value(false).multiple(false)`
//! `Option<T: FromStr>`         | optional positional argument or option            | `.takes_value(true).multiple(false)`
//! `Option<Option<T: FromStr>>` | optional option with optional value               | `.takes_value(true).multiple(false).min_values(0).max_values(1)`
//! `Vec<T: FromStr>`            | list of options or the other positional arguments | `.takes_value(true).multiple(true)`
//! `T: FromStr`                 | required option or positional argument            | `.takes_value(true).multiple(false).required(!has_default)`
//!
//! The `FromStr` trait is used to convert the argument to the given
//! type, and the `Arg::validator` method is set to a method using
//! `to_string()` (`FromStr::Err` must implement `std::fmt::Display`).
//! If you would like to use a custom string parser other than `FromStr`, see
//! the [same titled section](#custom-string-parsers) below.
//!
//! Thus, the `speed` argument is generated as:
//!
//! ```
//! # extern crate clap;
//! # fn parse_validator<T>(_: String) -> Result<(), String> { unimplemented!() }
//! # fn main() {
//! clap::Arg::with_name("speed")
//!     .takes_value(true)
//!     .multiple(false)
//!     .required(false)
//!     .validator(parse_validator::<f64>)
//!     .short("s")
//!     .long("speed")
//!     .help("Set speed")
//!     .default_value("42");
//! # }
//! ```
//!
//! ## Specifying argument types
//!
//! There are three types of arguments that can be supplied to each
//! (sub-)command:
//!
//!  - short (e.g. `-h`),
//!  - long (e.g. `--help`)
//!  - and positional.
//!
//! Like clap, structopt defaults to creating positional arguments.
//!
//! If you want to generate a long argument you can specify either
//! `long = $NAME`, or just `long` to get a long flag generated using
//! the field name.  The generated casing style can be modified using
//! the `rename_all` attribute. See the `rename_all` example for more.
//!
//! For short arguments, `short` will use the first letter of the
//! field name by default, but just like the long option it's also
//! possible to use a custom letter through `short = $LETTER`.
//!
//! If an argument is renamed using `name = $NAME` any following call to
//! `short` or `long` will use the new name.
//!
//! ```
//! #[macro_use]
//! extern crate structopt;
//!
//! use structopt::StructOpt;
//!
//! #[derive(StructOpt)]
//! #[structopt(rename_all = "kebab-case")]
//! struct Opt {
//!     /// This option can be specified with something like `--foo-option
//!     /// value` or `--foo-option=value`
//!     #[structopt(long)]
//!     foo_option: String,
//!
//!     /// This option can be specified with something like `-b value` (but
//!     /// not `--bar-option value`).
//!     #[structopt(short)]
//!     bar_option: String,
//!
//!     /// This option can be specified either `--baz value` or `-z value`.
//!     #[structopt(short = "z", long = "baz")]
//!     baz_option: String,
//!
//!     /// This option can be specified either by `--custom value` or
//!     /// `-c value`.
//!     #[structopt(name = "custom", long, short)]
//!     custom_option: String,
//!
//!     /// This option is positional, meaning it is the first unadorned string
//!     /// you provide (multiple others could follow).
//!     my_positional: String,
//! }
//!
//! # fn main() {
//! # Opt::from_clap(&Opt::clap().get_matches_from(
//! #    &["test", "--foo-option", "", "-b", "", "--baz", "", "--custom", "", "positional"]));
//! # }
//! ```
//!
//! ## Help messages
//!
//! Help messages for the whole binary or individual arguments can be
//! specified using the `about` attribute on the struct and the `help`
//! attribute on the field, as we've already seen. For convenience,
//! they can also be specified using doc comments. For example:
//!
//! ```
//! # #[macro_use] extern crate structopt;
//! #[derive(StructOpt)]
//! #[structopt(name = "foo")]
//! /// The help message that will be displayed when passing `--help`.
//! struct Foo {
//!   #[structopt(short = "b")]
//!   /// The description for the arg that will be displayed when passing `--help`.
//!   bar: String
//! }
//! # fn main() {}
//! ```
//!
//! If it is necessary or wanted to provide a more complex help message then the
//! previous used ones, it could still be a good idea to distinguish between the
//! actual help message a short summary. In this case `about` and `help` should
//! only contain the short and concise form while the two additional arguments
//! `long_about` and `long_help` can be used to store a descriptive and more in
//! depth message.
//!
//! If both - the short and the long version of the argument - are present,
//! the user can later chose between the short summary (`-h`) and the long
//! descriptive version (`--help`) of the help message. Also in case
//! of subcommands the short help message will automatically be used for the
//! command description inside the parents help message and the long version
//! as command description if help is requested on the actual subcommand.
//!
//! This feature can also be used with doc comments instead of arguments through
//! proper comment formatting. To be activated it requires, that the first line
//! of the comment is separated from the rest of the comment through an empty line.
//! In this case the first line is used as summary and the whole comment represents
//! the long descriptive message.
//!
//! ```
//! # #[macro_use] extern crate structopt;
//! #[derive(StructOpt)]
//! #[structopt(name = "foo")]
//! /// The help message that will be displayed when passing `--help`.
//! struct Foo {
//!   #[structopt(short = "b")]
//!   /// Only this summary is visible when passing `-h`.
//!   ///
//!   /// But the whole comment will be displayed when passing `--help`.
//!   /// This could be quite useful to provide further hints are usage
//!   /// examples.
//!   bar: String
//! }
//! # fn main() {}
//! ```
//!
//! ## Subcommands
//!
//! Some applications, especially large ones, split their functionality
//! through the use of "subcommands". Each of these act somewhat like a separate
//! command, but is part of the larger group.
//! One example is `git`, which has subcommands such as `add`, `commit`,
//! and `clone`, to mention just a few.
//!
//! `clap` has this functionality, and `structopt` supports it through enums:
//!
//! ```
//! # #[macro_use] extern crate structopt;
//! # use std::path::PathBuf;
//! #[derive(StructOpt)]
//! #[structopt(name = "git", about = "the stupid content tracker")]
//! enum Git {
//!     #[structopt(name = "add")]
//!     Add {
//!         #[structopt(short = "i")]
//!         interactive: bool,
//!         #[structopt(short = "p")]
//!         patch: bool,
//!         #[structopt(parse(from_os_str))]
//!         files: Vec<PathBuf>
//!     },
//!     #[structopt(name = "fetch")]
//!     Fetch {
//!         #[structopt(long = "dry-run")]
//!         dry_run: bool,
//!         #[structopt(long = "all")]
//!         all: bool,
//!         repository: Option<String>
//!     },
//!     #[structopt(name = "commit")]
//!     Commit {
//!         #[structopt(short = "m")]
//!         message: Option<String>,
//!         #[structopt(short = "a")]
//!         all: bool
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! Using `derive(StructOpt)` on an enum instead of a struct will produce
//! a `clap::App` that only takes subcommands. So `git add`, `git fetch`,
//! and `git commit` would be commands allowed for the above example.
//!
//! `structopt` also provides support for applications where certain flags
//! need to apply to all subcommands, as well as nested subcommands:
//!
//! ```
//! # #[macro_use] extern crate structopt;
//! # fn main() {}
//! #[derive(StructOpt)]
//! #[structopt(name = "make-cookie")]
//! struct MakeCookie {
//!     #[structopt(name = "supervisor", default_value = "Puck", long = "supervisor")]
//!     supervising_faerie: String,
//!     #[structopt(name = "tree")]
//!     /// The faerie tree this cookie is being made in.
//!     tree: Option<String>,
//!     #[structopt(subcommand)]  // Note that we mark a field as a subcommand
//!     cmd: Command
//! }
//!
//! #[derive(StructOpt)]
//! enum Command {
//!     #[structopt(name = "pound")]
//!     /// Pound acorns into flour for cookie dough.
//!     Pound {
//!         acorns: u32
//!     },
//!     #[structopt(name = "sparkle")]
//!     /// Add magical sparkles -- the secret ingredient!
//!     Sparkle {
//!         #[structopt(short = "m", parse(from_occurrences))]
//!         magicality: u64,
//!         #[structopt(short = "c")]
//!         color: String
//!     },
//!     #[structopt(name = "finish")]
//!     Finish(Finish),
//! }
//!
//! // Subcommand can also be externalized by using a 1-uple enum variant
//! #[derive(StructOpt)]
//! struct Finish {
//!     #[structopt(short = "t")]
//!     time: u32,
//!     #[structopt(subcommand)]  // Note that we mark a field as a subcommand
//!     finish_type: FinishType
//! }
//!
//! // subsubcommand!
//! #[derive(StructOpt)]
//! enum FinishType {
//!     #[structopt(name = "glaze")]
//!     Glaze {
//!         applications: u32
//!     },
//!     #[structopt(name = "powder")]
//!     Powder {
//!         flavor: String,
//!         dips: u32
//!     }
//! }
//! ```
//!
//! Marking a field with `structopt(subcommand)` will add the subcommands of the
//! designated enum to the current `clap::App`. The designated enum *must* also
//! be derived `StructOpt`. So the above example would take the following
//! commands:
//!
//! + `make-cookie pound 50`
//! + `make-cookie sparkle -mmm --color "green"`
//! + `make-cookie finish 130 glaze 3`
//!
//! ### Optional subcommands
//!
//! A nested subcommand can be marked optional:
//!
//! ```
//! # #[macro_use] extern crate structopt;
//! # fn main() {}
//! #[derive(StructOpt)]
//! #[structopt(name = "foo")]
//! struct Foo {
//!     file: String,
//!     #[structopt(subcommand)]
//!     cmd: Option<Command>
//! }
//!
//! #[derive(StructOpt)]
//! enum Command {
//!     Bar,
//!     Baz,
//!     Quux
//! }
//! ```
//!
//! ## Flattening
//!
//! It can sometimes be useful to group related arguments in a substruct,
//! while keeping the command-line interface flat. In these cases you can mark
//! a field as `flatten` and give it another type that derives `StructOpt`:
//!
//! ```
//! # #[macro_use] extern crate structopt;
//! # use structopt::StructOpt;
//! # fn main() {}
//! #[derive(StructOpt)]
//! struct Cmdline {
//!     #[structopt(short = "v", help = "switch on verbosity")]
//!     verbose: bool,
//!     #[structopt(flatten)]
//!     daemon_opts: DaemonOpts,
//! }
//!
//! #[derive(StructOpt)]
//! struct DaemonOpts {
//!     #[structopt(short = "u", help = "daemon user")]
//!     user: String,
//!     #[structopt(short = "g", help = "daemon group")]
//!     group: String,
//! }
//! ```
//!
//! In this example, the derived `Cmdline` parser will support the options `-v`,
//! `-u` and `-g`.
//!
//! This feature also makes it possible to define a `StructOpt` struct in a
//! library, parse the corresponding arguments in the main argument parser, and
//! pass off this struct to a handler provided by that library.
//!
//! ## Custom string parsers
//!
//! If the field type does not have a `FromStr` implementation, or you would
//! like to provide a custom parsing scheme other than `FromStr`, you may
//! provide a custom string parser using `parse(...)` like this:
//!
//! ```
//! # #[macro_use] extern crate structopt;
//! # fn main() {}
//! use std::num::ParseIntError;
//! use std::path::PathBuf;
//!
//! fn parse_hex(src: &str) -> Result<u32, ParseIntError> {
//!     u32::from_str_radix(src, 16)
//! }
//!
//! #[derive(StructOpt)]
//! struct HexReader {
//!     #[structopt(short = "n", parse(try_from_str = "parse_hex"))]
//!     number: u32,
//!     #[structopt(short = "o", parse(from_os_str))]
//!     output: PathBuf,
//! }
//! ```
//!
//! There are five kinds of custom parsers:
//!
//! | Kind              | Signature                             | Default                         |
//! |-------------------|---------------------------------------|---------------------------------|
//! | `from_str`        | `fn(&str) -> T`                       | `::std::convert::From::from`    |
//! | `try_from_str`    | `fn(&str) -> Result<T, E>`            | `::std::str::FromStr::from_str` |
//! | `from_os_str`     | `fn(&OsStr) -> T`                     | `::std::convert::From::from`    |
//! | `try_from_os_str` | `fn(&OsStr) -> Result<T, OsString>`   | (no default function)           |
//! | `from_occurrences`| `fn(u64) -> T`                        | `value as T`                    |
//!
//! The `from_occurrences` parser is special. Using `parse(from_occurrences)`
//! results in the _number of flags occurrences_ being stored in the relevant
//! field or being passed to the supplied function. In other words, it converts
//! something like `-vvv` to `3`. This is equivalent to
//! `.takes_value(false).multiple(true)`. Note that the default parser can only
//! be used with fields of integer types (`u8`, `usize`, `i64`, etc.).
//!
//! When supplying a custom string parser, `bool` will not be treated specially:
//!
//! Type        | Effect            | Added method call to `clap::Arg`
//! ------------|-------------------|--------------------------------------
//! `Option<T>` | optional argument | `.takes_value(true).multiple(false)`
//! `Vec<T>`    | list of arguments | `.takes_value(true).multiple(true)`
//! `T`         | required argument | `.takes_value(true).multiple(false).required(!has_default)`
//!
//! In the `try_from_*` variants, the function will run twice on valid input:
//! once to validate, and once to parse. Hence, make sure the function is
//! side-effect-free.

extern crate clap as _clap;

#[allow(unused_imports)]
#[macro_use]
extern crate structopt_derive;

#[doc(hidden)]
pub use structopt_derive::*;

use std::ffi::OsString;

/// Re-export of clap
pub mod clap {
    pub use _clap::*;
}

/// A struct that is converted from command line arguments.
pub trait StructOpt {
    /// Returns the corresponding `clap::App`.
    fn clap<'a, 'b>() -> clap::App<'a, 'b>;

    /// Creates the struct from `clap::ArgMatches`.  It cannot fail
    /// with a parameter generated by `clap` by construction.
    fn from_clap(&clap::ArgMatches) -> Self;

    /// Gets the struct from the command line arguments.  Print the
    /// error message and quit the program in case of failure.
    fn from_args() -> Self
    where
        Self: Sized,
    {
        Self::from_clap(&Self::clap().get_matches())
    }

    /// Gets the struct from any iterator such as a `Vec` of your making.
    /// Print the error message and quit the program in case of failure.
    fn from_iter<I>(iter: I) -> Self
    where
        Self: Sized,
        I: IntoIterator,
        I::Item: Into<OsString> + Clone,
    {
        Self::from_clap(&Self::clap().get_matches_from(iter))
    }

    /// Gets the struct from any iterator such as a `Vec` of your making.
    ///
    /// Returns a `clap::Error` in case of failure. This does *not* exit in the
    /// case of `--help` or `--version`, to achieve the same behavior as
    /// `from_iter()` you must call `.exit()` on the error value.
    fn from_iter_safe<I>(iter: I) -> Result<Self, clap::Error>
    where
        Self: Sized,
        I: IntoIterator,
        I::Item: Into<OsString> + Clone,
    {
        Ok(Self::from_clap(&Self::clap().get_matches_from_safe(iter)?))
    }
}
