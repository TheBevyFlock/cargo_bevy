# How to Release `bevy_lint`

## Kick-off Pull Request

1. Review the [changelog](../../CHANGELOG.md) and ensure that all notable changes have been documented.
2. Replace `[Unreleased]` heading with the version with the format `[X.Y.Z] - YYYY-MM-DD`.
3. Update the `**All Changes**` link to compare from `main` to the new tag `lint-vX.Y.Z`. (E.g. `lint-v0.1.0...main` to `lint-v0.1.0...lint-v0.2.0`.)
4. Remove the `-dev` suffix from the version in [`Cargo.toml`](../../Cargo.toml) and the compatibility table in [`README.md`](../../README.md).
      - Please ensure that [`Cargo.lock`](../../../Cargo.lock) also updates!
6. Commit all of these changes and open a pull request.
7. Merge the PR once a core Bevy maintainer approves it with no outstanding issues from other contributors.
      - This starts the release process, enacting a freeze on all other changes until the release has finished. While maintainers need to be aware of this so they do not merge PRs during this time, the release process should take less than an hour, so it's unlikely to ever be an issue.

## Release on Github

1. [Create a new Github release](https://github.com/TheBevyFlock/bevy_cli/releases/new).
2. Set the tag to `lint-vX.Y.Z`.
3. Set the title to `` `bevy_lint` - vX.Y.Z``
4. Paste and fill out the following template into the release description:

````markdown
<!-- One-sentence summary of changes. What awesome features can we spotlight? What critical bugs were fixed? -->

You can find the live documentation for this release [here](https://thebevyflock.github.io/bevy_cli/bevy_lint/).

> [!WARNING]
>
> This is an unofficial community project, hacked upon by the Bevy CLI working group until it is eventually upstreamed into the main [Bevy Engine organization](https://github.com/bevyengine). Pardon our rough edges, and please consider [submitting an issue](https://github.com/TheBevyFlock/bevy_cli/issues) if you run into trouble!

<!-- You can refer to the compatibility table in `bevy_lint/README.md` for the following two values. -->

This release uses the <!-- `nightly-YYYY-MM-DD` --> toolchain, based on Rust <!-- 1.XX.Y -->. You can install it from Git with the following commands:

<!-- Update `nightly-YYYY-MM-DD` and `lint-vX.Y.Z` in the following code block. -->

```bash
rustup toolchain install nightly-YYYY-MM-DD \
    --component rustc-dev \
    --component llvm-tools-preview

rustup run nightly-YYYY-MM-DD cargo install \    
    --git https://github.com/TheBevyFlock/bevy_cli.git \
    --tag lint-vX.Y.Z \
    --locked \
    bevy_lint
```

<!-- Paste the changelog for this release here. Make sure to include the "All Changes" link. :) -->
````

5. Check the pre-release box if this is an alpha release, then click "Publish release"!
6. Announce the release on Discord! Congrats!

## Post-Release

1. Add a new unreleased section to the top of the [changelog](../../CHANGELOG.md) from the following template:

```markdown
## [Unreleased]

<!-- Update `lint-vX.Y.Z` in the link to point to the latest release tag. -->

**All Changes**: [`lint-vX.Y.Z...main`](https://github.com/TheBevyFlock/bevy_cli/compare/lint-vX.Y.Z...main)
```

2. Bump the version in [`Cargo.toml`](../../Cargo.toml) to the next `-dev` version, and ensure [`Cargo.lock`](../../../Cargo.lock) also updates.
3. Add a new row to the compatibility table for the new `-dev` version in [`README.md`](../../README.md).
4. Commit all of these changes and open a pull request.
5. Merge the PR after it has been approved, unblocking frozen pull requests.
