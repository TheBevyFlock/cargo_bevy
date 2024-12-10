//! Checks for multiple versions of `bevy` in the dependencies.
//!
//! # Motivation
//!
//! When different third party crates use incompatible versions of Bevy, it can lead to confusing
//! errors and type incompatibilities.

use std::{collections::HashMap, path::Path, str::FromStr, sync::Arc};

use crate::lints::cargo::{toml_span, CargoToml, DUPLICATE_BEVY_DEPENDENCIES};
use cargo_metadata::{
    semver::{Error, Version, VersionReq},
    Metadata,
};
use clippy_utils::{diagnostics::span_lint_and_then, find_crates};
use rustc_hir::def_id::LOCAL_CRATE;
use rustc_lint::LateContext;
use rustc_span::{SourceFile, Symbol};
use toml::Spanned;

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

        let mut bevy_dependents = HashMap::new();
        for package in &metadata.packages {
            for dependency in &package.dependencies {
                if dependency.name.as_str() == "bevy"
                    && package.name.as_str() != local_name.as_str()
                {
                    bevy_dependents.insert(package.name.as_str(), dependency.req.clone());
                }
            }
        }

        match cargo_toml.dependencies.get("bevy") {
            Some(bevy_cargo) => {
                lint_with_target_version(cx, &cargo_toml, &file, bevy_cargo, &bevy_dependents);
            }
            None => minimal_lint(cx, &bevy_dependents),
        };
    }
}

fn lint_with_target_version(
    cx: &LateContext<'_>,
    cargo_toml: &CargoToml,
    file: &Arc<SourceFile>,
    bevy_cargo: &Spanned<toml::Value>,
    bevy_dependents: &HashMap<&str, VersionReq>,
) {
    let target_version = get_version_from_toml(bevy_cargo.as_ref()).unwrap();
    let bevy_cargo_toml_span = toml_span(bevy_cargo.span(), file);

    let mismatching_dependencies = bevy_dependents
        .iter()
        .filter(|dependency| !dependency.1.matches(&target_version));

    for mismatching_dependency in mismatching_dependencies {
        if let Some(cargo_toml_reference) = cargo_toml.dependencies.get(*mismatching_dependency.0) {
            span_lint_and_then(
                cx,
                DUPLICATE_BEVY_DEPENDENCIES.lint,
                toml_span(cargo_toml_reference.span(), file),
                "Mismatching versions of `bevy` found".to_string(),
                |diag| {
                    diag.span_help(
                        bevy_cargo_toml_span,
                        format!("Expected all crates to use `bevy` {target_version}"),
                    );
                },
            );
        }
    }
}

fn get_version_from_toml(table: &toml::Value) -> Result<Version, Error> {
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

fn minimal_lint(cx: &LateContext<'_>, bevy_dependents: &HashMap<&str, VersionReq>) {
    todo!()
}
