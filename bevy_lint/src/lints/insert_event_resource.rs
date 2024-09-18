//! Checks for the `Events<T>` resource being manually inserted through `App::init_resource()` or
//! `App::insert_resource()` instead of with `App::add_event()`.
//!
//! # Why is this bad?
//!
//! While `Events<T>` is technically a `Resource`, you cannot register an event by _just_ inserting
//! it into the `App`. There are other steps necessary as well, which are usually orchestrated by
//! `EventRegistry::register_event()`. Instead of inserting the `Events<T>` resource manually, call
//! `App::add_event()`, which will handle this all for you and result in desired behavior.
//!
//! # Example
//!
//! ```rust
//! # use bevy::prelude::*;
//! #
//! #[derive(Event)]
//! struct MyEvent;
//!
//! App::new().init_resource::<Events<MyEvent>>().run();
//! ```
//!
//! Use instead:
//!
//! ```rust
//! # use bevy::prelude::*;
//! #
//! #[derive(Event)]
//! struct MyEvent;
//!
//! App::new().add_event::<MyEvent>().run();
//! ```

use clippy_utils::{
    diagnostics::span_lint_and_sugg, source::snippet_with_applicability, sym, ty::match_type,
};
use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind, GenericArg, GenericArgs, Path, PathSegment, QPath};
use rustc_hir_analysis::lower_ty;
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{Ty, TyKind};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::Span;
use std::borrow::Cow;

declare_tool_lint! {
    pub bevy::INSERT_EVENT_RESOURCE,
    Deny,
    "called `App::insert_resource(Events<T>)` or `App::init_resource::<Events<T>>()` instead of `App::add_event::<T>()`"
}

declare_lint_pass! {
    InsertEventResource => [INSERT_EVENT_RESOURCE]
}

impl<'tcx> LateLintPass<'tcx> for InsertEventResource {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &Expr<'tcx>) {
        // Find a method call.
        if let ExprKind::MethodCall(path, src, args, method_span) = expr.kind {
            // Get the type for `src` in `src.method()`. We peel all references because the type
            // could either be `App` or `&mut App`.
            let src_ty = cx.typeck_results().expr_ty(src).peel_refs();

            // If `src` is not a Bevy `App`, exit.
            if !match_type(cx, src_ty, &crate::paths::APP) {
                return;
            }

            // If the method is `App::insert_resource()` or `App::init_resource()`, check it with
            // its corresponding function.
            match path.ident.name {
                symbol if symbol == sym!(insert_resource) => {
                    check_insert_resource(cx, args, method_span)
                }
                symbol if symbol == sym!(init_resource) => {
                    check_init_resource(cx, path, method_span)
                }
                _ => {}
            }
        }
    }
}

/// Checks if `App::insert_resource()` inserts an `Events<T>`, and emits a diagnostic if so.
fn check_insert_resource<'tcx>(cx: &LateContext<'tcx>, args: &[Expr], method_span: Span) {
    // Extract the argument if there is only 1 (which there should be!), else exit.
    let [arg] = args else {
        return;
    };

    // Find the type of `arg` in `App::insert_resource(arg)`.
    let ty = cx.typeck_results().expr_ty(arg);

    // If `arg` is `Events<T>`, emit the lint.
    if match_type(cx, ty, &crate::paths::EVENTS) {
        let mut applicability = Applicability::MachineApplicable;

        let event_ty_snippet = extract_ty_event_snippet(ty, &mut applicability);

        span_lint_and_sugg(
            cx,
            INSERT_EVENT_RESOURCE,
            method_span,
            "called `App::insert_resource(Events<T>)` instead of `App::add_event::<T>()`",
            "inserting an `Events` resource does not fully setup that event",
            format!("add_event::<{event_ty_snippet}>()"),
            applicability,
        );
    }
}

/// Creates a string representation of type `T` for [`Ty`] `Events<T>`.
///
/// This takes a mutable applicability reference, and will set it to
/// [`Applicability::HasPlaceholders`] if the type cannot be stringified.
fn extract_ty_event_snippet<'tcx>(
    events_ty: Ty<'tcx>,
    applicability: &mut Applicability,
) -> Cow<'tcx, str> {
    const DEFAULT: Cow<str> = Cow::Borrowed("T");

    let TyKind::Adt(_, events_arguments) = events_ty.kind() else {
        if let Applicability::MachineApplicable = applicability {
            *applicability = Applicability::HasPlaceholders;
        }

        return DEFAULT;
    };

    let Some(event_snippet) = events_arguments.iter().next() else {
        if let Applicability::MachineApplicable = applicability {
            *applicability = Applicability::HasPlaceholders;
        }

        return DEFAULT;
    };

    format!("{event_snippet:?}").into()
}

/// Checks if `App::init_resource()` inserts an `Events<T>`, and emits a diagnostic if so.
fn check_init_resource<'tcx>(cx: &LateContext<'tcx>, path: &PathSegment<'tcx>, method_span: Span) {
    if let Some(&GenericArgs {
        // `App::init_resource()` has one generic type argument: T.
        args: &[GenericArg::Type(resource_hir_ty)],
        ..
    }) = path.args
    {
        // Lower `rustc_hir::Ty` to `ty::Ty`, so we can inspect type information. For more
        // information, see <https://rustc-dev-guide.rust-lang.org/ty.html#rustc_hirty-vs-tyty>.
        // Note that `lower_ty()` is quasi-deprecated, and should be removed if a adequate
        // replacement is found.
        let resource_ty = lower_ty(cx.tcx, resource_hir_ty);

        // If the resource type is `Events<T>`, emit the lint.
        if match_type(cx, resource_ty, &crate::paths::EVENTS) {
            let mut applicability = Applicability::MachineApplicable;

            let event_ty_snippet =
                extract_hir_event_snippet(cx, resource_hir_ty, &mut applicability);

            span_lint_and_sugg(
                cx,
                INSERT_EVENT_RESOURCE,
                method_span,
                "called `App::init_resource::<Events<T>>()` instead of `App::add_event::<T>()`",
                "inserting an `Events` resource does not fully setup that event",
                format!("add_event::<{event_ty_snippet}>()"),
                applicability,
            );
        }
    }
}

/// Tries to extract the snippet `MyEvent` from the [`rustc_hir::Ty`] representing
/// `Events<MyEvent>`.
///
/// Note that this works on a best-effort basis, and will return `"T"` if the type cannot be
/// extracted. If so, it will mutate the passed applicability to [`Applicability::HasPlaceholders`],
/// similar to [`snippet_with_applicability()`].
fn extract_hir_event_snippet<'tcx>(
    cx: &LateContext<'tcx>,
    events_hir_ty: &rustc_hir::Ty<'tcx>,
    applicability: &mut Applicability,
) -> Cow<'static, str> {
    const DEFAULT: Cow<str> = Cow::Borrowed("T");

    // This is some crazy pattern matching. Let me walk you through it:
    let event_span = match events_hir_ty.kind {
        // There are multiple kinds of HIR types, but we're looking for a path to a type
        // definition. This path is likely `Events`, and contains the generic argument that we're
        // searching for.
        rustc_hir::TyKind::Path(QPath::Resolved(
            _,
            &Path {
                // There can be multiple segments in a path, such as if it were
                // `bevy::prelude::Events`, but in this case we just care about the last: `Events`.
                segments:
                    &[.., PathSegment {
                        // Find the arguments to `Events<T>`, extracting `T`.
                        args:
                            Some(&GenericArgs {
                                args: &[GenericArg::Type(ty)],
                                ..
                            }),
                        ..
                    }],
                ..
            },
        )) => {
            // We now have the HIR type `T` for `Events<T>`, let's return its span.
            ty.span
        }
        // Something in the above pattern matching went wrong, likely due to an edge case. For
        // this, we set the applicability to `HasPlaceholders` and return the default snippet.
        _ => {
            if let Applicability::MachineApplicable = applicability {
                *applicability = Applicability::HasPlaceholders;
            }

            return DEFAULT;
        }
    };

    // We now have the span to the event type, so let's try to extract it into a string.
    snippet_with_applicability(cx, event_span, &DEFAULT, applicability)
}
