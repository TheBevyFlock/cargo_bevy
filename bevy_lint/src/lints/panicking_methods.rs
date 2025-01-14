//! Checks for use of panicking methods of `Query`, `QueryState`, or `World` when a non-panicking
//! alternative exists.
//!
//! For instance, this will lint against `Query::single()`, recommending that `Query::get_single()`
//! should be used instead.
//!
//! This lint is actually two: [`PANICKING_QUERY_METHODS`] and [`PANICKING_WORLD_METHODS`]. Each
//! can be toggled separately. The query variant lints for `Query` and `QueryState`, while the
//! world variant lints for `World`.
//!
//! # Motivation
//!
//! Panicking is the nuclear option of error handling in Rust: it is meant for cases where recovery
//! is near-impossible. As such, panicking is usually undesirable in long-running applications
//! and games like what Bevy is used for. This lint aims to prevent unwanted crashes in these
//! applications by forcing developers to handle the `Option` or `Result` in their code.
//!
//! # Example
//!
//! ```
//! # use bevy::prelude::*;
//! #
//! #[derive(Component)]
//! struct MyComponent;
//!
//! #[derive(Resource)]
//! struct MyResource;
//!
//! fn panicking_query(query: Query<&MyComponent>) {
//!     let component = query.single();
//!     // ...
//! }
//!
//! fn panicking_world(world: &mut World) {
//!     let resource = world.resource::<MyResource>();
//!     // ...
//! }
//! #
//! # bevy::ecs::system::assert_is_system(panicking_query);
//! # bevy::ecs::system::assert_is_system(panicking_world);
//! ```
//!
//! Use instead:
//!
//! ```
//! # use bevy::prelude::*;
//! #
//! #[derive(Component)]
//! struct MyComponent;
//!
//! #[derive(Resource)]
//! struct MyResource;
//!
//! fn graceful_query(query: Query<&MyComponent>) {
//!     match query.get_single() {
//!         Ok(component) => {
//!             // ...
//!         }
//!         Err(error) => {
//!             error!("Invariant not upheld: {:?}", error);
//!             return;
//!         }
//!     }
//! }
//!
//! fn graceful_world(world: &mut World) {
//!     let Some(resource) = world.get_resource::<MyResource>() else {
//!         // Resource may not exist.
//!         return;
//!     };
//!
//!     // ...
//! }
//! #
//! # bevy::ecs::system::assert_is_system(graceful_query);
//! # bevy::ecs::system::assert_is_system(graceful_world);
//! ```

use crate::{declare_bevy_lint, declare_bevy_lint_pass};
use clippy_utils::{
    diagnostics::span_lint_and_help,
    source::{snippet, snippet_opt},
    ty::match_type,
};
use rustc_hir::{Expr, ExprKind, GenericArgs};
use rustc_lint::{LateContext, LateLintPass, Lint};
use rustc_middle::ty::Ty;
use rustc_span::{Span, Symbol};

declare_bevy_lint! {
    pub PANICKING_QUERY_METHODS,
    RESTRICTION,
    "called a `Query` or `QueryState` method that can panic when a non-panicking alternative exists",
}

declare_bevy_lint! {
    pub PANICKING_WORLD_METHODS,
    RESTRICTION,
    "called a `World` method that can panic when a non-panicking alternative exists",
}

declare_bevy_lint_pass! {
    pub PanickingMethods => [PANICKING_QUERY_METHODS.lint, PANICKING_WORLD_METHODS.lint],
}

impl<'tcx> LateLintPass<'tcx> for PanickingMethods {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &Expr<'tcx>) {
        // Find a method call.
        if let ExprKind::MethodCall(path, src, args, method_span) = expr.kind {
            // Get the type of `src` for `src.method()`. We peel all references to that `Foo`,
            // `&Foo`, `&&Foo`, etc. all look identical, since method calls automatically
            // dereference the source.
            let src_ty = cx.typeck_results().expr_ty(src).peel_refs();

            // Check if `src` is a type that has panicking methods (e.g. `Query`), else exit.
            let Some(panicking_type) = PanickingType::try_from_ty(cx, src_ty) else {
                return;
            };

            // Get a list of methods that panic and their alternatives for the specific query
            // variant.
            let panicking_alternatives = panicking_type.alternatives();

            // Here we check if the method name matches one of methods in `panicking_alternatives`.
            // If it does match, we store the recommended alternative for reference in diagnostics
            // later. If nothing matches, we exit the entire function.
            let alternative = 'block: {
                for (panicking_method, alternative_method) in panicking_alternatives {
                    // If performance is an issue in the future, this could be cached.
                    let key = Symbol::intern(panicking_method);

                    if path.ident.name == key {
                        // It is one of the panicking methods. Write down the alternative and stop
                        // searching.
                        break 'block *alternative_method;
                    }
                }

                // If we reach this point, the method is not one we're searching for. In this case,
                // we exit.
                return;
            };

            // By this point, we've verified that `src` is a panicking type and the method is one
            // that panics with a viable alternative. Let's emit the lint.

            // Try to find the string representation of `src`. This usually returns `my_query`
            // without the trailing `.`, so we manually append it. When the snippet cannot be
            // found, we default to the qualified `Type::` form.
            let src_snippet = snippet_opt(cx, src.span).map_or_else(
                || format!("{}::", panicking_type.name()),
                |mut s| {
                    s.push('.');
                    s
                },
            );

            // Try to find the generic arguments of the method, if any exist. This can either
            // evaluate to `""` or `"::<A, B, C>"`.
            let generics_snippet = path
                .args // Find the generic arguments of this path.
                .and_then(GenericArgs::span_ext) // Find the span of the generics.
                .and_then(|span| snippet_opt(cx, span)) // Extract the string, which may look like `<A, B>`.
                .map(|snippet| format!("::{snippet}")) // Insert `::` before the string.
                .unwrap_or_default(); // If any of the previous failed, return an empty string.

            // Try to find the string representation of the arguments to our panicking method. See
            // `span_args()` for more details on how this is done.
            let args_snippet = snippet(cx, span_args(args), "");

            span_lint_and_help(
                cx,
                panicking_type.lint(),
                method_span,
                format!(
                    "called a `{}` method that can panic when a non-panicking alternative exists",
                    panicking_type.name()
                ),
                None,
                // This usually ends up looking like: `query.get_many([e1, e2])`.
                format!("use `{src_snippet}{alternative}{generics_snippet}({args_snippet})` and handle the `Option` or `Result`"),
            );
        }
    }
}

enum PanickingType {
    Query,
    QueryState,
    World,
}

impl PanickingType {
    /// Returns the corresponding variant for the given [`Ty`], if it is supported by this lint.
    fn try_from_ty<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> Option<Self> {
        if match_type(cx, ty, &crate::paths::QUERY) {
            Some(Self::Query)
        } else if match_type(cx, ty, &crate::paths::QUERY_STATE) {
            Some(Self::QueryState)
        } else if match_type(cx, ty, &crate::paths::WORLD) {
            Some(Self::World)
        } else {
            None
        }
    }

    /// Returns a list of panicking methods for each of the supported types.
    ///
    /// Each item in the returned [`slice`] is of the format
    /// `(panicking_method, alternative_method)`.
    fn alternatives(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Query => &[
                ("single", "get_single"),
                ("single_mut", "get_single_mut"),
                ("many", "get_many"),
                ("many_mut", "get_many_mut"),
            ],
            Self::QueryState => &[
                ("single", "get_single"),
                ("single_mut", "get_single_mut"),
                // `QueryState` does not currently have `many()` or `many_mut()`.
            ],
            Self::World => &[
                ("entity", "get_entity"),
                ("entity_mut", "get_entity_mut"),
                ("many_entities", "get_many_entities"),
                ("many_entities_mut", "get_many_entities_mut"),
                ("resource", "get_resource"),
                ("resource_mut", "get_resource_mut"),
                ("resource_ref", "get_resource_ref"),
                ("non_send_resource", "get_non_send_resource"),
                ("non_send_resource_mut", "get_non_send_resource_mut"),
                ("run_schedule", "try_run_schedule"),
                ("schedule_scope", "try_schedule_scope"),
            ],
        }
    }

    /// Returns the name of the type this variant represents.
    fn name(&self) -> &'static str {
        match self {
            Self::Query => "Query",
            Self::QueryState => "QueryState",
            Self::World => "World",
        }
    }

    /// Returns the [`Lint`] associated with this panicking type.
    ///
    /// This can either return [`PANICKING_QUERY_METHODS`] or [`PANICKING_WORLD_METHODS`].
    fn lint(&self) -> &'static Lint {
        match self {
            Self::Query | Self::QueryState => PANICKING_QUERY_METHODS.lint,
            Self::World => PANICKING_WORLD_METHODS.lint,
        }
    }
}

/// Returns the [`Span`] of an array of method arguments.
///
/// [`ExprKind::MethodCall`] does not provide a good method for extracting the [`Span`] of _just_
/// the method's arguments. Instead, it contains a [`slice`] of [`Expr`]. This function tries it's
/// best to find a span that contains all arguments from the passed [`slice`].
///
/// This function assumes that `args` is sorted by order of appearance. An [`Expr`] that appears
/// earlier in the source code should appear earlier in the [`slice`].
///
/// If there are no [`Expr`]s in the [`slice`], this will return [`Span::default()`].
fn span_args(args: &[Expr]) -> Span {
    // Start with an empty span. If `args` is empty, this will be returned. This may look like
    // `0..0`.
    let mut span = Span::default();

    // If at least 1 item exists in `args`, get the first expression and overwrite `span` with it's
    // value. `span` may look like `7..12` now, with a bit of extra metadata.
    if let Some(first_arg) = args.first() {
        span = first_arg.span;
    }

    // Get the last `Expr`, if it exists, and overwrite our span's highest index with the last
    // expression's highest index. If there is only one item in `args`, this will appear to do
    // nothing. `span` may now look like `7..20`.
    if let Some(last_arg) = args.last() {
        span = span.with_hi(last_arg.span.hi());
    }

    span
}
