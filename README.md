# example-workspace

Example setup for hot-rebuilding `rust-gpu` shaders in response to entrypoints requested by a bevy application.

This repo contains two workspaces; `bevy-app` is a non-nightly rust workspace containing a shader viewer crate,
and `rust-gpu` is a nightly rust workspace configured to compile shader crates into SPIR-V.

See `README.md` in each subdirectory for more information.
