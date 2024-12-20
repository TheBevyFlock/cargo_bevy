use clap::{ArgAction, Args, Subcommand};

use crate::external_cli::{arg_builder::ArgBuilder, cargo::run::CargoRunArgs};

#[derive(Debug, Args)]
pub struct RunArgs {
    /// The subcommands available for the run command.
    #[command(subcommand)]
    pub subcommand: Option<RunSubcommands>,

    /// Commands to forward to `cargo run`.
    #[clap(flatten)]
    pub cargo_args: CargoRunArgs,
}

impl RunArgs {
    /// Whether to run the app in the browser.
    pub(crate) fn is_web(&self) -> bool {
        matches!(self.subcommand, Some(RunSubcommands::Web(_)))
    }

    /// The profile used to compile the app.
    pub(crate) fn profile(&self) -> &str {
        self.cargo_args.compilation_args.profile()
    }

    /// The targeted platform.
    pub(crate) fn target(&self) -> Option<String> {
        self.cargo_args.compilation_args.target(self.is_web())
    }

    /// Generate arguments for `cargo`.
    pub(crate) fn cargo_args_builder(&self) -> ArgBuilder {
        self.cargo_args.args_builder(self.is_web())
    }
}

#[derive(Debug, Subcommand)]
pub enum RunSubcommands {
    /// Run your app in the browser.
    Web(RunWebArgs),
}

#[derive(Debug, Args)]
pub struct RunWebArgs {
    /// The port to run the web server on.
    #[arg(short, long, default_value_t = 4000)]
    pub port: u16,

    /// Open the app in the browser.
    #[arg(short = 'o', long = "open", action = ArgAction::SetTrue, default_value_t = false)]
    pub open: bool,
}
