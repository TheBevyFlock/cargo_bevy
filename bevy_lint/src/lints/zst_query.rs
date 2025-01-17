//! Checks for queries that query for a zero-sized type.
//!
//! # Motivation
//!
//! Zero-sized types (ZSTs) are types that have no size as a result of containing no runtime data.
//! In Bevy, such types are often used as marker components and are best used as filters.
//!
//! # Example
//!
//! ```
//! # use bevy::prelude::*;
//!
//! #[derive(Component)]
//! struct Player;
//!
//! fn move_player(mut query: Query<(&mut Transform, &Player)>) {
//!     // ...
//! }
//! ```
//!
//! Use instead:
//!
//! ```
//! # use bevy::prelude::*;
//!
//! #[derive(Component)]
//! struct Player;
//!
//! fn move_player(query: Query<&mut Transform, With<Player>>) {
//!     // ...
//! }
//! ```

use crate::{
    declare_bevy_lint, declare_bevy_lint_pass,
    utils::hir_parse::{detuple, generic_type_at},
};
use clippy_utils::{
    diagnostics::span_lint_and_help,
    ty::{is_normalizable, match_type},
};
use rustc_abi::Size;
use rustc_hir_analysis::collect::ItemCtxt;
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{
    layout::{LayoutOf, TyAndLayout},
    Ty,
};

declare_bevy_lint! {
    pub ZST_QUERY,
    RESTRICTION,
    "query for a zero-sized type",
}

declare_bevy_lint_pass! {
    pub ZstQuery => [ZST_QUERY.lint],
}

impl<'tcx> LateLintPass<'tcx> for ZstQuery {
    fn check_ty(&mut self, cx: &LateContext<'tcx>, hir_ty: &'tcx rustc_hir::Ty<'tcx>) {
        let item_cx = ItemCtxt::new(cx.tcx, hir_ty.hir_id.owner.def_id);
        let ty = item_cx.lower_ty(hir_ty);

        let Some(query_kind) = QueryKind::try_from_ty(cx, ty) else {
            return;
        };

        let Some(query_data_ty) = generic_type_at(cx, hir_ty, 2) else {
            return;
        };

        for hir_ty in detuple(*query_data_ty) {
            let ty = item_cx.lower_ty(&hir_ty);

            // We want to make sure we're evaluating `Foo` and not `&Foo`/`&mut Foo`
            let peeled = ty.peel_refs();

            if !is_zero_sized(cx, peeled).unwrap_or_default() {
                continue;
            }

            // TODO: We can also special case `Option<&Foo>`/`Option<&mut Foo>` to
            //       instead suggest `Has<Foo>`
            span_lint_and_help(
                cx,
                ZST_QUERY.lint,
                hir_ty.span,
                ZST_QUERY.lint.desc,
                None,
                query_kind.help(peeled),
            );
        }
    }
}

enum QueryKind {
    Query,
}

impl QueryKind {
    fn try_from_ty<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> Option<Self> {
        if match_type(cx, ty, &crate::paths::QUERY) {
            Some(Self::Query)
        } else {
            None
        }
    }

    fn help(&self, ty: Ty<'_>) -> String {
        // It should be noted that `With<Foo>` is not always the best filter to suggest.
        // While it's most often going to be what users want, there's also `Added<Foo>`
        // and `Changed<Foo>` which might be more appropriate in some cases
        // (i.e. users are calling `foo.is_added()` or `foo.is_changed()` in the body of
        // the system).
        // In the future, we might want to span the usage site of `is_added()`/`is_changed()`
        // and suggest to use `Added<Foo>`/`Changed<Foo>` instead.
        match self {
            Self::Query => format!("consider using a filter instead: `With<{ty}>`"),
        }
    }
}

/// Checks if a type is zero-sized.
///
/// Returns:
/// - `Some(true)` if the type is most likely a ZST
/// - `Some(false)` if the type is most likely not a ZST
/// - `None` if we cannot determine the size (e.g., type is not normalizable)
fn is_zero_sized<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> Option<bool> {
    // `cx.layout_of()` panics if the type is not normalizable.
    if !is_normalizable(cx, cx.param_env, ty) {
        return None;
    }

    // Note: we don't use `approx_ty_size` from `clippy_utils` here
    // because it will return `0` as the default value if the type is not
    // normalizable, which will put us at risk of emitting more false positives.
    if let Ok(TyAndLayout { layout, .. }) = cx.layout_of(ty) {
        Some(layout.size() == Size::ZERO)
    } else {
        None
    }
}
