//! Utilities for the `rustup` CLI tool.

use std::{env, ffi::OsString, process::Command};

use anyhow::Context;
use dialoguer::Confirm;

/// The rustup command can be customized via the `BEVY_CLI_RUSTUP` env
fn program() -> OsString {
    env::var_os("BEVY_CLI_RUSTUP").unwrap_or("rustup".into())
}

/// Given a target triple, determine if it is already installed.
fn is_target_installed(target: &str) -> bool {
    let output = Command::new(program()).arg("target").arg("list").output();

    // Check if the target list has an entry like this:
    // <target_triple> (installed)
    let Ok(output) = output else { return false };
    let list = String::from_utf8_lossy(&output.stdout);
    list.lines()
        .any(|line| line.contains(target) && line.contains("(installed)"))
}

/// Install a compilation target, if it is not already installed.
pub(crate) fn install_target_if_needed(target: &str, silent: bool) -> anyhow::Result<()> {
    if is_target_installed(target) {
        return Ok(());
    }

    if !silent {
        // Abort if the user doesn't want to install it
        if !Confirm::new()
            .with_prompt(format!(
                "Compilation target `{target}` is missing, should I install it for you?",
            ))
            .interact()
            .context(
                "failed to show interactive prompt, try using `--yes` to confirm automatically",
            )?
        {
            anyhow::bail!("User does not want to install target `{target}`.");
        }
    }

    println!("Installing missing target: `{target}`");

    let mut cmd = Command::new(program());
    cmd.arg("target").arg("add").arg(target);

    anyhow::ensure!(
        cmd.output()?.status.success(),
        "Failed to install target `{target}`."
    );
    Ok(())
}
