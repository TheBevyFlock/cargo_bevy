use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use anyhow::Context;
use serial_test::serial;

pub trait CommandHelpers {
    fn ensure_status(&mut self) -> anyhow::Result<ExitStatus>;
}

impl CommandHelpers for Command {
    /// Ensure that the command exits with a successful status code.
    fn ensure_status(&mut self) -> anyhow::Result<ExitStatus> {
        let status = self.status()?;
        anyhow::ensure!(
            status.success(),
            "Command {} exited with status code {}",
            self.get_program().to_string_lossy(),
            status
        );
        Ok(status)
    }
}

/// The path to the test repository.
fn test_path() -> PathBuf {
    Path::new("bevy_cli_test").to_owned()
}

/// The path to the target directory of the test repository.
fn target_path() -> PathBuf {
    test_path().join("target")
}

/// The name to the executable on Windows, ending in ".exe".
#[cfg(target_os = "windows")]
fn executable(binary: &str) -> String {
    format!("{binary}.exe")
}

/// The name of the executable on Linux and MacOS, without a file extension.
#[cfg(not(target_os = "windows"))]
fn executable(binary: &str) -> String {
    binary.to_string()
}

/// General setup to use throughout the tests.
fn setup() -> anyhow::Result<()> {
    install_bevy_cli()?;
    Ok(())
}

/// Install the Bevy CLI to use it in the tests.
fn install_bevy_cli() -> anyhow::Result<ExitStatus> {
    Command::new("cargo")
        .args(["install", "--path=./", "--color=always"])
        .ensure_status()
        .context("failed installing Bevy CLI")
}

/// Delete the binaries in the given path inside the target directory.
///
/// Still retains some cached compilations to speed up compile times,
/// but ensures that binaries are created freshly.
fn clean_target_binaries<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let path_to_clean = target_path().join(path);
    if fs::exists(&path_to_clean)? {
        for entry in (fs::read_dir(path_to_clean)?).flatten() {
            if entry.file_type()?.is_file() {
                fs::remove_file(entry.path())?;
            }
        }
    }
    Ok(())
}

fn ensure_target_exists<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let artifact_path = target_path().join(path);
    anyhow::ensure!(
        fs::exists(&artifact_path)?,
        "artifact {} does not exist",
        artifact_path.display()
    );
    Ok(())
}

/// The bevy CLI command, executed in the test repository.
fn bevy() -> Command {
    let mut cmd = Command::new("bevy");
    cmd.current_dir(test_path());
    cmd
}

#[test]
#[serial]
fn should_build_native_dev() -> anyhow::Result<()> {
    setup()?;
    let target_path = Path::new("debug");
    clean_target_binaries(target_path)?;
    bevy()
        .args(["build", "-p=bevy_default", "--yes"])
        .ensure_status()?;

    ensure_target_exists(target_path.join(executable("bevy_default")))
        .context("binary does not exist")
}

#[test]
#[serial]
fn should_build_native_release() -> anyhow::Result<()> {
    setup()?;
    let target_path = Path::new("release");
    clean_target_binaries(target_path)?;
    bevy()
        .args(["build", "-p=bevy_default", "--yes", "--release"])
        .ensure_status()?;

    ensure_target_exists(target_path.join(executable("bevy_default")))
        .context("binary does not exist")
}

#[test]
#[serial]
fn should_build_web_dev() -> anyhow::Result<()> {
    setup()?;
    let target_path = Path::new("wasm32-unknown-unknown").join("web");
    clean_target_binaries(&target_path)?;
    bevy()
        .args(["build", "-p=bevy_default", "--yes", "web"])
        .ensure_status()?;

    ensure_target_exists(target_path.join("bevy_default.wasm"))
        .context("Wasm executable does not exist")?;
    ensure_target_exists(target_path.join("bevy_default_bg.wasm"))
        .context("Wasm bindings do not exist")?;
    ensure_target_exists(target_path.join("bevy_default.js")).context("JS bindings do not exist")
}

#[test]
#[serial]
fn should_build_web_release() -> anyhow::Result<()> {
    setup()?;
    let target_path = Path::new("wasm32-unknown-unknown").join("web-release");
    clean_target_binaries(&target_path)?;
    bevy()
        .args(["build", "-p=bevy_default", "--release", "--yes", "web"])
        .ensure_status()?;

    ensure_target_exists(target_path.join("bevy_default.wasm"))
        .context("Wasm executable does not exist")?;
    ensure_target_exists(target_path.join("bevy_default_bg.wasm"))
        .context("Wasm bindings do not exist")?;
    ensure_target_exists(target_path.join("bevy_default.js")).context("JS bindings do not exist")
}
