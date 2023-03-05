# `bevy-app` Workspace

Stable rust workspace housing our `bevy` app.

`Cargo.toml` contains custom `[profile.dev]` and `[profile.dev.package."*"]` sections for fast compiles.

In addition, it contains a patch section redirecting bevy to [Shfty/bevy:early-shader-defs](https://github.com/Shfty/bevy/tree/early-shader-defs),
which is the `v0.9.1` tag patched to inject built-in shader defs in time for `Material` to clear them and prevent a `ShaderProcessor` error.

This is a necessary prerequisite for SPIR-V usage in `Material` implementors, and is tracked in [this `bevy` issue](https://github.com/bevyengine/bevy/issues/7771).

Compile shaders from the `rust-gpu` workspace, and run one of the following to view the result:

`cargo run --example simple-material` to view a simple material that can be edited from the `shader` crate in the `rust-gpu` workspace.

`cargo run --example standard-material` to view a side-by-side comparison of WGSL and Rust StandardMaterial.

## `viewer` Crate

Bevy binary crate, loads an example scene that renders a side-by-side comparison of WGSL and Rust PBR materials.

Uses the workspace root as its asset folder, and hot-reloads `rust-gpu/target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv` via AssetServer.

The shader is loaded into a custom `RustGpu<StandardMaterial>` material, which uses `bevy-rust-gpu` type machinery to resolve statically-compiled entrypoint permutations at runtime.
These permutations will be written to `entry_points.json`, which is read by the `rust-gpu` workspace at compile time and used to generate new entry points.
On compile, the `.spv` file will be reloaded, and its material re-specialized if necessary.

