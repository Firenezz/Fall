# Contributing to Fall

Thank you for your interest in **Fall**, a [Bevy](https://bevyengine.org/) game project. This guide is written for newcomers, with a focus on **Linux** development. It covers how to set up your machine, build and run the game, run tests, and open a pull request.

If something is unclear or out of date, open an issue or ask in your PR — feedback on this document is welcome.

## Getting help

- **Bevy learning resources:** [bevyengine.org/learn](https://bevyengine.org/learn/) and the [Bevy Cheat Book](https://bevy-cheatbook.github.io/introduction.html) are the best starting points for engine concepts.
- **Community:** The [official Bevy Discord](https://discord.gg/bevy) is active and helpful for engine questions.
- **This repository:** Check [README.md](README.md) for template-level notes (web builds, known issues). Domain-specific design notes live under [`docs/`](docs/) (for example [docs/THERMAL_SIMULATION.md](docs/THERMAL_SIMULATION.md) for heat diffusion between tiles).

## Prerequisites

### Rust toolchain

The repo expects a standard [rustup](https://rustup.rs/) installation. [`rust-toolchain.toml`](rust-toolchain.toml) ensures these components are available:

- `rustfmt`
- `clippy`
- `rust-analyzer`

From the repository root, `rustup` will install missing components automatically when you run `cargo`.

CI uses the **stable** toolchain. If you use [Nix](https://nixos.org/), the dev shell in [`flake.nix`](flake.nix) pins Rust **1.92.0** (see [`nix/rust-toolchain.toml`](nix/rust-toolchain.toml)) and provides Linux libraries for Bevy. Enter the shell with:

```bash
nix develop
```

### Linux system packages

Bevy on Linux needs development libraries for graphics, windowing, and audio. Exact package names differ by distribution.

**Fedora** (also noted in [`todo.txt`](todo.txt)):

```bash
sudo dnf install \
  alsa-lib-devel \
  systemd-devel \
  wayland-devel \
  wayland-protocols-devel \
  clang \
  mold \
  pkg-config
```

**Debian / Ubuntu** (similar to what [CI](.github/workflows/ci.yml) installs):

```bash
sudo apt-get update
sudo apt-get install --no-install-recommends \
  libasound2-dev \
  libudev-dev \
  clang \
  mold \
  pkg-config
```

You will also want a working **Vulkan** stack (Mesa drivers on most distros are enough). If the game fails to create a window or GPU context, install your distribution’s Mesa/Vulkan packages and verify with `vulkaninfo` if needed.

Depending on your graphics card, you may have to install one of the following:
`vulkan-radeon`, `vulkan-intel`, or `mesa-vulkan-drivers`

[`/.cargo/config.toml`](.cargo/config.toml) configures the **mold** linker on `x86_64-unknown-linux-gnu` for faster links. Install `mold` and `clang` as above; without mold, you can comment out the `-Clink-arg=-fuse-ld=mold` lines in that file and use the system default linker instead.

### Optional: web (Wasm) tooling

To build or serve the browser version you need:

- [Trunk](https://trunkrs.dev/): `cargo install --locked trunk`
- Wasm target: `rustup target add wasm32-unknown-unknown`

See [Web build (optional)](#web-build-optional) below.

## Clone and first-time setup

```bash
git clone <repository-url>
cd Fall
```

No extra bootstrap scripts are required. The first `cargo build` or `cargo run` will download crates and may take several minutes.

Assets live under [`assets/`](assets/) and are loaded at runtime from the project root (keep `CARGO_MANIFEST_DIR` pointing at the repo when debugging).

## Building and running (native Linux)

**Default development run** (includes `dev_mode`, `debug_mode`, and `serde`):

```bash
cargo run
```

**Faster iteration** uses dynamic linking via the `dev_mode` feature (already in defaults):

```bash
cargo run --features dev_mode
```

**Release-style run** (no dev dynamic linking; closer to a shipped build):

```bash
cargo run --no-default-features --features release_mode
```

The binary is named `fall` and is produced at `target/debug/fall` (or `target/release/fall` with `--release`).

### Vulkan validation warnings

If you see noisy Vulkan validation messages (for example `VUID-StandaloneSpirv-MemorySemantics-10871`), they are a known false positive with Bevy/wgpu and do not affect gameplay. See [README.md — Vulkan Validation Warnings](README.md#vulkan-validation-warnings) for ways to filter or ignore them during development.

## Running tests

Run the full workspace test suite:

```bash
cargo test
```

Focus on a single crate when working on library code:

```bash
cargo test -p simulation
cargo test -p voronoi
cargo test -p common
```

Doc tests (also run in CI):

```bash
cargo test --doc --all-features
```

## Project layout

| Path | Role |
|------|------|
| [`src/`](src/) | Main Bevy application: game states, menu, world/tiles, sandbox, plugins wiring into workspace crates |
| [`src/main.rs`](src/main.rs) | Binary entry point (`fall`) |
| [`crates/common/`](crates/common/) | Shared types: world grid, buildings, units (temperature), serialization helpers |
| [`crates/simulation/`](crates/simulation/) | Simulation systems (e.g. thermal conduction); see [docs/THERMAL_SIMULATION.md](docs/THERMAL_SIMULATION.md) |
| [`crates/voronoi/`](crates/voronoi/) | Voronoi / geometry utilities with unit tests |
| [`assets/`](assets/) | Game data (RON manifests, textures, etc.) |
| [`build/`](build/) | Icons, web helpers, platform packaging (from the upstream Bevy template) |
| [`docs/`](docs/) | Design and contributor-oriented documentation |
| [`.github/workflows/`](.github/workflows/) | CI and release automation |

The root [`Cargo.toml`](Cargo.toml) defines a **workspace** with members `common`, `simulation`, and `voronoi`. The game crate `fall` depends on all three. **Bevy 0.18** is shared via `[workspace.dependencies]`.

## Code style and conventions

- **Edition:** Rust **2024** for the game and workspace crates.
- **Formatting:** Before opening a PR, run:

  ```bash
  cargo fmt --all
  ```

- **Linting:** CI runs Clippy with warnings denied. Match that locally with:

  ```bash
  cargo clippy --workspace --all-targets --all-features -- -Dwarnings
  ```

- **Feature flags** (root `fall` crate):
  - `dev_mode` — enables Bevy `dynamic_linking` for faster rebuilds (default).
  - `debug_mode` — optional debug dump tooling (default).
  - `serde` — serialization-related integration (default).
  - `release_mode` — defaults without `dev_mode` / `debug_mode`; use for release-like builds.
  - `editor` — optional editor integration (`jackdaw`).

  With `debug_mode` or debug builds, the game adds **bevy-inspector-egui** (`WorldInspectorPlugin`) for inspecting the ECS at runtime.

- **Simulation logic** belongs in [`crates/simulation/`](crates/simulation/) when it should stay testable without the full game; shared types go in [`crates/common/`](crates/common/).

- Match existing module structure and naming in the area you are changing rather than introducing one-off patterns.

## How to contribute

1. **Fork** the repository (if you are not a direct collaborator) and create a branch from the default branch.
2. **Make focused changes** — one logical change per PR when possible.
3. **Run tests and lint** locally (see above). CI on every push and pull request will:
   - Run `cargo test` on Windows, Linux, and macOS
   - Run `cargo test --doc --all-features` on Ubuntu
   - Run `cargo clippy` and `cargo fmt --check` on Ubuntu
4. **Open a pull request** with a short description of what changed and why. Mention any manual testing you did (e.g. “ran `cargo run`, verified sandbox heat diffusion”).
5. Keep **[`credits/`](credits/)** accurate if you add third-party assets or code with license requirements.

There is no separate style guide beyond `rustfmt`, Clippy, and the conventions in this file.

## Debugging tips (Linux)

### `dev_mode` and dynamic linking

With `dev_mode`, Bevy is linked dynamically. Debuggers and sometimes the shell need the Rust toolchain and `target/debug/deps` on the library path. [`.vscode/launch.json`](.vscode/launch.json) sets `LD_LIBRARY_PATH` for **CodeLLDB** configurations; use those as a reference if you debug from another IDE.

Quick launch from VS Code/Cursor: **“Quick Launch”** runs `cargo run --features dev_mode`.

### Inspector and diagnostics

In debug builds (or with `debug_mode`), the in-game **world inspector** (egui) is available for browsing entities and components. Frame diagnostics plugins may also be enabled in that configuration.

### Logging

Bevy’s `trace` feature is enabled on several dependencies; use `RUST_LOG` as usual, for example:

```bash
RUST_LOG=info cargo run
```

## Web build (optional)

The project supports a Wasm build via [Trunk](https://trunkrs.dev/), configured in [`Trunk.toml`](Trunk.toml) (default port **8080**).

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
trunk serve
```

Open `http://127.0.0.1:8080` after the first build finishes. Trunk rebuilds when sources change. See [README.md](README.md) for GitHub Pages deployment via the `deploy-github-page` workflow.

Audio in browser builds can be flaky in some browsers; that is a known template limitation, not specific to Fall.

## What not to commit

- **`target/`** — build artifacts (already in [`.gitignore`](.gitignore))
- **`dist/`** — Trunk output
- **Local IDE/OS junk** — e.g. `.idea/`, `.DS_Store`
- **Secrets** — API keys, tokens, personal `.env` files
- **Huge generated binaries** unless they are intentional release artifacts handled by CI

Run `cargo fmt` before committing so CI’s format check passes.

## License

The project is under [CC0 1.0 Universal](LICENSE) except where noted for assets and template icons; see [credits/CREDITS.md](credits/CREDITS.md). By contributing, you agree that your contributions can be licensed under the same terms unless you state otherwise in the PR.
