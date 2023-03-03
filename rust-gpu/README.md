## `rust-gpu` Workspace

Nightly rust workspace housing `rust-gpu` crates.

`Cargo.toml` contains the `rust-gpu`-recommended `[profile.*.build-override]` settings to ensure fast shader compiles,

`rust-toolchain` contains the necessary toolchain specification for `rust-gpu`.

`.cargo/config.toml` contains environment variables overriding the permutation file path used to compile `bevy-pbr-rust`.

`cargo run --release "crates/shader"` to produce `target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv`.

`cargo run --release "crates/shader" -w ./crates -w ../bevy-app/crates/viewer` to watch both workspaces and recompile on change.

### `shader`

Project-level `rust-gpu` shader crate. Pulls in `bevy-pbr-rust` to expose its entrypoints.

Entrypoints are exported relative to their containing crate using rust module path syntax,
i.e. `mesh::vertex`, `pbr::fragment`.

