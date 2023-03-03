# `bevy-app` Workspace

Stable rust workspace housing our `bevy` app.

`Cargo.toml` contains custom `[profile.dev]` and `[profile.dev.package."*"]` sections for fast compiles.

`cargo run` after compiling shaders from the `rust-gpu` workspace to view the result.

## `viewer` Crate

Bevy binary crate, loads an example scene that renders a side-by-side comparison of WGSL and Rust PBR materials.

Uses the workspace root as its asset folder, and hot-reloads `rust-gpu/target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv` via AssetServer.

The shader is loaded into a custom `RustGpu<StandardMaterial>` material, which uses `bevy-rust-gpu` type machinery to resolve statically-compiled entrypoint permutations at runtime.
These permutations will be written to `entry_points.json`, which is read by the `rust-gpu` workspace at compile time and used to generate new entry points.
On compile, the `.spv` file will be reloaded, and its material re-specialized if necessary.

### Custom Bevy

`viewer` depends on [Shfty/bevy:remove-spv-defs](https://github.com/Shfty/bevy), which is the `v0.9.1` tag patched to prevent rejection of SPIR-V modules when shader defs are present.

