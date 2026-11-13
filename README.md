# Brevy RTS

A minimal [Bevy](https://bevy.org) 0.20.1 Rust application. The `main` branch contains an empty Bevy starter and the CI/CD foundation. Gameplay will be added through reviewed feature branches.

## Development

Install the current stable Rust toolchain (Bevy 0.20.1 requires Rust 1.97.1 or newer). On Linux, install the native dependencies described in the [Bevy setup guide](https://bevy.org/learn/quick-start/getting-started/setup/).

```sh
cargo run
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Commit `Cargo.lock` once generated to make dependency resolution reproducible.

## CI/CD

- **Linting:** automatic on pull requests and pushes to `main`; checks formatting, Clippy, and tests.
- **Tag release:** the **only** release entry point, run manually under **Actions → Tag release → Run workflow**, with branch **main** selected.
- Choose **patch**, **minor**, or **major**. The workflow increments the highest existing `vX.Y.Z` tag (starting from `v0.0.0` when no tags exist). It verifies a successful existing `main` Linting run for the exact source commit (without running Linting again), builds Linux and Windows x86-64 binaries, then creates an annotated tag **only after both builds succeed** and publishes one GitHub release with generated commit/PR notes.
- Pushing a tag manually does **not** start a release build. Only the manual workflow publishes releases.
- Builds use the exact validated commit. Dependency versions are not changed automatically by the release workflow.

## RTS starter world

The game scaffold adds an 18 × 12 tile map and two opposing groups of placeholder units, rendered in 2D. Pan the camera with **WASD** or the **arrow keys**.

The map and units are deliberately visual placeholders. Selection, issuing orders, pathfinding, combat, resource systems and multiplayer will be separate gameplay features.
