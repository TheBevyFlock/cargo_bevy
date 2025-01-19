//! Checks for method or function calls inside of `commands.spawn` that return unit `()`
//!
//! # Motivation
//!
//! In Bevy, the `commands.spawn` method is used to create entities in the ECS with the given
//! `Bundle`. A `Bundle` can be a tuple of `Component` that should be added to this entity. If a
//! Value of type `()` is mistakenly passed, it results in an empty component being added.
//!
//! # Example
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use std::f32::consts::PI;
//! #
//! fn main() {
//!     App::new().add_systems(Startup, test);
//! }
//!
//! fn test(mut commands: Commands) {
//!     commands.spawn((
//!         Name::new("Decal"),
//!         Transform::from_translation(Vec3::new(0.75, 0.0, 0.0)).rotate_z(PI / 4.0),
//!     ));
//! }
//! ```
//!
//! ```text
//! warning: Expression returns `unit` and results in an empty component insertion
//!   --> src/main.rs:15:64
//!    |
//! 15 | ...Vec3::new(0.75, 0.0, 0.0)).rotate_z(PI / 4.0),
//!    |                               ^^^^^^^^
//!    |
//!    = note: `#[warn(bevy::unit_component_insertion)]` on by default
//! ```
use std::ops::ControlFlow;

use clippy_utils::{diagnostics::span_lint, sym, ty::match_type, visitors::for_each_expr};
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Symbol;

use crate::{declare_bevy_lint, declare_bevy_lint_pass};

declare_bevy_lint! {
    pub UNIT_COMPONENT_INSERTION,
    SUSPICIOUS,
    "method returns `unit` and will be inserted as a component",
}

declare_bevy_lint_pass! {
    pub UnitComponentInsertion => [UNIT_COMPONENT_INSERTION.lint],
    @default = {
        spawn: Symbol = sym!(spawn),
    },
}

impl<'tcx> LateLintPass<'tcx> for UnitComponentInsertion {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Find a method call.
        let ExprKind::MethodCall(path, src, args, _method_span) = expr.kind else {
            return;
        };

        let src_ty = cx.typeck_results().expr_ty(src).peel_refs();

        // If the method call was not to `commands.spawn` we skip it.
        if !(match_type(cx, src_ty, &crate::paths::COMMANDS) && path.ident.name == self.spawn) {
            return;
        }

        // iterate through all `Expr` inside the method `args` tuple, check if any return `()`
        for_each_expr(cx, args, |expr| {
            // get the expression definition of the Call
            let (def_id, span) = match expr.kind {
                ExprKind::Call(
                    &Expr {
                        kind: ExprKind::Path(ref path),
                        hir_id,
                        span,
                        ..
                    },
                    _,
                ) => {
                    let def_id = cx.qpath_res(path, hir_id).opt_def_id();
                    (def_id, span)
                }
                ExprKind::MethodCall(path, _, _, _) => (
                    cx.typeck_results().type_dependent_def_id(expr.hir_id),
                    path.ident.span,
                ),
                // If the expression was not of `kind` `Call` or `MethodCall`,
                // continue to the next Expression
                _ => return ControlFlow::<()>::Continue(()),
            };

            if let Some(def_id) = def_id {
                // Check if the return type of a function signature is of type `unit`
                if cx
                    .tcx
                    .fn_sig(def_id)
                    .skip_binder()
                    .output()
                    .skip_binder()
                    .is_unit()
                {
                    span_lint(
                        cx,
                        UNIT_COMPONENT_INSERTION.lint,
                        span,
                        "Expression returns `unit` and results in an empty component insertion",
                    );
                }
            }

            ControlFlow::<()>::Continue(())
        });
    }
}
