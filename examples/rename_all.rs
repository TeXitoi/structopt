//! Example on how the `rename_all` parameter works.
//!
//! `rename_all` can be used to override the casing style used during argument
//! generation. By default the `verbatim-case` style will be used but there are a wide
//! variety of other styles available.
//!
//! ## Supported styles overview:
//!
//! - **Camel Case**: Indicate word boundaries with uppercase letter, excluding
//!                   the first word.
//! - **Kebab Case**: Keep all letters lowercase and indicate word boundaries
//!                   with hyphens.
//! - **Pascal Case**: Indicate word boundaries with uppercase letter,
//!                    including the first word.
//! - **Screaming Snake Case**: Keep all letters uppercase and indicate word
//!                             boundaries with underscores.
//! - **Snake Case**: Keep all letters lowercase and indicate word boundaries
//!                   with underscores.
//! - **Verbatim**: Use the original attribute name defined in the code.
#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "rename_all", rename_all = "screaming_snake_case")]
enum Opt {
    // This subcommand will be named `FIRST_COMMAND`. As the command doesn't
    // override the initial casing style, ...
    /// A screaming loud first command. Only use if necessary.
    FirstCommand {
        // this flag will be available as `--FOO` and `-F`.
        /// This flag will even scream louder.
        #[structopt(long, short)]
        foo: bool,
    },

    // As we override the casing style for this variant the related subcommand
    // will be named `SecondCommand`.
    /// Not nearly as loud as the first command.
    #[structopt(rename_all = "pascal_case")]
    SecondCommand {
        // We can also override it again on a single field.
        /// Nice quiet flag. No one is annoyed.
        #[structopt(rename_all = "snake_case", long)]
        bar_option: bool,

        // Renaming will not be propagated into subcommand flagged enums. If
        // a non default casing style is required it must be defined on the
        // enum itself.
        #[structopt(subcommand)]
        cmds: Subcommands,

        // or flattened structs.
        #[structopt(flatten)]
        options: BonusOptions,
    },
}

#[derive(StructOpt, Debug)]
enum Subcommands {
    // This one will be available as `FirstSubcommand`.
    FirstSubcommand,
}

#[derive(StructOpt, Debug)]
struct BonusOptions {
    // And this one will be available as `baz_option`.
    #[structopt(long)]
    baz_option: bool,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
