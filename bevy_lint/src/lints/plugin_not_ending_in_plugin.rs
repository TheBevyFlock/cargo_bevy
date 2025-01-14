//! Checks for types who implement `Plugin` but whose names does not end in "Plugin".
//!
//! This does _not_ check function-style plugins (`fn plugin(app: &mut App)`), only structures with
//! `Plugin` explicitly implemented with `impl Plugin for T`.
//!
//! # Motivation
//!
//! Unlike traits like [`Clone`] or [`Debug`], the primary purpose of a type that implements
//! `Plugin` is to be a Bevy plugin. As such, it is common practice to suffix plugin names with
//! "Plugin" to signal how they should be used.
//!
//! # Example
//!
//! ```
//! # use bevy::prelude::*;
//! #
//! struct Physics;
//!
//! impl Plugin for Physics {
//!     fn build(&self, app: &mut App) {
//!         // ...
//!     }
//! }
//! ```
//!
//! Use instead:
//!
//! ```
//! # use bevy::prelude::*;
//! #
//! struct PhysicsPlugin;
//!
//! impl Plugin for PhysicsPlugin {
//!     fn build(&self, app: &mut App) {
//!         // ...
//!     }
//! }
//! ```

use crate::{declare_bevy_lint, declare_bevy_lint_pass};
use clippy_utils::{diagnostics::span_lint_hir_and_then, match_def_path, path_res};
use rustc_errors::Applicability;
use rustc_hir::{def::Res, HirId, Item, ItemKind, OwnerId};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::symbol::Ident;

declare_bevy_lint! {
    pub PLUGIN_NOT_ENDING_IN_PLUGIN,
    STYLE,
    "implemented `Plugin` for a structure whose name does not end in \"Plugin\"",
}

declare_bevy_lint_pass! {
    pub PluginNotEndingInPlugin => [PLUGIN_NOT_ENDING_IN_PLUGIN.lint],
}

impl<'tcx> LateLintPass<'tcx> for PluginNotEndingInPlugin {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &Item<'tcx>) {
        // Find `impl` items...
        if let ItemKind::Impl(impl_) = item.kind
            // ...that implement a trait...
            && let Some(of_trait) = impl_.of_trait
            // ...where the trait is a path to user code... (I don't believe this will ever be
            // false, since the alternatives are primitives, `Self`, and others that wouldn't make
            // since in this scenario.)
            && let Res::Def(_, trait_def_id) = of_trait.path.res
            // ...where the trait being implemented is Bevy's `Plugin`...
            && match_def_path(cx, trait_def_id, &crate::paths::PLUGIN)
        {
            // Try to resolve where this type was originally defined. This will result in a `DefId`
            // pointing to the original `struct Foo` definition, or `impl <T>` if it's a generic
            // parameter.
            let Some(struct_def_id) = path_res(cx, impl_.self_ty).opt_def_id() else {
                return;
            };

            // If this type is a generic parameter, exit. Their names, such as `T`, cannot be
            // referenced by others.
            if impl_
                .generics
                .params
                .iter()
                .any(|param| param.def_id.to_def_id() == struct_def_id)
            {
                return;
            }

            // Find the original name and span of the type. (We don't use the name from the path,
            // since that can be spoofed through `use Foo as FooPlugin`.)
            let Some(Ident {
                name: struct_name,
                span: struct_span,
            }) = cx.tcx.opt_item_ident(struct_def_id)
            else {
                return;
            };

            // If the type's name ends in "Plugin", exit.
            if struct_name.as_str().ends_with("Plugin") {
                return;
            }

            // Convert the `DefId` of the structure to a `LocalDefId`. If it cannot be converted
            // then the struct is from an external crate, in which case this lint should not be
            // emitted. (The user cannot easily rename that struct if they didn't define it.)
            let Some(struct_local_def_id) = struct_def_id.as_local() else {
                return;
            };

            // Convert struct `LocalDefId` to an `HirId` so that we can emit the lint for the
            // correct HIR node.
            let struct_hir_id: HirId = OwnerId {
                def_id: struct_local_def_id,
            }
            .into();

            span_lint_hir_and_then(
                cx,
                PLUGIN_NOT_ENDING_IN_PLUGIN.lint,
                struct_hir_id,
                struct_span,
                PLUGIN_NOT_ENDING_IN_PLUGIN.lint.desc,
                |diag| {
                    diag.span_suggestion(
                        struct_span,
                        "rename the plugin",
                        format!("{struct_name}Plugin"),
                        // There may be other references that also need to be renamed.
                        Applicability::MaybeIncorrect,
                    );

                    diag.span_note(item.span, "`Plugin` implemented here");
                },
            );
        }
    }
}
