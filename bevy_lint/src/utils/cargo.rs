//! Utilities when working with [Cargo].
//!
//! [Cargo]: https://doc.rust-lang.org/cargo

use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str,
};

/// The name of the `cargo` executable.
const CARGO: &str = "cargo";

/// Locates the path to `Cargo.toml` from a path within a Cargo project.
///
/// This is a wrapper over [`cargo locate-project`]. By default it finds the `Cargo.toml` for the
/// current crate, but when `workspace` is true will find the `Cargo.toml` for the current
/// workspace.
///
/// [`cargo locate-project`]: https://doc.rust-lang.org/cargo/commands/cargo-locate-project.html
pub fn locate_project(relative_to: &Path, workspace: bool) -> io::Result<PathBuf> {
    let mut command = Command::new(CARGO);

    command
        .arg("locate-project")
        // Output the plain text path to `Cargo.toml`, not JSON.
        .arg("--message-format=plain")
        // If there is an error, display it directly to the user instead of capturing it.
        .stderr(Stdio::inherit());

    // If `relative_to` is a folder, set that as the working directory for the command. Else, if it
    // is a file, find the folder that it is contained in and use that instead.
    if relative_to.is_dir() {
        command.current_dir(relative_to);
    } else {
        command.current_dir(relative_to.parent().unwrap());
    }

    if workspace {
        command.arg("--workspace");
    }

    let output = command.output()?;

    // Convert the captured path to UTF-8, returning an error if it is not valid. We specifically
    // do not use `from_utf8_lossy()` here because replacing invalid UTF-8 with � would cause the
    // path to become incorrect. Better to emit an error here than a "file not found" later.
    let path = str::from_utf8(&output.stdout)
        .map_err(|utf_error| io::Error::new(io::ErrorKind::InvalidData, utf_error))?;

    Ok(PathBuf::from(path))
}
