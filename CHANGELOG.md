# v0.3.1 (2019-09-06)

* Fix error messages ([#241](https://github.com/TeXitoi/structopt/issues/241))
* Fix "`skip` plus long doc comment" bug ([#245](https://github.com/TeXitoi/structopt/issues/245))
* Now `structopt` emits dummy `StructOpt` implementation along with an error. It suppresses
  meaningless errors like `from_args method is not found for Opt`
* `.version()` not get generated if `CARGO_PKG_VERSION` is not set anymore.

# v0.3.0 (2019-08-30)

## Breaking changes

### Bump minimum rustc version to 1.36 by [@TeXitoi](https://github.com/TeXitoi)
Now `rustc` 1.36 is the minimum compiler version supported by `structopt`,
it likely won't work with older compilers.

### Remove "nightly" feature
Once upon a time this feature had been used to enable some of improvements
in `proc-macro2` crate that were available only on nightly. Nowadays this feature doesn't
mean anything so it's now removed.

### Support optional vectors of arguments for distinguishing between `-o 1 2`, `-o` and no option provided at all by [@sphynx](https://github.com/sphynx) ([#180](https://github.com/TeXitoi/structopt/issues/188)).

```rust
#[derive(StructOpt)]
struct Opt {
  #[structopt(long)]
  fruit: Option<Vec<String>>,
}

fn main() {
  assert_eq!(Opt::from_args(&["test"]), None);
  assert_eq!(Opt::from_args(&["test", "--fruit"]), Some(vec![]));
  assert_eq!(Opt::from_args(&["test", "--fruit=apple orange"]), Some(vec!["apple", "orange"]));
}
```

If you need to fall back to the old behavior you can use a type alias:
```rust
type Something = Vec<String>;

#[derive(StructOpt)]
struct Opt {
  #[structopt(long)]
  fruit: Option<Something>,
}
```

### Change default case from 'Verbatim' into 'Kebab' by [@0ndorio](https://github.com/0ndorio) ([#202](https://github.com/TeXitoi/structopt/issues/202)).
`structopt` 0.3 uses field renaming to deduce a name for long options and subcommands.

```rust
#[derive(StructOpt)]
struct Opt {
  #[structopt(long)]
  http_addr: String, // will be renamed to `--http-addr`

  #[structopt(subcommand)]
  addr_type: AddrType // this adds `addr-type` subcommand
}
```

`structopt` 0.2 used to leave things "as is", not renaming anything. If you want to keep old
behavior add `#[structopt(rename_all = "verbatim")]` on top of a `struct`/`enum`.

### Change `version`, `author` and `about` attributes behavior.
Proposed by [@TeXitoi](https://github.com/TeXitoi) [(#217)](https://github.com/TeXitoi/structopt/issues/217), implemented by [@CreepySkeleton](https://github.com/CreepySkeleton) [(#229)](https://github.com/TeXitoi/structopt/pull/229).

`structopt` have been deducing `version`, `author`, and `about` properties from `Cargo.toml`
for a long time (more accurately, from `CARGO_PKG_...` environment variables).
But many users found this behavior somewhat confusing, and a hack was added to cancel out
this behavior: `#[structopt(author = "")]`.

In `structopt` 0.3 this has changed.
* `author` and `about` are no longer deduced by default. You should use `#[structopt(author, about)]`
  to explicitly request `structopt` to deduce them.
* Contrary, `version` **is still deduced by default**. You can use `#[structopt(no_version)]` to
  cancel it out.
* `#[structopt(author = "", about = "", version = "")]` is no longer a valid syntax
  and will trigger an error.
* `#[structopt(version = "version", author = "author", about = "about")]` syntax
  stays unaffected by this changes.

### Raw attributes are removed ([#198](https://github.com/TeXitoi/structopt/pull/198)) by [@sphynx](https://github.com/sphynx)
In `structopt` 0.2 you were able to use any method from `clap::App` and `clap::Arg` via
raw attribute: `#[structopt(raw(method_name = "arg"))]`. This syntax was kind of awkward.

```rust
#[derive(StructOpt, Debug)]
#[structopt(raw(
    global_settings = "&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]"
))]
struct Opt {
    #[structopt(short = "l", long = "level", raw(aliases = r#"&["set-level", "lvl"]"#))]
    level: Vec<String>,
}
```

Raw attributes were removed in 0.3. Now you can use any method from `App` and `Arg` *directly*:
```rust
#[derive(StructOpt)]
#[structopt(global_settings(&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]))]
struct Opt {
    #[structopt(short = "l", long = "level", aliases(&["set-level", "lvl"]))]
    level: Vec<String>,
}
```

## Improvements

### Support skipping struct fields
Proposed by [@Morganamilo](https://github.com/Morganamilo) in ([#174](https://github.com/TeXitoi/structopt/issues/174))
implemented by [@sphynx](https://github.com/sphynx) in ([#213](https://github.com/TeXitoi/structopt/issues/213)).

Sometimes you want to include some fields in your `StructOpt` `struct` that are not options
and `clap` should know nothing about them. In `structopt` 0.3 it's possible via the
`#[structopt(skip)]` attribute. The field in question will be assigned with `Default::default()`
value.

```rust
#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    speed: f32,

    car: String,

    // this field should not generate any arguments
    #[structopt(skip)]
    meta: Vec<u64>
}
```

### Add optional feature to support `paw` by [@gameldar](https://github.com/gameldar) ([#187](https://github.com/TeXitoi/structopt/issues/187))

### Significantly improve error reporting by [@CreepySkeleton](https://github.com/CreepySkeleton) ([#225](https://github.com/TeXitoi/structopt/pull/225/))
Now (almost) every error message points to the location it originates from:

```text
error: default_value is meaningless for bool
  --> $DIR/bool_default_value.rs:14:24
   |
14 |     #[structopt(short, default_value = true)]
   |                        ^^^^^^^^^^^^^
```

# v0.2.16 (2019-05-29)

### Support optional options with optional argument, allowing `cmd [--opt[=value]]` by [@sphynx](https://github.com/sphynx) ([#188](https://github.com/TeXitoi/structopt/issues/188))
Sometimes you want to represent an optional option that optionally takes an argument,
i.e `[--opt[=value]]`. This is represented by `Option<Option<T>>`

```rust
#[derive(StructOpt)]
struct Opt {
  #[structopt(long)]
  fruit: Option<Option<String>>,
}

fn main() {
  assert_eq!(Opt::from_args(&["test"]), None);
  assert_eq!(Opt::from_args(&["test", "--fruit"]), Some(None));
  assert_eq!(Opt::from_args(&["test", "--fruit=apple"]), Some("apple"));
}
```

# v0.2.15 (2019-03-08)

* Fix [#168](https://github.com/TeXitoi/structopt/issues/168) by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.14 (2018-12-10)

* Introduce smarter parsing of doc comments by [@0ndorio](https://github.com/0ndorio)

# v0.2.13 (2018-11-01)

* Automatic naming of fields and subcommands by [@0ndorio](https://github.com/0ndorio)

# v0.2.12 (2018-10-11)

* Fix minimal clap version by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.11 (2018-10-05)

* Upgrade syn to 0.15 by [@konstin](https://github.com/konstin)

# v0.2.10 (2018-06-07)

* 1.21.0 is the minimum required rustc version by
  [@TeXitoi](https://github.com/TeXitoi)

# v0.2.9 (2018-06-05)

* Fix a bug when using `flatten` by
  [@fbenkstein](https://github.com/fbenkstein)
* Update syn, quote and proc_macro2 by
  [@TeXitoi](https://github.com/TeXitoi)
* Fix a regression when there is multiple authors by
  [@windwardly](https://github.com/windwardly)

# v0.2.8 (2018-04-28)

* Add `StructOpt::from_iter_safe()`, which returns an `Error` instead of
  killing the program when it fails to parse, or parses one of the
  short-circuiting flags. ([#98](https://github.com/TeXitoi/structopt/pull/98)
  by [@quodlibetor](https://github.com/quodlibetor))
* Allow users to enable `clap` features independently by
  [@Kerollmops](https://github.com/Kerollmops)
* Fix a bug when flattening an enum
  ([#103](https://github.com/TeXitoi/structopt/pull/103) by
  [@TeXitoi](https://github.com/TeXitoi)

# v0.2.7 (2018-04-12)

* Add flattening, the insertion of options of another StructOpt struct
  into another ([#92](https://github.com/TeXitoi/structopt/pull/92))
  by [@birkenfeld](https://github.com/birkenfeld)
* Fail compilation when using `default_value` or `required` with
  `Option` ([#88](https://github.com/TeXitoi/structopt/pull/88)) by
  [@Kerollmops](https://github.com/Kerollmops)

# v0.2.6 (2018-03-31)

* Fail compilation when using `default_value` or `required` with `bool` ([#80](https://github.com/TeXitoi/structopt/issues/80)) by [@TeXitoi](https://github.com/TeXitoi)
* Fix compilation with `#[deny(warnings)]` with the `!` type (https://github.com/rust-lang/rust/pull/49039#issuecomment-376398999) by [@TeXitoi](https://github.com/TeXitoi)
* Improve first example in the documentation ([#82](https://github.com/TeXitoi/structopt/issues/82)) by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.5 (2018-03-07)

* Work around breakage when `proc-macro2`'s nightly feature is enabled. ([#77](https://github.com/Texitoi/structopt/pull/77) and [proc-macro2#67](https://github.com/alexcrichton/proc-macro2/issues/67)) by [@fitzgen](https://github.com/fitzgen)

# v0.2.4 (2018-02-25)

* Fix compilation with `#![deny(missig_docs]` ([#74](https://github.com/TeXitoi/structopt/issues/74)) by [@TeXitoi](https://github.com/TeXitoi)
* Fix [#76](https://github.com/TeXitoi/structopt/issues/76) by [@TeXitoi](https://github.com/TeXitoi)
* Re-licensed to Apache-2.0/MIT by [@CAD97](https://github.com/cad97)

# v0.2.3 (2018-02-16)

* An empty line in a doc comment will result in a double linefeed in the generated about/help call by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.2 (2018-02-12)

* Fix [#66](https://github.com/TeXitoi/structopt/issues/66) by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.1 (2018-02-11)

* Fix a bug around enum tuple and the about message in the global help by [@TeXitoi](https://github.com/TeXitoi)
* Fix [#65](https://github.com/TeXitoi/structopt/issues/65) by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.0 (2018-02-10)

## Breaking changes

### Don't special case `u64` by [@SergioBenitez](https://github.com/SergioBenitez)

If you are using a `u64` in your struct to get the number of occurence of a flag, you should now add `parse(from_occurrences)` on the flag.

For example
```rust
#[structopt(short = "v", long = "verbose")]
verbose: u64,
```
must be changed by
```rust
#[structopt(short = "v", long = "verbose", parse(from_occurrences))]
verbose: u64,
```

This feature was surprising as shown in [#30](https://github.com/TeXitoi/structopt/issues/30). Using the `parse` feature seems much more natural.

### Change the signature of `Structopt::from_clap` to take its argument by reference by [@TeXitoi](https://github.com/TeXitoi)

There was no reason to take the argument by value. Most of the StructOpt users will not be impacted by this change. If you are using `StructOpt::from_clap`, just add a `&` before the argument.

### Fail if attributes are not used by [@TeXitoi](https://github.com/TeXitoi)

StructOpt was quite fuzzy in its attribute parsing: it was only searching for interresting things, e. g. something like `#[structopt(foo(bar))]` was accepted but not used. It now fails the compilation.

You should have nothing to do here. This breaking change may highlight some missuse that can be bugs.

In future versions, if there is cases that are not highlighed, they will be considerated as bugs, not breaking changes.

### Use `raw()` wrapping instead of `_raw` suffixing by [@TeXitoi](https://github.com/TeXitoi)

The syntax of raw attributes is changed to improve the syntax.

You have to change `foo_raw = "bar", baz_raw = "foo"` by `raw(foo = "bar", baz = "foo")` or `raw(foo = "bar"), raw(baz = "foo")`.

## New features

* Add `parse(from_occurrences)` parser by [@SergioBenitez](https://github.com/SergioBenitez)
* Support 1-uple enum variant as subcommand by [@TeXitoi](https://github.com/TeXitoi)
* structopt-derive crate is now an implementation detail, structopt reexport the custom derive macro by [@TeXitoi](https://github.com/TeXitoi)
* Add the `StructOpt::from_iter` method by [@Kerollmops](https://github.com/Kerollmops)

## Documentation

* Improve doc by [@bestouff](https://github.com/bestouff)
* All the documentation is now on the structopt crate by [@TeXitoi](https://github.com/TeXitoi)

# v0.1.7 (2018-01-23)

* Allow opting out of clap default features by [@ski-csis](https://github.com/ski-csis)

# v0.1.6 (2017-11-25)

* Improve documentation by [@TeXitoi](https://github.com/TeXitoi)
* Fix bug [#31](https://github.com/TeXitoi/structopt/issues/31) by [@TeXitoi](https://github.com/TeXitoi)

# v0.1.5 (2017-11-14)

* Fix a bug with optional subsubcommand and Enum by [@TeXitoi](https://github.com/TeXitoi)

# v0.1.4 (2017-11-09)

* Implement custom string parser from either `&str` or `&OsStr` by [@kennytm](https://github.com/kennytm)

# v0.1.3 (2017-11-01)

* Improve doc by [@TeXitoi](https://github.com/TeXitoi)

# v0.1.2 (2017-11-01)

* Fix bugs [#24](https://github.com/TeXitoi/structopt/issues/24) and [#25](https://github.com/TeXitoi/structopt/issues/25) by [@TeXitoi](https://github.com/TeXitoi)
* Support of methods with something else that a string as argument thanks to `_raw` suffix by [@Flakebi](https://github.com/Flakebi)

# v0.1.1 (2017-09-22)

* Better formating of multiple authors by [@killercup](https://github.com/killercup)

# v0.1.0 (2017-07-17)

* Subcommand support by [@williamyaoh](https://github.com/williamyaoh)

# v0.0.5 (2017-06-16)

* Using doc comment to populate help by [@killercup](https://github.com/killercup)

# v0.0.3 (2017-02-11)

* First version with flags, arguments and options support by [@TeXitoi](https://github.com/TeXitoi)
