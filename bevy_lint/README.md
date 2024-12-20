<div class = "rustdoc-hidden">

# `bevy_lint`

`bevy_lint` is a custom linter for the [Bevy game engine](https://bevyengine.org), similar to [Clippy](https://doc.rust-lang.org/stable/clippy).

</div>

- [Documentation](https://thebevyflock.github.io/bevy_cli/bevy_lint/)
- [Repository](https://github.com/TheBevyFlock/bevy_cli)
- [Issue Tracker](https://github.com/TheBevyFlock/bevy_cli/issues?q=is%3Aopen+is%3Aissue+label%3AA-Linter)

<div class="rustdoc-alert rustdoc-alert-warning">

> **Warning**
>
> This is an unofficial community project, hacked upon by the Bevy CLI working group until it is eventually upstreamed into the main [Bevy Engine organization](https://github.com/bevyengine). Pardon our rough edges, and please consider [submitting an issue](https://github.com/TheBevyFlock/bevy_cli/issues) if you run into trouble!

</div>

## Installation

`bevy_lint` depends on a pinned nightly version of Rust with the `rustc-dev` Rustup component. This is because `bevy_lint` uses [internal `rustc` crates](https://doc.rust-lang.org/nightly/nightly-rustc/) that can only be imported with the permanently-unstable [`rustc_private` feature](https://doc.rust-lang.org/nightly/unstable-book/language-features/rustc-private.html). You can refer to the [compatibility table](#compatibility) to see which version of the linter requires which toolchain.

You can install the toolchain with:

```bash
rustup toolchain install $TOOLCHAIN_VERSION \
    --component rustc-dev \
    --component llvm-tools-preview
```

For example, you would replace `$TOOLCHAIN_VERSION` with `nightly-2024-11-14` if you were installing `bevy_lint` 0.1.0, based on the [compatibility table](#compatibility). Please be aware that you must keep this toolchain installed for `bevy_lint` to function[^keep-toolchain-installed].

[^keep-toolchain-installed]: `bevy_lint` imports internal `rustc` libraries in order to hook into the compiler process. These crates are stored in a [dynamic library](https://en.wikipedia.org/wiki/Dynamic_linker) that is installed with the `rustc-dev` component and loaded by `bevy_lint` at runtime. Uninstalling the nightly toolchain would remove this dynamic library, causing `bevy_lint` to fail.

Once you have the toolchain installed, you can compile and install `bevy_lint` through `cargo`:

```bash
rustup run $TOOLCHAIN_VERSION cargo install \    
    --git https://github.com/TheBevyFlock/bevy_cli.git \
    --tag $TAG \
    --locked \
    bevy_lint
```

Make sure to replace `$TOOLCHAIN_VERSION` and `$TAG` in the above command. The tag for a specific release can be found in the [releases tab](https://github.com/TheBevyFlock/bevy_cli/releases). For example, the tag for 0.1.0 is `lint-v0.1.0`.

## Usage

`bevy_lint` has the same API as the `cargo check` command:

```bash
bevy_lint --help
```

If you have the [Bevy CLI](https://github.com/TheBevyFlock/bevy_cli) installed, the linter is also available through the `lint` subcommand:

```bash
bevy lint --help
```

<div class="rustdoc-alert rustdoc-alert-note">

> **Note**
>
> `bevy_lint` checks your code with the nightly toolchain it was installed with, meaning you _do_ have access to unstable features when it is called. This is best used when [detecting `bevy_lint`](#detecting-bevy_lint).

</div>

### Detecting `bevy_lint`

The linter passes `--cfg bevy_lint` when it checks your code, allowing you to detect it:

```rust,ignore
// Conditionally include this function only when `bevy_lint` is used.
#[cfg(bevy_lint)]
fn foo() {
    // ...
}

// Conditionally add an attribute only when `bevy_lint` is used.
#[cfg_attr(bevy_lint, ...)]
struct Foo;
```

If you use this, you may also need to register `bevy_lint` as a valid `cfg` flag in your `Cargo.toml`:

```toml
[lints.rust]
unexpected_cfg = { level = "warn", check-cfg = ["cfg(bevy_lint)"] }
```

### Registering `bevy` as a Tool

When you run `bevy_lint` on a project, `rustc` knows an exact list of all `bevy::` lints registered. With this it can detect that `bevy::missing_reflect` is valid and `bevy::uh_oh` isn't, and emit a corresponding warning.

When you run normal `cargo check`, however, it does not know about _any_ `bevy::` lints. In order to avoid erroring on _all_ usages of `bevy::`, but to still provide good diagnostics on typos, the `#![register_tool(...)]` attribute was introduced.

```rust,ignore
// Note that this is nightly-only. We'll get to that in a second!
#![register_tool(bevy)]
```

Using `#![register_tool(bevy)]` tells the compiler that `bevy` is a valid name in attributes, even if it does not know what `bevy` is.[^rustfmt-skip] When `cargo check` now runs over a project with `#[warn(bevy::lint_name)]`, it will simply skip it instead of emitting an error. (But running `bevy_lint` will still detect and check this attribute as normal.)

[^rustfmt-skip]: If you've ever used `#[rustfmt::skip]` in your code, this is how `rustc` avoids erroring on it. However unlike the `bevy` namespace, `rustfmt` is registered automatically without a need for `#![register_tool(rustfmt)]` due to it being an official tool.

If you wish to refer to a `bevy` lint at all in your code or configuration (usually to [toggle it](#toggling-lints)), you must add `#![register_tool(bevy)]` to each crate root. Unfortunately, `#![register_tool(...)]` is [currently unstable](https://doc.rust-lang.org/nightly/unstable-book/language-features/register-tool.html), meaning you need to add `#![feature(register_tool)]` to your code as well. This isn't an issue if you [detect when `bevy_lint` is enabled](#detecting-bevy_lint), since it is guaranteed to check your code using nightly Rust.

```rust,ignore
// When `bevy_lint` is used, enable the `register_tool` feature and register the `bevy` namespace
// as a tool.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
```

<div class="rustdoc-alert rustdoc-alert-tip">

> **Tip**
>
> If your project already uses nightly Rust, you can forego the `#[cfg_attr(bevy_lint, ...)]` attributes and write `#![feature(register_tool)]` and `#![register_tool(bevy)]` directly!

</div>

### Toggling Lints

If you wish to enable and disable certain lints, you must first [register `bevy` as a tool](#registering-bevy-as-a-tool). Not doing so will cause `#[allow(bevy::lint_name)]` and related attributes to fail to compile.

You can now toggle lints throughout your project, as long as they too are behind `#[cfg_attr(bevy_lint, ...)]`:

```rust,ignore
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]

// Enable pedantic lints, which are off by default.
#![cfg_attr(bevy_lint, warn(bevy::pedantic))]

// Deny methods of `World` in this system that can panic when a non-panicking alternative exists.
#[cfg_attr(bevy_lint, deny(bevy::panicking_world_methods))]
fn my_critical_system(world: &mut World) {
    // ...
}
```

There are several other ways to toggle lints, but they have varying levels of support:

|Method|Support|Additional Information|
|-|-|-|
|`#[allow(...)]` and related|✅|Must be behind `#[cfg_attr(bevy_lint, ...)]` on stable Rust.|
|`[lints.bevy]` in `Cargo.toml`|⚠️|Nightly only because `#[register_tool(bevy)]` must always be enabled.|
|`[workspace.lints.bevy]` in `Cargo.toml`|⚠️|Nightly only (same as `[lints.bevy]`) and prints a warning each time `cargo` is executed.|
|`RUSTFLAGS="-A bevy::lint"`|❌|`RUSTFLAGS` applies to dependencies, but they do not have `#[register_tool(bevy)]`.|

## Compatibility

|`bevy_lint` Version|Rust Version|Rustup Toolchain|Bevy Version|
|-|-|-|-|
|0.2.0-dev|1.84.0|`nightly-2024-11-28`|0.14|
|0.1.0|1.84.0|`nightly-2024-11-14`|0.14|

The Rust version in the above table specifies what [version of the Rust language](https://github.com/rust-lang/rust/releases) can be compiled with `bevy_lint`. Code written for a later version of Rust may not compile. (This is not usually an issue, though, because `bevy_lint`'s Rust version is kept 1 to 2 releases ahead of stable Rust.)

The Rustup toolchain specifies which toolchain must be installed in order for `bevy_lint` to be installed and used. Please see [the installation section](#installation) for more info.

The Bevy version is a range of Bevy versions that `bevy_lint` has been tested with and is guaranteed to work. Newer or older releases may not be linted correctly and may cause the linter to crash. (If this does happen for you, please consider [submitting a bug report](https://github.com/TheBevyFlock/bevy_cli/issues)!)

## License

The Bevy Linter is licensed under either of

- Apache License, Version 2.0 ([`LICENSE-APACHE`](https://github.com/TheBevyFlock/bevy_cli/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([`LICENSE-MIT`](https://github.com/TheBevyFlock/bevy_cli/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributing

Please see [`CONTRIBUTING.md`](https://github.com/TheBevyFlock/bevy_cli/blob/main/CONTRIBUTING.md) for the CLI for more information! There is also a linter-specific contributing guide in the [`docs` folder](https://github.com/TheBevyFlock/bevy_cli/tree/main/bevy_lint/docs).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
