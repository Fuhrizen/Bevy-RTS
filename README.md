# Brevy RTS

A Rust real-time strategy game scaffold using **Bevy 0.20.1**. The starter opens a 2D map with two groups of placeholder units; use **WASD** or the **arrow keys** to pan the camera.

## Develop

Install the stable Rust toolchain using [rustup](https://rustup.rs/). On Linux, install the system packages described in [Bevy's Linux dependency guide](https://github.com/bevyengine/bevy/blob/v0.20.1/docs/linux_dependencies.md).

```bash
cargo run
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

This is a starter world, not a full RTS: selection, commands, pathfinding, economy, and combat are future features. The source is organised around Bevy plugins to accommodate them.

## Continuous integration and releases

GitHub Actions validates formatting, Clippy, and tests on pull requests and pushes to `main`. Linux and Windows **release** executables are built only for tagged versions, not every development commit.

To publish a new version, open **Actions → Tag release → Run workflow** on `main`. Choose `patch`, `minor`, `major`, or `custom`. For `custom`, enter the exact `X.Y.Z` (optionally prefixed with `v`). The workflow chooses the next version from existing `vX.Y.Z` tags, validates it, pushes an annotated tag, builds both platforms, and publishes one GitHub release containing the binaries and commit summaries.

You can also push a `vX.Y.Z` tag yourself; that invokes the same release build. Release tags are the authoritative distributed version labels. The `Cargo.toml` package version describes the development crate and is not rewritten by release automation.

The first build generates `Cargo.lock`. **Commit that lockfile from a Rust-enabled machine** to make dependency resolution repeatable across later CI and release runs. Do not add `Cargo.lock` to `.gitignore`.
