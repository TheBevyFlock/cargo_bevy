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
    semver::{Version, VersionReq},
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

        // If `bevy` is listed as a direct dependency, use its version as the target version for all
        // other crates, and check for any dependents that use a different version.
        // If `bevy` is not listed as a direct dependency, check if multiple versions of `bevy` are
        // resolved. If so, report a single lint error.
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
    // Semver only supports checking if a given `VersionReq` matches a `Version` and not if two
    // `VersionReq` can successfully resolve to one `Version`. Therefore we try to parse the
    // `Version` from the `bevy` dependency in the `Cargo.toml` file. This only works if a
    // single version  of `bevy` is specified and not a range.
    let Ok(target_version) = get_version_from_toml(bevy_cargo.as_ref()) else {
        return;
    };

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
    // Examples of the underlying string representation of resolved crates
    // "id": "file:///path/to/my-package#0.1.0",
    // "id": "registry+https://github.com/rust-lang/crates.io-index#bevy@0.9.1",
    let mut resolved_bevy_versions: Vec<&str> = resolved
        .nodes
        .iter()
        .filter_map(|node| {
            // Extract version from local crates
            if node.id.repr.starts_with("file:///") {
                return node.id.repr.split('#').nth(1).map(|version| vec![version]);
            }
            // Extract versions from external crates
            if let Some((id, _)) = node.id.repr.split_once('@') {
                if bevy_dependents
                    .keys()
                    .any(|crate_name| id.ends_with(crate_name))
                {
                    return Some(
                        node.dependencies
                            .iter()
                            .filter_map(|dep| dep.repr.split_once('@'))
                            .filter_map(|(name, version)| {
                                (name.contains("bevy")).then_some(version)
                            })
                            .collect(),
                    );
                }
            }

            None
        })
        .flatten()
        .collect();

    resolved_bevy_versions.sort_unstable();
    resolved_bevy_versions.dedup();

    if resolved_bevy_versions.len() > 1 {
        span_lint(
            cx,
            DUPLICATE_BEVY_DEPENDENCIES.lint,
            rustc_span::DUMMY_SP,
            "found multiple versions of bevy",
        );
    }
}

/// Extracts the `version` field from a [`toml::Value`] and parses it into a [`Version`]
/// There are two possible formats:
/// 1. A toml-string `<crate> = <version>`
/// 2. A toml-table `<crate> = { version = <version> , ... }`
///
/// Cargo supports specifying version ranges,
/// but [`Version::from_str`] can only parse exact  versions and not ranges.
fn get_version_from_toml(table: &toml::Value) -> anyhow::Result<Version> {
    match table {
        toml::Value::String(version) => Version::from_str(version).map_err(anyhow::Error::from),
        toml::Value::Table(table) => table
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("The 'version' field is required."))
            .and_then(|version| Version::from_str(version).map_err(anyhow::Error::from)),
        _ => Err(anyhow::anyhow!(
            "Unexpected TOML format: expected a toml-string '<crate> = <version>' or a toml-table with '<crate> = {{ version = <version> }} '"
        )),
    }
}
