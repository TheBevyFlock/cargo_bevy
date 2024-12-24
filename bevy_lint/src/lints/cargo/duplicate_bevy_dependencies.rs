//! Checks for multiple versions of `bevy` in the dependencies.
//!
//! # Motivation
//!
//! When different third party crates use incompatible versions of Bevy, it can lead to confusing
//! errors and type incompatibilities.
//!
//! # Example
//!
//! ```toml
//! [package]
//! name = "multiple-bevy-versions"
//! version = "0.1.0"
//! publish = false
//! edition = "2021"
//!
//! [workspace]
//!
//! [dependencies]
//! bevy = { version = "0.14.2" }
//! leafwing-input-manager = "0.13"
//! ```
//!
//! Lint output:
//! error: Mismatching versions of `bevy` found
//!   --> Cargo.toml:11:26
//!    |
//! 11 | leafwing-input-manager = "0.13"
//!    |                          ^^^^^^
//!    |
//! help: Expected all crates to use `bevy` 0.14.2
//!   --> Cargo.toml:10:8
//!    |
//! 10 | bevy = { version = "0.14.2" }
//!    |        ^^^^^^^^^^^^^^^^^^^^^^
//!    = note: `#[deny(bevy::duplicate_bevy_dependencies)]` on by default
//!
//! error: could not compile `multiple-bevy-versions` (bin "multiple-bevy-versions") due to 1
//! previous error Check failed: exit status: 101.

use std::{collections::HashMap, path::Path, str::FromStr, sync::Arc};

use crate::lints::cargo::{toml_span, CargoToml, DUPLICATE_BEVY_DEPENDENCIES};
use cargo_metadata::{
    semver::{Error, Version, VersionReq},
    Metadata, Resolve,
};
use clippy_utils::{
    diagnostics::{span_lint, span_lint_and_then},
    find_crates,
};
use rustc_hir::def_id::LOCAL_CRATE;
use rustc_lint::LateContext;
use rustc_span::{SourceFile, Symbol};
use toml::Spanned;

pub(super) fn check(cx: &LateContext<'_>, metadata: &Metadata, bevy_symbol: Symbol) {
    // no reason to continue the check if there is only one instance of `bevy` required
    if find_crates(cx.tcx, bevy_symbol).len() == 1 {
        return;
    }

    // Load the `Cargo.toml` into the `SourceMap` this is  necessary to get the `Span` of the
    // `Cargo.toml` file.
    if let Ok(file) = cx.tcx.sess.source_map().load_file(Path::new("Cargo.toml"))
        && let Some(src) = file.src.as_deref()
        // Parse the `Cargo.toml` file into a `CargoToml` struct, this helps getting the correct span and not just
        // the root span of the `Cargo.toml` file.
        && let Ok(cargo_toml) = toml::from_str::<CargoToml>(src)
    {
        let local_name = cx.tcx.crate_name(LOCAL_CRATE);

        // get the package name and the corresponding version of `bevy` that they depend on
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

        // If there is a direct dependency on `bevy` use this as the target version for all other
        // crates and lint all the dependents that require a different version.
        // otherwise check if the resolved dependencies have more than one version of `bevy` and
        // lint a single error if that is the case.
        match cargo_toml.dependencies.get("bevy") {
            Some(bevy_cargo) => {
                lint_with_target_version(cx, &cargo_toml, &file, bevy_cargo, &bevy_dependents);
            }

            None => {
                if let Some(resolve) = &metadata.resolve {
                    minimal_lint(cx, &bevy_dependents, resolve);
                };
            }
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

fn minimal_lint(
    cx: &LateContext<'_>,
    bevy_dependents: &HashMap<&str, VersionReq>,
    resolved: &Resolve,
) {
    let mut dependencies = resolved
        .nodes
        .iter()
        .filter_map(|node| {
            // "id": "file:///path/to/my-package#0.1.0",
            // "id": "registry+https://github.com/rust-lang/crates.io-index#bevy@0.9.1",
            // "id": "registry+https://github.com/rust-lang/crates.io-index#regex@1.11.1",
            if node.id.repr.starts_with("file:///") {
                todo!()
            }
            if let Some((id, _version)) = node.id.repr.split_once('@') {
                if bevy_dependents
                    .keys()
                    .any(|crate_name| id.ends_with(crate_name))
                {
                    return Some(
                        node.dependencies
                            .iter()
                            .filter_map(|dep| {
                                if dep.repr.contains("bevy@") {
                                    return Some(
                                        dep.repr
                                            .split('@')
                                            .nth(1)
                                            .expect("resolved crate to contain <url>@<version>"),
                                    );
                                }
                                None
                            })
                            .collect::<Vec<&str>>(),
                    );
                }
            }
            None
        })
        .flatten()
        .collect::<Vec<&str>>();

    dependencies.sort_unstable();
    dependencies.dedup();

    if dependencies.len() > 1 {
        span_lint(
            cx,
            DUPLICATE_BEVY_DEPENDENCIES.lint,
            rustc_span::DUMMY_SP,
            "found multiple versions of bevy",
        );
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
