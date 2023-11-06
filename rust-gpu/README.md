## `rust-gpu` Workspace

```

```

Nightly rust workspace housing `rust-gpu` shader crates.

`rust-toolchain` contains the necessary toolchain specification for `rust-gpu`,
and `.cargo/config.toml` contains environment variables overriding the permutation file used to compile `bevy-pbr-rust` entry points, as well as some `rust-gpu` compile arguments.

To avoid depending on a local clone of `rust-gpu` for SPIR-V compilation, `rust-gpu-builder` has been added to the `crates` directory as a submodule,
and is set as the workspace's default build target.

Run `cargo run --release -- crates/shader ../bevy-app/crates/viewer/assets/rust-gpu/shader.rust-gpu.msgpack` to populate the `bevy-app` workspace with a compiled shader asset.

Run `cargo run --release -- "crates/shader" -w ./crates/shader/src -w ../bevy-app/crates/viewer/entry_points.json ../bevy-app/crates/viewer/assets/rust-gpu/shader.rust-gpu.msgpack` to watch both workspaces and recompile on change.

### `shader`

Project-level `rust-gpu` shader crate. Pulls in `bevy-pbr-rust` to expose its entrypoints.

