# BEV Front

A Rust real-time strategy game built with [Bevy](https://bevy.org) 0.20.1.

## Playtest

Choose **Play**, configure four Blue infantry squads, and deploy against the Red AI.
Capture all six objectives to win. A squad must stay in an objective's circle to
capture it. Both sides in the circle pause capture. Owned objectives remain
under your control when you leave.

Each team's deployment zone replenishes supplies and treats surviving soldiers.
Infantry automatically engages nearby opponents. The HUD shows selected squads'
remaining manpower, equipment, and supplies. Use smoke or entrench to protect
a squad while it holds position.

| Input | Action |
| --- | --- |
| Left click or drag | Select a squad or box-select several |
| Shift + select | Add squads to the selection |
| Right click terrain | Move selected squads |
| Shift + right click terrain | Queue another waypoint |
| Right click Red squad | Focus it |
| WASD or arrow keys | Pan |
| Mouse wheel | Zoom |
| Ctrl+A | Select all living Blue squads |
| Ctrl+1–4 | Assign a control group |
| 1–4 / Shift+1–4 | Recall a group / add it to the selection |
| Esc | Pause or resume |

Settings include opponent difficulty, borderless fullscreen, edge scrolling, and
interface scale.

## Local development

Install Rust stable (Bevy 0.20.1 requires Rust 1.97.1+) and the [Bevy Linux dependencies](https://bevy.org/learn/quick-start/getting-started/setup/) where appropriate.

```sh
cargo run                               # Initial build and normal local run
cargo fast                              # Opt-in Bevy dynamic linking for faster iteration
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Fast headless tests (no Bevy rendering dependency compiled)
cargo test -p brevy-core --lib

# Optional full Bevy app tests, much slower and not used by standard CI
cargo test -p brevy-rts --all-targets
```

`src/world.rs` renders the map and handles battlefield input. `src/interface.rs` implements the menus and floating HUD. `src/battle.rs` resolves the match, `src/ai.rs` directs Red squads, and `src/model.rs` holds the shared state. `crates/brevy-core` retains the engine-independent grid helpers and fast tests.

Commit the generated `Cargo.lock` to guarantee repeatable Rust dependency resolution.

### Why compilation can be slow

Bevy's much-quoted **0.8–3 second** figure comes from its 2020 introduction. It measured *warm incremental* builds of examples with a fast-compile configuration, **not** compiling the Bevy engine on a fresh runner or generating fully optimised release binaries.

For local editing, use `cargo fast` (an alias for `cargo run --features bevy/dynamic_linking`). This compiles the Bevy library dynamically and can reduce later incremental build/link time once the first full build is cached. It may still take several minutes the first time and requires platform-compatible dynamic library loading. Continue using `cargo run` for a standard development build.

Canary uses a dedicated `[profile.canary]` that prioritises build speed (`opt-level = 1`, more codegen units) over peak runtime performance. It produces **Linux x86-64 only**, with an HTML Cargo compilation-timings report attached to the workflow run. The stable release continues using `--release` and produces Linux and Windows binaries with full release optimisations.

Compiler options and linker speed vary by operating system and Rust version. Do not assume a clean GitHub Actions runner can achieve a sub-three-second release build.

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

A successful, push-triggered **Linting** run on `main` automatically updates the [rolling canary prerelease](https://github.com/Fuhrizen/Bevy-RTS/releases/tag/canary).

- The **single** mutable `canary` tag always targets the latest validated `main` commit, and the existing prerelease is updated instead of creating historical canary releases.
- Canary builds are packaged for **Linux x86-64 only**, using the faster canary profile. When executable-affecting files change, the workflow runs changed-code headless tests and rebuilds Linux; otherwise, it can advance the tag without recompiling or replacing identical assets. Existing Windows canary assets are removed on the next successful publication.
- In-progress canary runs are superseded when a newer `main` run succeeds. The publish job double-checks the current `main` SHA before updating the tag.
- Canary is always marked **prerelease** and **not latest**. Stable `vX.Y.Z` tags and releases are never modified by the canary pipeline.

To retain a particular canary build, archive its downloaded files separately before the next update. The `canary` tag is deliberately moved forward and must not be used as an immutable dependency.

Only the manual **Tag release** workflow produces stable releases. Pushing a tag alone does not launch a release.

## Skirmish operations

Choose a map and mode on the loadout screen. The four Blue squads share a
32-soldier budget, so increasing one squad may require thinning another.

- **Riverbend** uses the original river valley and wide sectors.
- **Highmoor** offsets the approach routes and uses a different relief map.
- **Crossroads** clusters objectives closer to the centre.

**Conquest** requires all six sectors. **Control** awards one point per second
per uncontested sector and ends when a side reaches 480 with the lead.
**Timed** uses the same scoring and finishes after three minutes. In Control
and Timed, eliminating the other force also wins. An even score at the Timed
deadline results in a draw.

Green rings mark cover, which reduces incoming damage by 25%. The north and
south depots replenish supply while friendly and uncontested. Resupply directs
selected squads to the nearest available depot or home base. Home bases also
restore health. Toggle Hold Fire applies one fire policy to all selected squads.

Matches start at 0.65x simulation speed. Use the pace controls to switch to 1x
or enter planning pause. Space toggles planning pause and restores your last
running speed. You can still move the camera, select squads, and issue orders
in planning pause. The squad marker and selected-squad panel show the current
condition. Selected attack orders draw a line and a ring around their target.

### Frontline mode

Select **Frontline** in the loadout and choose a reserve kit. Squads project
territory based on strength and distance. Sectors change hands when influence
reaches the flag. Blue and Red hatching show territorial influence, and amber
shows a contested boundary. A sector majority drains the opposing team's
resolve. The match ends when one team runs out of resolve or is eliminated.

Both sides start with 30 requisition points and gain one per simulation second.
One six-person reserve squad can deploy at home from 45 seconds onward for
60 points. Blue uses the chosen reserve kit, and Red receives Support.
