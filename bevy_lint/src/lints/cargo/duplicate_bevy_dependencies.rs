//! Checks for multiple versions of `bevy` in the dependencies.
//!
//! # Motivation
//!
//! When different third party crates use incompatible versions of Bevy, it can lead to confusing
//! errors and type incompatibilities.

use std::{collections::HashMap, path::Path, str::FromStr};

use crate::lints::cargo::{toml_span, CargoToml, DUPLICATE_BEVY_DEPENDENCIES};
use cargo_metadata::{
    semver::{Error, Version},
    Metadata,
};
use clippy_utils::{
    diagnostics::{span_lint, span_lint_and_help},
    find_crates,
};
use rustc_hir::def_id::LOCAL_CRATE;
use rustc_lint::LateContext;
use rustc_span::Symbol;

pub(super) fn check(cx: &LateContext<'_>, metadata: &Metadata, bevy_symbol: Symbol) {
    // no reason to continue the check if there is only one instance of `bevy` required
    if find_crates(cx.tcx, bevy_symbol).len() == 1 {
        return;
    }

    if let Ok(file) = cx.tcx.sess.source_map().load_file(Path::new("Cargo.toml"))
        && let Some(src) = file.src.as_deref()
        && let Ok(cargo_toml) = toml::from_str::<CargoToml>(src)
    {
        let local_name = cx.tcx.crate_name(LOCAL_CRATE);
        let target_version = get_version_from_toml(
            cargo_toml
                .dependencies
                .get("bevy")
                .unwrap()
                .as_ref()
                .clone(),
        )
        .unwrap();

        let mut incoherent_bevy_dependents = HashMap::new();

        for package in &metadata.packages {
            for dependency in &package.dependencies {
                if dependency.name.as_str() == "bevy"
                    && package.name.as_str() != local_name.as_str()
                    && !dependency.req.matches(&target_version)
                {
                    incoherent_bevy_dependents
                        .insert(package.name.as_str(), dependency.req.clone());
                }
            }
        }
        let bevy_toml_ref = cargo_toml
            .dependencies
            .get("bevy")
            .expect("bevy to be present");
        let bevy_ref_span = toml_span(bevy_toml_ref.span(), &file);
        for incoherent_version in &incoherent_bevy_dependents {
            // this can error if a dependency has a dependency that requires an incoherent version
            if let Some(cargo_toml_reference) = cargo_toml.dependencies.get(*incoherent_version.0) {
                span_lint_and_help(
                    cx,
                    DUPLICATE_BEVY_DEPENDENCIES.lint,
                    toml_span(cargo_toml_reference.span(), &file),
                    "Mismatching versions of `bevy` found".to_string(),
                    Some(bevy_ref_span),
                    format!("Help: Expected all crates to use `bevy` {target_version}"),
                );
            }
        }
    }
}

fn get_version_from_toml(table: toml::Value) -> Result<Version, Error> {
    // TODO: make this more robust, this fails if someone uses version ranges for bevy
    match table {
        toml::Value::String(version) => Version::from_str(version.as_str()),
        toml::Value::Table(map) => {
            let version = map.get("version").expect("version field is required");
            Version::from_str(
                version
                    .as_str()
                    .expect("version field is required to be a string"),
            )
        }
        _ => panic!("impossible to hit"),
    }
}
