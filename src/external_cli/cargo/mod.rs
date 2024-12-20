use std::{env, ffi::OsString};

use clap::{ArgAction, Args};

use super::arg_builder::ArgBuilder;

pub(crate) mod build;
pub(crate) mod install;
pub(crate) mod metadata;
pub(crate) mod run;

fn program() -> OsString {
    env::var_os("BEVY_CLI_CARGO").unwrap_or("cargo".into())
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Feature Selection")]
pub struct CargoFeatureArgs {
    /// Space or comma separated list of features to activate
    #[clap(short = 'F', long = "features", value_name = "FEATURES")]
    pub features: Vec<String>,

    /// Activate all available features
    #[clap(long = "all-features", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_all_features: bool,

    /// Do not activate the `default` feature
    #[clap(long = "no-default-features", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_no_default_features: bool,
}

impl CargoFeatureArgs {
    pub(crate) fn args_builder(&self) -> ArgBuilder {
        ArgBuilder::new()
            .add_value_list("--features", self.features.clone())
            .add_flag_if("--all-features", self.is_all_features)
            .add_flag_if("--no-default-features", self.is_no_default_features)
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Compilation Options")]
pub struct CargoCompilationArgs {
    /// Build artifacts in release mode, with optimizations.
    #[clap(short = 'r', long = "release", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_release: bool,

    /// Build artifacts with the specified profile
    #[clap(long = "profile", value_name = "PROFILE-NAME")]
    pub profile: Option<String>,

    /// Number of parallel jobs, defaults to # of CPUs.
    #[clap(short = 'j', long = "jobs", value_name = "N")]
    pub jobs: Option<u32>,

    /// Do not abort the build as soon as there is an error
    #[clap(long = "keep-going", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_keep_going: bool,

    /// Build for the target triple.
    #[clap(long = "target", value_name = "TRIPLE")]
    pub target: Option<String>,

    /// Directory for all generated artifacts.
    #[clap(long = "target-dir", value_name = "DIRECTORY")]
    pub target_dir: Option<String>,
}

impl CargoCompilationArgs {
    /// The profile used to compile the app.
    ///
    /// This is determined by the `--release` and `--profile` arguments.
    pub(crate) fn profile(&self) -> &str {
        if self.is_release {
            "release"
        } else if let Some(profile) = &self.profile {
            profile
        } else {
            "debug"
        }
    }

    pub(crate) fn target(&self, is_web: bool) -> Option<String> {
        if is_web {
            Some("wasm32-unknown-unknown".to_string())
        } else {
            self.target.clone()
        }
    }

    pub(crate) fn args_builder(&self, is_web: bool) -> ArgBuilder {
        // web takes precedence over --target <TRIPLE>
        let target = if is_web {
            Some("wasm32-unknown-unknown".to_string())
        } else {
            self.target.clone()
        };

        ArgBuilder::new()
            .add_flag_if("--release", self.is_release)
            .add_opt_value("--profile", &self.profile)
            .add_opt_value("--jobs", &self.jobs.map(|jobs| jobs.to_string()))
            .add_flag_if("--keep-going", self.is_keep_going)
            .add_opt_value("--target", &target)
            .add_opt_value("--target-dir", &self.target_dir)
    }
}

#[derive(Debug, Args)]
#[command(next_help_heading = "Manifest Options")]
pub struct CargoManifestArgs {
    /// Path to Cargo.toml
    #[clap(long = "manifest-path", value_name = "PATH")]
    pub manifest_path: Option<String>,

    /// Ignore `rust-version` specification in packages
    #[clap(long = "ignore-rust-version", action = ArgAction::SetTrue, default_value_t = false)]
    pub ignore_rust_version: bool,

    /// Assert that `Cargo.lock` will remain unchanged
    #[clap(long = "locked", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_locked: bool,

    /// Run without accessing the network
    #[clap(long = "offline", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_offline: bool,

    /// Equivalent to specifying both --locked and --offline
    #[clap(long = "frozen", action = ArgAction::SetTrue, default_value_t = false)]
    pub is_frozen: bool,
}

impl CargoManifestArgs {
    pub(crate) fn args_builder(&self) -> ArgBuilder {
        ArgBuilder::new()
            .add_opt_value("--manifest-path", &self.manifest_path)
            .add_flag_if("--ignore-rust-version", self.ignore_rust_version)
            .add_flag_if("--locked", self.is_locked)
            .add_flag_if("--offline", self.is_offline)
            .add_flag_if("--frozen", self.is_frozen)
    }
}
