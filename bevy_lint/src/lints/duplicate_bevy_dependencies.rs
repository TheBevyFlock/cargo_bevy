//! Checks for multiple versions of `bevy` in the dependencies.
//!
//! # Motivation
//!
//! When different third party crates use incompatible versions of Bevy, it can lead to confusing
//! errors and type incompatibilities.

use crate::declare_bevy_lint;
use clippy_utils::{diagnostics::span_lint, find_crates, sym};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::impl_lint_pass;
use rustc_span::Symbol;

declare_bevy_lint! {
    pub DUPLICATE_BEVY_DEPENDENCIES,
    CORRECTNESS,
    "duplicate bevy dependencies",
}

pub(crate) struct DuplicateBevyDependencies {
    bevy_symbol: Symbol,
}

impl Default for DuplicateBevyDependencies {
    fn default() -> Self {
        Self {
            bevy_symbol: sym!(bevy),
        }
    }
}

impl_lint_pass! {
     DuplicateBevyDependencies => [DUPLICATE_BEVY_DEPENDENCIES.lint]
}

impl<'tcx> LateLintPass<'tcx> for DuplicateBevyDependencies {
    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        let bevy_crates = find_crates(cx.tcx, self.bevy_symbol);

        if bevy_crates.len() > 1 {
            let span = cx.tcx.def_span(bevy_crates[1].def_id());
            span_lint(
                cx,
                DUPLICATE_BEVY_DEPENDENCIES.lint,
                span,
                "Multiple versions of `bevy` found",
            );
        }
    }
}
