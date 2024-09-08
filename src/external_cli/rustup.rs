//! Utilities for the `rustup` CLI tool.

use std::process::{exit, Command};

use dialoguer::Confirm;

const PROGRAM: &str = "rustup";

/// Create a new command to execute `rustup`.
fn command() -> Command {
    Command::new(PROGRAM)
}

/// Given a target triple, determine if it is already installed.
fn is_target_installed(target: &str) -> bool {
    let output = command().arg("target").arg("list").output();

    // Check if the target list has an entry like this:
    // <target_triple> (installed)
    if let Ok(output) = output {
        if let Ok(list) = String::from_utf8(output.stdout) {
            for line in list.lines() {
                if line.contains(target) && line.contains("(installed)") {
                    return true;
                }
            }
        }
    }

    false
}

/// Install a compilation target, if it is not already installed.
pub(crate) fn install_target_if_needed(
    target: &str,
    ask_user: bool,
    hidden: bool,
) -> anyhow::Result<()> {
    if is_target_installed(target) {
        return Ok(());
    }

    // Abort if the user doesn't want to install it
    if ask_user
        && !Confirm::new()
            .with_prompt(format!(
                "Compilation target `{target}` is missing, should I install it for you?",
            ))
            .interact()?
    {
        exit(1);
    }

    let mut cmd = command();
    cmd.arg("target").arg("add").arg(target);

    let status = if hidden {
        cmd.output()?.status
    } else {
        cmd.status()?
    };

    if !status.success() {
        Err(anyhow::anyhow!("Failed to install target `{}`.", target))
    } else {
        Ok(())
    }
}