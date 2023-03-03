## `rust-gpu` Workspace

Nightly rust workspace housing `rust-gpu` shader crates.

`rust-toolchain` contains the necessary toolchain specification for `rust-gpu`,
and `.cargo/config.toml` contains environment variables overriding the permutation file used to compile `bevy-pbr-rust` entry points.

To avoid depending on a local clone of `rust-gpu` for SPIR-V compilation, `rust-gpu-builder` has been added to the `crates` directory as a submodule,
and is set as the workspace's default build target.

Run `cargo run --release -- "crates/shader"` to produce `target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv`.

Run `cargo run --release -- "crates/shader" -w ./crates/shader/src -w ../bevy-app/crates/viewer/entry_points.json` to watch both workspaces and recompile on change.

### `shader`

Project-level `rust-gpu` shader crate. Pulls in `bevy-pbr-rust` to expose its entrypoints.

