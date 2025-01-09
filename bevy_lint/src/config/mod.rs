use std::{collections::BTreeMap, sync::RwLock};

use rustc_interface::Config;
use rustc_lint::Level;
use rustc_session::{config::Input, utils::was_invoked_from_cargo};
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

use crate::utils;

/// The global lint configuration for the crate currently being compiled.
static LINT_CONFIG: RwLock<BTreeMap<String, InlineTable>> = RwLock::new(BTreeMap::new());

pub fn with_config<F, R>(name: &str, func: F) -> R
where
    F: FnOnce(&InlineTable) -> R,
{
    let config_map = LINT_CONFIG.read().unwrap();

    match config_map.get(name) {
        Some(config) => (func)(config),
        None => (func)(&InlineTable::new()),
    }
}

pub fn load_config(compiler_config: &mut Config) {
    // Lock the global linter configuration and get a mutable reference to it.
    let mut lint_config = LINT_CONFIG.write().unwrap();

    // Reset configuration. This prevents old values from leaking into new sessions in the case we
    // cannot load configuration.
    lint_config.clear();

    if !was_invoked_from_cargo() {
        // If we're not being called from Cargo, do not load any configuration.
        return;
    }

    let Some(manifest) = load_cargo_manifest(compiler_config) else {
        // If no manifest can be found, or it cannot be loaded, exit.
        return;
    };

    // Get all the data under `[package.metadata.bevy_lint]`, if any exists.
    let Some(linter_config) = manifest
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("bevy_lint"))
        .and_then(Item::as_table)
    else {
        // There is no configuration for `bevy_lint`, or it is not a table and should be skipped.
        return;
    };

    // Modify the compiler CLI arguments to include `--warn LINT`, `--allow LINT`, etc. for all
    // lint level configuration.
    append_lint_levels_to_options(compiler_config, linter_config);

    for (k, v) in linter_config {
        if let Item::Value(Value::InlineTable(inline_table)) = v {
            let mut extra_config = inline_table.clone();

            extra_config.remove("level");

            if !extra_config.is_empty() {
                lint_config.insert(k.into(), extra_config);
            }
        }
    }
}

/// Finds the `Cargo.toml` that `rustc` is most likely compiling for, and parses it into a
/// [`DocumentMut`].
fn load_cargo_manifest(compiler_config: &Config) -> Option<DocumentMut> {
    let Input::File(ref input_path) = compiler_config.input else {
        // A string was passed directly to the compiler, not a file, so we cannot locate the
        // Cargo project.
        return None;
    };

    let manifest_path = utils::cargo::locate_project(input_path, false).ok()?;

    let manifest = std::fs::read_to_string(manifest_path).ok()?;

    manifest.parse::<DocumentMut>().ok()
}

fn append_lint_levels_to_options(compiler_config: &mut Config, linter_config: &Table) {
    for (lint_name, lint_config) in linter_config {
        let lint_config = lint_config.as_value().unwrap();

        let level = match lint_config {
            // TODO: Emit an error for this
            Value::String(level) => Level::from_str(level.value()),
            Value::InlineTable(inline_table) => {
                inline_table
                    .get("level")
                    // TODO: Emit an error for this
                    .and_then(|value| value.as_str())
                    // TODO: Emit an error for this
                    .and_then(|level| Level::from_str(level))
            }
            // TODO: Emit an error for this
            _ => None,
        };

        if let Some(level) = level {
            compiler_config
                .opts
                .lint_opts
                .push((format!("bevy::{lint_name}"), level));
        }
    }
}
