# Brevy RTS

A Rust real-time strategy game built with [Bevy](https://bevy.org) 0.20.1.

The starter contains an **18 × 12** tile map, two opposing groups of placeholder units, and **WASD/arrow-key** camera controls.

## Local development

Install Rust stable (Bevy 0.20.1 requires Rust 1.97.1+) and the [Bevy Linux dependencies](https://bevy.org/learn/quick-start/getting-started/setup/) where appropriate.

```sh
cargo run
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Fast headless tests (no Bevy rendering dependency compiled)
cargo test -p brevy-core --lib

# Optional full Bevy app tests, much slower and not used by standard CI
cargo test -p brevy-rts --all-targets
```

`src/world.rs` is the Bevy rendering layer. `crates/brevy-core` contains engine-independent gameplay/grid rules with fast tests. Keep new business logic in the core crate when possible, leaving ECS/rendering/input integration in the Bevy application.

Commit the generated `Cargo.lock` to guarantee repeatable Rust dependency resolution.

## Validation

**Linting** is the required GitHub check for PRs and `main`:

- The workflow checks changed-file whitespace on every run.
- Rust formatting and Clippy run only if Rust inputs changed (`src/`, `crates/`, `tests/`, Cargo manifests/lockfile, build scripts, Rust toolchain and Cargo configuration).
- Documentation and workflow-only changes retain a successful **Linting** check without compiling Bevy.
- Unit tests are **not** repeated on every PR or `main` push. Release workflows run headless tests when relevant code has changed since their previous channel version.
- Every Linux and Windows release build still compiles the complete game as a final compile-time validation. Full Bevy integration tests may be run manually when needed.

## Stable releases

Under **Actions → Tag release → Run workflow**, choose `patch`, `minor`, or `major` with `main` selected.

The workflow verifies an existing successful `main` **Linting** run for the exact SHA, determines the next `vX.Y.Z` tag, runs headless tests only when testable changes occurred since the previous stable release, builds the full game on Linux and Windows, and creates an immutable-version GitHub release with generated notes. Tags are created **after** both builds succeed. The release workflow does not rerun the normal PR lint checks.

## Canary releases

A successful, push-triggered **Linting** run on `main` automatically updates the [rolling canary prerelease](https://github.com/Fuhrizen/Brevy-RTS/releases/tag/canary).

- The **single** mutable `canary` tag always targets the latest validated `main` commit, and the existing prerelease is updated instead of creating historical canary releases.
- Canary builds are packaged for Linux x86-64 and Windows x86-64. When executable-affecting files change, the workflow runs changed-code headless tests and rebuilds both; otherwise, it can advance the tag without recompiling or replacing identical assets.
- In-progress canary runs are superseded when a newer `main` run succeeds. The publish job double-checks the current `main` SHA before updating the tag.
- Canary is always marked **prerelease** and **not latest**. Stable `vX.Y.Z` tags and releases are never modified by the canary pipeline.

To retain a particular canary build, archive its downloaded files separately before the next update. The `canary` tag is deliberately moved forward and must not be used as an immutable dependency.

Only the manual **Tag release** workflow produces stable releases. Pushing a tag alone does not launch a release.
