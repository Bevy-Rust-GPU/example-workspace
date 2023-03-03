# `bevy-app` Workspace

Stable rust workspace housing our `bevy` app.

`Cargo.toml` contains custom `[profile.dev]` and `[profile.dev.package."*"]` sections for fast compiles.

`cargo run` after building the `rust-gpu` workspace to preview the generated shader.

## `viewer` Crate

Bevy binary crate, loads an example scene that renders a side-by-side comparison of WGSL and Rust PBR materials.

Uses the workspace root as its asset folder, and hot-reloads `rust-gpu/target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv` via AssetServer.

The shader is loaded into a custom `RustGpuMaterial` material, which composes `StandardMaterial` with type-level entrypoint overrides and value-level shader overrides.

Shader permutations are selected by using the `RustGpuEntryPoint` trait to translate shader defs into an entrypoint name.

In addition, the `shader.spv.json` metadata generated by `rust-gpu` can be loaded as a resource and provided to `RustGpuMaterial` to enable runtime entrypoint validation.

This will defer material loading until metadata is available, and prevent bevy from panicking in case of a missing entrypoint. A warning will be printed, and the material will fall back to the default vertex / fragment shader.

`WgpuLimits::max_storage_buffers_per_shader_stage` is forced to 0 via `WgpuSettings` to ensure a `NO_STORAGE_BUFFER_SUPPORT` environment.

### Custom Bevy

`viewer` depends on [Shfty/bevy:remove-spv-defs](https://github.com/Shfty/bevy), which is the `v0.9.1` tag patched to prevent rejection of SPIR-V modules when shader defs are present.
