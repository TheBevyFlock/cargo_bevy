use std::process::Command;

use clap::{ArgAction, Args};

use crate::external_cli::arg_builder::ArgBuilder;

use super::{program, CargoCompilationArgs, CargoFeatureArgs, CargoManifestArgs};

/// Create a command to run `cargo build`.
pub(crate) fn command() -> Command {
    let mut command = Command::new(program());
    command.arg("build");
    command
}

#[derive(Debug, Args)]
pub struct CargoBuildArgs {
    /// Override a configuration value.
    ///
    /// The argument should be in TOML syntax of KEY=VALUE,
    /// or provided as a path to an extra configuration file.
    /// This flag may be specified multiple times.
    #[clap(long = "config", value_name = "KEY=VALUE|PATH")]
    pub config: Vec<String>,

    #[clap(flatten)]
    pub package_args: CargoPackageBuildArgs,
    #[clap(flatten)]
    pub target_args: CargoTargetBuildArgs,
    #[clap(flatten)]
    pub feature_args: CargoFeatureArgs,
    #[clap(flatten)]
    pub compilation_args: CargoCompilationArgs,
    #[clap(flatten)]
    pub manifest_args: CargoManifestArgs,
}

impl CargoBuildArgs {
    pub(crate) fn args_builder(&self, is_web: bool) -> ArgBuilder {
        ArgBuilder::new()
            .append(self.package_args.args_builder())
            .append(self.target_args.args_builder())
            .append(self.feature_args.args_builder())
            .append(self.compilation_args.args_builder(is_web))
            .append(self.manifest_args.args_builder())
            .add_values_separately("--config", self.config.iter())
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Package Selection")]
pub struct CargoPackageBuildArgs {
    /// Package to build (see `cargo help pkgid`)
    #[clap(short = 'p', long = "package", value_name = "SPEC")]
    pub package: Option<String>,

    /// Build all packages in the workspace
    #[clap(long = "workspace", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_workspace: bool,

    /// Exclude packages from the build
    #[clap(long = "exclude", value_name = "SPEC")]
    pub exclude: Option<String>,
}

impl CargoPackageBuildArgs {
    pub(crate) fn args_builder(&self) -> ArgBuilder {
        ArgBuilder::new()
            .add_opt_value("--package", &self.package)
            .add_flag_if("--workspace", self.is_workspace)
            .add_opt_value("--exclude", &self.exclude)
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Target Selection")]
pub struct CargoTargetBuildArgs {
    /// Build only this package's library
    #[clap(long = "lib", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_lib: bool,

    /// Build all binaries.
    #[clap(long = "bins", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_bins: bool,

    /// Build only the specified binary.
    #[clap(long = "bin", value_name = "NAME")]
    pub bin: Option<String>,

    /// Build all examples.
    #[clap(long = "examples", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_examples: bool,

    /// Build only the specified example.
    #[clap(long = "example", value_name = "NAME")]
    pub example: Option<String>,

    /// Build all tests.
    #[clap(long = "tests", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_tests: bool,

    /// Build only the specified test target.
    #[clap(long = "test", value_name = "NAME")]
    pub test: Option<String>,

    /// Build all benches.
    #[clap(long = "benches", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_benches: bool,

    /// Build only the specified bench target.
    #[clap(long = "bench", value_name = "NAME")]
    pub bench: Option<String>,

    /// Build all benches.
    #[clap(long = "all-targets", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_all_targets: bool,
}

impl CargoTargetBuildArgs {
    pub(crate) fn args_builder(&self) -> ArgBuilder {
        ArgBuilder::new()
            .add_flag_if("--lib", self.is_lib)
            .add_flag_if("--bins", self.is_bins)
            .add_opt_value("--bin", &self.bin)
            .add_flag_if("--examples", self.is_examples)
            .add_opt_value("--example", &self.example)
            .add_flag_if("--tests", self.is_tests)
            .add_opt_value("--test", &self.test)
            .add_flag_if("--benches", self.is_benches)
            .add_opt_value("--bench", &self.bench)
            .add_flag_if("--all-targets", self.is_all_targets)
    }
}
