# bevy-rust-gpu

An example workspace demonstrating the use of rust-gpu to compile shaders for bevy projects.

## Overview

### Workspace

Cargo.toml contains the `rust-gpu`-recommended `[profile.*.build-override]` settings to ensure fast shader compiles,
as well as custom `[profile.dev]` and `[profile.dev.package."*"]` sections for fast bevy app compiles.

### Crates

#### shader

The rust-gpu shader crate.
Contains a working reimplementation of `bevy_pbr`.

Shader def conditionals are implemented as cargo features.

At time of writing, `rust-gpu` only supports read-write access to storage buffers,
which renders it incompatible with the read-only buffers bevy uses to store light and cluster data on supported platforms.

As such, the `NO_STORAGE_BUFFER_SUPPORT` feature is enabled by default, and the bevy app is configured to match.

#### shader-builder

Empty library crate used to invoke build.rs as its own build target.
Encapsulates the nightly build process needed for `rust-gpu`.

Contains the `rust_toolchain` file needed by `rust-gpu`, and invokes `spirv-builder` via build.rs.

Run via `cargo build -p shader-builder` to produce `target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv`.

#### viewer

Main bevy project.
Loads an example scene that renders a side-by-side comparison of WGSL and Rust PBR materials.

Uses the workspace root as its asset folder, and hot-reloads `target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv` file via AssetServer.

`WgpuLimits::max_storage_buffers_per_shader_stage` is forced to 0 via `WgpuSettings` to ensure a `NO_STORAGE_BUFFER_SUPPORT` environment.
