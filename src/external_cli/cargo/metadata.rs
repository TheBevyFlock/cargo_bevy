#![expect(dead_code, reason = "Will be used for bevy bump and perhaps bevy run")]
use std::{ffi::OsStr, path::PathBuf, process::Command};

use semver::{Version, VersionReq};
use serde::Deserialize;

use super::program;

/// Create a command to run `cargo metadata`.
pub(crate) fn command() -> Command {
    let mut command = Command::new(program());
    // The format version needs to be fixed for compatibility and to avoid a warning log
    command.args(["metadata", "--format-version", "1"]);
    command
}

/// Try to obtain the Cargo metadata of this package.
pub(crate) fn metadata() -> anyhow::Result<Metadata> {
    metadata_with_args::<[&str; 0], &str>([])
}

/// Try to obtain the Cargo metadata of this package.
///
/// To see which additional args are available, [consult the `cargo metadata` documentation](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
/// or use `cargo metadata --help`.
pub(crate) fn metadata_with_args<I, S>(additional_args: I) -> anyhow::Result<Metadata>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = command().args(additional_args).output()?;
    let metadata = serde_json::from_slice(&output.stdout)?;
    Ok(metadata)
}

/// Metadata information about the current package.
///
/// See the [`cargo metadata` specification](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html#json-format).
#[derive(Debug, Deserialize)]
pub struct Metadata {
    /// List of all packages in the workspace.
    ///
    /// It also includes all feature-enabled dependencies unless `--no-deps` is used.
    pub packages: Vec<Package>,
    /// List of members of the workspace.
    ///
    /// Each entry is the Package ID for the package.
    pub workspace_members: Option<Vec<String>>,
    /// List of default members of the workspace.
    ///
    /// Each entry is the Package ID for the package.
    pub workspace_default_members: Option<Vec<String>>,
    /// The absolute path to the build directory where Cargo places its output.
    pub target_directory: PathBuf,
    /// The absolute path to the root of the workspace.
    pub workspace_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    /// The name of the package.
    pub name: String,
    /// The version of the package.
    pub version: Version,
    /// The Package ID for referring to the package within the document and as the `--package`
    /// argument to many commands.
    pub id: String,
    /// List of Cargo targets.
    pub targets: Vec<Target>,
    /// Absolute path to this package's manifest.
    pub manifest_path: PathBuf,
    /// Optional string that is the default binary picked by cargo run.
    pub default_run: Option<String>,
}

impl Package {
    /// Check if the package has an executable binary.
    pub fn has_bin(&self) -> bool {
        self.targets.iter().any(|target| {
            target
                .kind
                .iter()
                .any(|target_kind| *target_kind == TargetKind::Bin)
        })
    }

    /// An iterator over all binary targets contained in this package.
    pub fn bin_targets(&self) -> impl Iterator<Item = &Target> {
        self.targets
            .iter()
            .filter(|target| target.kind.iter().any(|kind| *kind == TargetKind::Bin))
    }

    /// An iterator over all example targets contained in this package.
    pub fn example_targets(&self) -> impl Iterator<Item = &Target> {
        self.targets
            .iter()
            .filter(|target| target.kind.iter().any(|kind| *kind == TargetKind::Example))
    }
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
    /// The name of the dependency.
    pub name: String,
    /// The version requirement for the dependency.
    ///
    /// Dependencies without a version requirement have a value of `*`.
    #[serde(default)]
    pub req: VersionReq,
    /// The dependency kind.
    ///
    /// `"dev"`, `"build"`, or `null` for a normal dependency.
    #[serde(default)]
    pub kind: DependencyKind,
    /// The file system path for a local path dependency.
    ///
    /// Not present if not a path dependency.
    pub path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    #[default]
    Normal,
    Dev,
    Build,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub kind: Vec<TargetKind>,
    /// The name of the target.
    ///
    /// For lib targets, dashes will be replaced with underscores.
    pub name: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    Lib,
    Rlib,
    Dylib,
    ProcMacro,
    Bin,
    Example,
    Test,
    Bench,
    CustomBuild,
    #[serde(untagged)]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_determine_metadata_of_this_package() {
        let metadata = metadata();
        assert!(metadata.is_ok());
        let metadata = metadata.unwrap();

        assert!(metadata
            .packages
            .iter()
            .any(|package| package.name == "bevy_cli"));
    }
}
