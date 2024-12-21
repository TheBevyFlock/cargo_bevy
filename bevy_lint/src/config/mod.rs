use std::{collections::BTreeMap, sync::RwLock};

use rustc_interface::Config;
use rustc_session::{config::Input, utils::was_invoked_from_cargo};
use rustc_span::Symbol;
use toml_edit::{DocumentMut, Table};

static LINT_CONFIG: RwLock<BTreeMap<Symbol, Table>> = RwLock::new(BTreeMap::new());

pub fn with_config<F, R>(name: Symbol, func: F) -> R
where
    F: FnOnce(&Table) -> R,
{
    let config_map = LINT_CONFIG.read().unwrap();

    match config_map.get(&name) {
        Some(config) => (func)(config),
        None => (func)(&Table::new())
    }
}

pub fn load_config(compiler_config: &Config) {
    let mut lint_config = LINT_CONFIG.write().unwrap();

    // Reset configuration. This prevents old values from leaking into new sessions in the case we
    // cannot load configuration.
    *lint_config = BTreeMap::new();

    if !was_invoked_from_cargo() {
        // TODO: Debug that we're skipping configuration steps;
        return;
    }

    let Input::File(ref input_path) = compiler_config.input else {
        // A string was passed directly to the compiler, not a file, so we cannot locate the
        // Cargo project.
        return;
    };

    let Ok(manifest_path) = crate::utils::cargo::locate_project(input_path, false) else {
        // TODO: Warn when failed to locate project.
        return;
    };

    let manifest = std::fs::read_to_string(manifest_path).unwrap();

    let toml = manifest.parse::<DocumentMut>().unwrap();

    todo!()
}
