# How to Work with Types

> [!NOTE]
>
> This document assumes you are working with [`ty::Ty`], not [`rustc_hir::Ty`], unless explicitly stated otherwise.
>
> [`ty::Ty`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html
> [`rustc_hir::Ty`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Ty.html

## Getting the Type of an Expression

It is possible to get the type of an expression ([`Expr`]) through [`TypeckResults`]:

```rust
fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &Expr<'tcx>) {
    let ty = cx.typeck_results().expr_ty(expr);
}
```

[`Expr`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Expr.html
[`TypeckResults`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/typeck_results/struct.TypeckResults.html

## Peeling References

Often you have the type that may be behind one or more references, such as `&&str` or `*mut [T]`, but you need to extract the underlying type (`str` and `[T]` in this case). You can do this by "peeling" references:

```rust
let peeled_ty = ty.peel_refs();
```

See [`Ty::peel_refs()`] for more information.

[`Ty::peel_refs()`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html#method.peel_refs

## Checking for a Specific Type

Often you have a `Ty`, and want to check if it matches a specific hardcoded type, such as Bevy's [`App`]. You can do this with `clippy_utils`'s [`match_type()`] function:

```rust
use clippy_utils::ty::match_type;

// The absolute path to `App`'s definition.
const APP: [&str; 3] = ["bevy_app", "app", "App"];

if match_type(cx, ty, &APP) {
    // ...
}
```

All path constants are defined in [`paths.rs`](../../src/paths.rs). If you add a new constant, place it there.

> [!IMPORTANT]
>
> `bevy_app::app` is a [private module], but we still have to refer to it by name because [`struct App`] is within `bevy_app/src/app.rs`. Do not be tricked by re-exported types, such as `bevy::prelude::App`!
>
> [private module]: https://docs.rs/bevy_app/0.15.0/src/bevy_app/lib.rs.html#14
> [`struct App`]: https://docs.rs/bevy_app/0.15.0/src/bevy_app/app.rs.html#67-77

[`App`]: https://docs.rs/bevy/latest/bevy/app/struct.App.html
[`match_type()`]: https://doc.rust-lang.org/nightly/nightly-rustc/clippy_utils/ty/fn.match_type.html

## Getting `ty::Ty` from `rustc_hir::Ty`

Often you'll have an [`rustc_hir::Ty`], but you need [`ty::Ty`]. This is a process known as _lowering_, and it is accomplished through two different structures:

- [`FnCtxt`]: Used to type-check bodies of functions, closures, and `const`s. (Anything with expressions and statements.)
- [`ItemCtxt`]: Used to type-check item signatures.

It is important to use the right context for the right situation, or the compiler may panic!

Also note that this conversion is one-directional and cannot be easily reversed. While [`rustc_hir::Ty`]s are associated with a specific span of code, [`ty::Ty`]s are not. For more information, please see [`rustc_hir::Ty` vs `ty::Ty`] from the `rustc` Dev Guide.

[`rustc_hir::Ty`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Ty.html
[`ty::Ty`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html
[`FnCtxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_typeck/fn_ctxt/struct.FnCtxt.html
[`ItemCtxt`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir_analysis/collect/struct.ItemCtxt.html
[`rustc_hir::Ty` vs `ty::Ty`]: https://rustc-dev-guide.rust-lang.org/ty.html#rustc_hirty-vs-tyty

### Within Bodies

Instead of manually constructing a [`FnCtxt`], it is easier to go through [`TypeckResults::node_type()`]:

```rust
fn check_local(&mut self, cx: &LateContext<'tcx>, let_stmt: &LetStmt<'tcx>) {
    // Extract the type `T` from `let name: T = ...`, if it is specified.
    if let Some(hir_ty) = let_stmt.ty {
        // Find the `ty::Ty` for this `rustc_hir::Ty`. The reason this does not panic
        // is because the type is from a `let` statement, which must be within a body.
        let ty = cx.typeck_results().node_type(hir_ty.hir_id);
    }
}
```

[`TypeckResults::node_type()`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/typeck_results/struct.TypeckResults.html#method.node_type

### Outside Bodies

When outside of a body, you must construct an [`ItemCtxt`]:

```rust
fn check_ty(&mut self, cx: &LateContext<'tcx>, hir_ty: &rustc_hir::Ty<'tcx>) {
    // `ItemCtxt` needs a ` LocalDefId` of the item that this type is within, which we access
    // through the the type's `HirId`'s owner.
    let item_cx = ItemCtxt::new(cx.tcx, hir_ty.hir_id.owner.def_id);
    let ty = item_cx.lower_ty(hir_ty);
}
```
