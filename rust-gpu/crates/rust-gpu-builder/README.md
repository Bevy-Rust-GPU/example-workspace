<div align="center">

# `rust-gpu-builder`

[![Documentation](https://img.shields.io/badge/docs-API-blue)](https://bevy-rust-gpu.github.io/rust-gpu-builder/)

`rust-gpu` helper program for automating shader crate compilation.

Can compile once, or run in daemon mode and recompile in response to filesystem changes.

</div>

## Usage

`rust-gpu-builder` relies on the [`spirv-builder`](https://embarkstudios.github.io/rust-gpu/api/spirv_builder/) crate, which in turn relies on the cargo ecosystem
to configure the appropriate nightly toolchain for `rust-gpu`. As such, it needs to be run from inside a workspace via `cargo run`.

Thus, it's recommended to add `rust-gpu-builder` as a a git submodule of your cargo workspace, and set it up as the default binary target.

### One-shot compilation

`cargo run --release -- <path-to-shader-crate>` will compile the provided shader crate and output `<crate-name>.spv` and `<crate-name>.spv.json` to `target/spirv-unknown-spv1.5/release/deps/`.

### Hot-recompile

`cargo run --release -- <path-to-shader-crate> -w <path-to-watch>` will compile as per the above, then watch the provided path and recompile whenever it changes.
