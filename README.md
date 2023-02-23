# bevy-rust-gpu

An example project demonstrating the use of rust-gpu to compile shaders for bevy projects.

## `bevy-app` Workspace

Stable rust workspace housing our `bevy` app.

`Cargo.toml` contains custom `[profile.dev]` and `[profile.dev.package."*"]` sections for fast compiles.

`cargo run` after building the `rust-gpu` workspace to preview the generated shader.

### `viewer` Crate

Main bevy crate. Loads an example scene that renders a side-by-side comparison of WGSL and Rust PBR materials.

Uses the workspace root as its asset folder, and hot-reloads `rust-gpu/target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv` via AssetServer.

The shader is loaded into a custom `ShaderMaterial` material, which composes `StandardMaterial` with overrides for vertex / fragment shaders and their entrypoints.

`WgpuLimits::max_storage_buffers_per_shader_stage` is forced to 0 via `WgpuSettings` to ensure a `NO_STORAGE_BUFFER_SUPPORT` environment.

#### Custom Bevy

`viewer` depends on [Shfty/bevy:remove-spv-defs](https://github.com/Shfty/bevy), which is the `v0.9.1` tag patched to prevent rejection of SPIR-V modules when shader defs are present.

## `rust-gpu` Workspace

Nightly rust workspace housing `rust-gpu` crates.

`Cargo.toml` contains the `rust-gpu`-recommended `[profile.*.build-override]` settings to ensure fast shader compiles,

`rust-toolchain` contains the necessary toolchain specification for `rust-gpu`.

`cargo build` to produce `target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv`.

### `bevy-pbr-rust` Crate

Contains a working reimplementation of `bevy_pbr`.

Shader def conditionals are implemented using compile-time trait generics, which opens the door to programmatic permutation generation.

At time of writing, `rust-gpu` only supports read-write access to storage buffers,
which renders it incompatible with the read-only buffers bevy uses to store light and cluster data on supported platforms.

As such, the `NO_STORAGE_BUFFER_SUPPORT` feature is enabled by default, and the bevy app is configured to match.

### `shader`

Project-level `rust-gpu` shader crate. Pulls in `bevy-pbr-rust`.

Entrypoints are exported relative to their containing crate using rust module path syntax,
i.e. `mesh::vertex`, `pbr::fragment`.

### `shader-builder` Crate

Empty library crate used to invoke `spirv-builder` via `build.rs` independently of the bevy app.

## `shared` Directory

Houses dependencies shared by the `shader` and `bevy-app` workspaces

### `shader-glam` Crate

Wrapper crate gating `glam` and `spirv-std::glam` behind cargo features.
Used for writing crates that can be shared between `rust-gpu` and regular `rust`.

### `shader-util` Crate

Contains utility traits for replicating common shading language functions.
