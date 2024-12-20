//! Checks for function parameters that take a mutable reference to a
//! re-borrowable type.
//!
//! # Motivation
//!
//! Several Bevy types look like they are owned, when in reality they contain an `&mut` reference
//! to the data owned by the ECS. `Commands` and `Query` are examples of such types that _pretend_
//! to own data for better user ergonomics.
//!
//! This can be an issue when a user writes a function that takes a mutable reference to one of
//! these types, not realizing that it itself is _already_ a reference. These mutable references
//! can almost always be readily converted back to an owned instance of the type, which is a cheap
//! operation that avoids nested references.
//!
//! The only time a re-borrowable type cannot be re-borrowed is when the function returns
//! referenced data that is bound to the mutable reference of the re-borrowable type.
//!
//! # Known Issues
//!
//! This lint does not currently support the [`Fn`] traits or function pointers. This means the
//! following types will not be caught by the lint:
//!
//! - `impl FnOnce(&mut Commands)`
//! - `Box<dyn FnMut(&mut Commands)>`
//! - `fn(&mut Commands)`
//!
//! # Example
//!
//! ```
//! # use bevy::prelude::*;
//! #
//! fn system(mut commands: Commands) {
//!     helper_function(&mut commands);
//! }
//!
//! // This takes `&mut Commands`, but it doesn't need to!
//! fn helper_function(commands: &mut Commands) {
//!     // ...
//! }
//! #
//! # bevy::ecs::system::assert_is_system(system);
//! ```
//!
//! Use instead:
//!
//! ```
//! # use bevy::prelude::*;
//! #
//! fn system(mut commands: Commands) {
//!     // Convert `&mut Commands` to `Commands`.
//!     helper_function(commands.reborrow());
//! }
//!
//! fn helper_function(mut commands: Commands) {
//!     // ...
//! }
//! #
//! # bevy::ecs::system::assert_is_system(system);
//! ```
//!
//! The following is an example where a type cannot be re-borrowed, for which this lint will not
//! emit any warning:
//!
//! ```
//! # use bevy::{prelude::*, ecs::system::EntityCommands};
//! #
//! fn system(mut commands: Commands) {
//!     let entity_commands = helper_function(&mut commands);
//! }
//!
//! // Note how this function returns a reference with the same lifetime as `Commands`.
//! fn helper_function<'a>(commands: &'a mut Commands) -> EntityCommands<'a> {
//!     commands.spawn_empty()
//! }
//! #
//! # bevy::ecs::system::assert_is_system(system);
//! ```

use std::ops::ControlFlow;

use crate::declare_bevy_lint;
use clippy_utils::{diagnostics::span_lint_and_sugg, ty::match_type};
use rustc_errors::Applicability;
use rustc_hir::{intravisit::FnKind, Body, FnDecl, Mutability};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{Interner, Ty, TyKind, TypeVisitable, TypeVisitor};
use rustc_session::declare_lint_pass;
use rustc_span::{
    def_id::LocalDefId,
    symbol::{kw, Ident},
    Span,
};

declare_bevy_lint! {
    pub BORROWED_REBORROWABLE,
    PEDANTIC,
    "parameter takes a mutable reference to a re-borrowable type",
}

declare_lint_pass! {
    BorrowedReborrowable => [BORROWED_REBORROWABLE.lint]
}

impl<'tcx> LateLintPass<'tcx> for BorrowedReborrowable {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: FnKind<'tcx>,
        decl: &'tcx FnDecl<'tcx>,
        _: &'tcx Body<'tcx>,
        _: Span,
        def_id: LocalDefId,
    ) {
        let fn_sig = match kind {
            FnKind::Closure => cx.tcx.closure_user_provided_sig(def_id).value,
            // We use `instantiate_identity` to discharge the binder since we don't
            // mind using placeholders for any bound arguments
            _ => cx.tcx.fn_sig(def_id).instantiate_identity(),
        };

        let arg_names = cx.tcx.fn_arg_names(def_id);

        let args = fn_sig.inputs().skip_binder();

        for (arg_index, arg_ty) in args.iter().enumerate() {
            let TyKind::Ref(region, ty, Mutability::Mut) = arg_ty.kind() else {
                // We only care about `&mut` parameters
                continue;
            };

            let arg_ident = arg_names[arg_index];

            // This lint would emit a warning on `&mut self` if `self` was reborrowable. This isn't
            // useful, though, because it would hurt the ergonomics of using methods of
            // reborrowable types.
            //
            // To avoid this, we skip any parameter named `self`. This won't false-positive on
            // other function arguments named `self`, since it is a special keyword that is
            // disallowed in other positions.
            if arg_ident.name == kw::SelfLower {
                continue;
            }

            let Some(reborrowable) = Reborrowable::try_from_ty(cx, *ty) else {
                // The type is not one of our known re-borrowable types
                continue;
            };

            let is_output_bound_to_arg = fn_sig
                .output()
                .visit_with(&mut ContainsRegion(*region))
                .is_break();

            if is_output_bound_to_arg {
                // We don't want to suggest re-borrowing if the return type's
                // lifetime is bound to the argument's reference.
                // This is because it's impossible to convert something like:
                // `for<'a> (&'a mut Commands<'_, '_>) -> EntityCommands<'a>`
                // to something like:
                // `for<'a> (Commands<'_, '_>) -> EntityCommands<'a>`
                // without getting: `error[E0515]: cannot return value referencing function
                // parameter `commands` ``
                continue;
            }

            let span = decl.inputs[arg_index].span.to(arg_ident.span);

            span_lint_and_sugg(
                cx,
                BORROWED_REBORROWABLE.lint,
                span,
                reborrowable.message(),
                reborrowable.help(),
                reborrowable.suggest(arg_ident, ty.to_string()),
                // Not machine-applicable since the function body may need to
                // also be updated to account for the removed ref
                Applicability::MaybeIncorrect,
            );
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Reborrowable {
    Commands,
    Deferred,
    DeferredWorld,
    EntityCommands,
    EntityMut,
    FilteredEntityMut,
    Mut,
    MutUntyped,
    NonSendMut,
    PtrMut,
    Query,
    ResMut,
}

impl Reborrowable {
    fn try_from_ty<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> Option<Self> {
        use crate::paths::*;

        const PATH_MAP: &[(&[&str], Reborrowable)] = &[
            (&COMMANDS, Reborrowable::Commands),
            (&DEFERRED, Reborrowable::Deferred),
            (&DEFERRED_WORLD, Reborrowable::DeferredWorld),
            (&ENTITY_COMMANDS, Reborrowable::EntityCommands),
            (&ENTITY_MUT, Reborrowable::EntityMut),
            (&FILTERED_ENTITY_MUT, Reborrowable::FilteredEntityMut),
            (&MUT, Reborrowable::Mut),
            (&MUT_UNTYPED, Reborrowable::MutUntyped),
            (&NON_SEND_MUT, Reborrowable::NonSendMut),
            (&PTR_MUT, Reborrowable::PtrMut),
            (&QUERY, Reborrowable::Query),
            (&RES_MUT, Reborrowable::ResMut),
        ];

        for &(path, reborrowable) in PATH_MAP {
            if match_type(cx, ty, path) {
                return Some(reborrowable);
            }
        }

        None
    }

    fn message(&self) -> String {
        let name = self.name();
        format!("parameter takes `&mut {name}` instead of a re-borrowed `{name}`",)
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Commands => "Commands",
            Self::Deferred => "Deferred",
            Self::DeferredWorld => "DeferredWorld",
            Self::EntityCommands => "EntityCommands",
            Self::EntityMut => "EntityMut",
            Self::FilteredEntityMut => "FilteredEntityMut",
            Self::Mut => "Mut",
            Self::MutUntyped => "MutUntyped",
            Self::NonSendMut => "NonSendMut",
            Self::PtrMut => "PtrMut",
            Self::Query => "Query",
            Self::ResMut => "ResMut",
        }
    }

    fn help(&self) -> String {
        let name = self.name();
        format!("use `{name}` instead")
    }

    fn suggest(&self, ident: Ident, ty: String) -> String {
        format!("mut {ident}: {ty}")
    }
}

/// [`TypeVisitor`] for checking if the given region is contained in the type.
struct ContainsRegion<I: Interner>(pub I::Region);

impl<I: Interner> TypeVisitor<I> for ContainsRegion<I> {
    type Result = ControlFlow<()>;

    fn visit_region(&mut self, r: I::Region) -> Self::Result {
        if self.0 == r {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
